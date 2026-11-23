use crate::db::Vault;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

#[derive(Serialize)]
pub struct AgentClient {
    pub id: String,
    pub name: String,
    pub allow_chats: bool,
    pub provider: Option<String>,
    pub can_curate: bool,
    pub paused: bool,
    pub last_request_at: Option<f64>,
}

#[derive(Serialize)]
pub struct NewClient {
    pub client: AgentClient,
    pub token: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRequest {
    pub client_id: String,
    pub token: String,
    pub tool: String,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub conversation_id: String,
    #[serde(default)]
    pub curation: Option<crate::curation::CurationOperation>,
}

impl Vault {
    pub fn create_client(&self, name: &str, allow_chats: bool) -> Result<NewClient, String> {
        self.create_scoped_client(name, allow_chats.then_some("chatgpt"), false)
    }

    pub fn create_scoped_client(
        &self,
        name: &str,
        provider: Option<&str>,
        can_curate: bool,
    ) -> Result<NewClient, String> {
        if provider.is_some_and(|p| !matches!(p, "chatgpt" | "grok" | "claude"))
            || (can_curate && provider.is_none())
        {
            return Err("Choose a supported provider scope.".into());
        }
        let allow_chats = provider.is_some();
        let name = name.trim();
        if name.is_empty() || name.len() > 80 {
            return Err("Name the client using 1–80 bytes.".into());
        }
        let mut secret = [0u8; 32];
        getrandom::fill(&mut secret).map_err(|_| "Secure randomness unavailable.")?;
        let token = hex::encode(secret);
        use zeroize::Zeroize;
        secret.zeroize();
        let hash = hex::encode(Sha256::digest(token.as_bytes()));
        let id = uuid::Uuid::new_v4().to_string();
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO agent_clients(id,name,token_hash,allow_chats,created_at) VALUES (?1,?2,?3,?4,unixepoch())",params![id,name,hash,allow_chats]).map_err(|e| e.to_string())?;
        if let Some(provider) = provider {
            tx.execute(
                "INSERT INTO agent_scopes(client_id,provider,can_curate) VALUES (?1,?2,?3)",
                params![id, provider, can_curate],
            )
            .map_err(|e| e.to_string())?;
        }
        crate::agent_activity::record(&tx, Some(&id), name, "client_created", "completed")?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(NewClient {
            client: AgentClient {
                id,
                name: name.into(),
                allow_chats,
                provider: provider.map(str::to_owned),
                can_curate,
                paused: false,
                last_request_at: None,
            },
            token,
        })
    }
    pub fn clients(&self) -> Result<Vec<AgentClient>, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT a.id,a.name,s.provider IS NOT NULL,s.provider,COALESCE(s.can_curate,0),COALESCE(c.paused,0),c.last_request_at FROM agent_clients a LEFT JOIN agent_scopes s ON s.client_id=a.id LEFT JOIN agent_controls c ON c.client_id=a.id ORDER BY a.created_at,a.id")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(AgentClient {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    allow_chats: r.get(2)?,
                    provider: r.get(3)?,
                    can_curate: r.get(4)?,
                    paused: r.get(5)?,
                    last_request_at: r.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string());
        rows
    }
    pub fn revoke_client(&self, id: &str) -> Result<(), String> {
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let name: Option<String> = tx
            .query_row("SELECT name FROM agent_clients WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(name) = name {
            crate::agent_activity::record(&tx, Some(id), &name, "client_revoked", "completed")?;
            tx.execute("DELETE FROM agent_clients WHERE id=?1", [id])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
    pub fn agent_call(&self, request: &AgentRequest) -> Result<serde_json::Value, String> {
        if request.token.len() != 64 || request.client_id.len() > 80 {
            return Err("Client access was denied.".into());
        }
        let client: Option<(String, Option<String>, bool, bool)> = self
            .connection
            .query_row(
                "SELECT a.token_hash,s.provider,COALESCE(s.can_curate,0),COALESCE(c.paused,0) FROM agent_clients a LEFT JOIN agent_scopes s ON s.client_id=a.id LEFT JOIN agent_controls c ON c.client_id=a.id WHERE a.id=?1",
                [&request.client_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let (expected, provider, can_curate, paused) =
            client.ok_or("Client access was denied or revoked.")?;
        let actual = hex::encode(Sha256::digest(request.token.as_bytes()));
        if !bool::from(expected.as_bytes().ct_eq(actual.as_bytes())) {
            return Err("Client access was denied.".into());
        }
        let event = self.begin_agent_activity(&request.client_id, &request.tool)?;
        if paused {
            self.finish_agent_activity(event, "paused")?;
            return Err("This connection is paused. Resume it in Pre-Drink to use its existing permissions.".into());
        }
        let result = self.dispatch_agent_call(request, provider, can_curate);
        self.finish_agent_activity(
            event,
            if result.is_ok() {
                "completed"
            } else {
                "failed"
            },
        )?;
        result
    }
    fn dispatch_agent_call(
        &self,
        request: &AgentRequest,
        provider: Option<String>,
        can_curate: bool,
    ) -> Result<serde_json::Value, String> {
        let limit = request.limit.unwrap_or(20).clamp(1, 50);
        match request.tool.as_str() {
            "connection_status" => Ok(serde_json::json!({
                "connected": true, "provider": provider,
                "allow_history": provider.is_some(), "can_curate": can_curate,
            })),
            "get_facts" | "get_decisions" | "get_projects" => {
                let kind = match request.tool.as_str() {
                    "get_facts" => "fact",
                    "get_decisions" => "decision",
                    _ => "project",
                };
                let items = self
                    .memories(Some(kind), "", true)?
                    .into_iter()
                    .take(50)
                    .map(crate::memory::AgentMemory::from)
                    .collect::<Vec<_>>();
                serde_json::to_value(items).map_err(|e| e.to_string())
            }
            "get_context_pack" => {
                Ok(serde_json::json!({"markdown":self.context_pack(&request.query,false)?}))
            }
            "search_conversations" => {
                if provider.is_none() {
                    return Err("This client is approved for memory only. Chat access must be enabled by the owner in Pre-Drink.".into());
                }
                if request.query.trim().is_empty() {
                    return Err("Provide a search query.".into());
                }
                serde_json::to_value(
                    self.provider_conversations(
                        provider.as_deref().unwrap(),
                        &request.query,
                        limit,
                    )?
                    .into_iter()
                    .take(limit)
                    .collect::<Vec<_>>(),
                )
                .map_err(|e| e.to_string())
            }
            "mark_private" => {
                if provider.is_none() {
                    return Err("This client is approved for memory only.".into());
                }
                self.require_provider_conversation(
                    provider.as_deref().unwrap(),
                    &request.conversation_id,
                )?;
                self.mark_private(&request.conversation_id, true)?;
                Ok(serde_json::json!({"private":true}))
            }
            "curate_memory" => {
                if !can_curate {
                    return Err("This client has no curation permission.".into());
                }
                self.curate(
                    provider.as_deref().ok_or("No provider scope.")?,
                    &request.client_id,
                    request
                        .curation
                        .as_ref()
                        .ok_or("Missing curation operation.")?,
                )
            }
            _ => Err("Unknown tool.".into()),
        }
    }
}
