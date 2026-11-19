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
}

impl Vault {
    pub fn create_client(&self, name: &str, allow_chats: bool) -> Result<NewClient, String> {
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
        self.connection.execute("INSERT INTO agent_clients(id,name,token_hash,allow_chats,created_at) VALUES (?1,?2,?3,?4,unixepoch())",params![id,name,hash,allow_chats]).map_err(|e| e.to_string())?;
        Ok(NewClient {
            client: AgentClient {
                id,
                name: name.into(),
                allow_chats,
            },
            token,
        })
    }
    pub fn clients(&self) -> Result<Vec<AgentClient>, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT id,name,allow_chats FROM agent_clients ORDER BY created_at,id")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(AgentClient {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    allow_chats: r.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string());
        rows
    }
    pub fn revoke_client(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM agent_clients WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn agent_call(&self, request: &AgentRequest) -> Result<serde_json::Value, String> {
        if request.token.len() != 64 || request.client_id.len() > 80 {
            return Err("Client access was denied.".into());
        }
        let client: Option<(String, bool)> = self
            .connection
            .query_row(
                "SELECT token_hash,allow_chats FROM agent_clients WHERE id=?1",
                [&request.client_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let (expected, allow_chats) = client.ok_or("Client access was denied or revoked.")?;
        let actual = hex::encode(Sha256::digest(request.token.as_bytes()));
        if !bool::from(expected.as_bytes().ct_eq(actual.as_bytes())) {
            return Err("Client access was denied.".into());
        }
        let limit = request.limit.unwrap_or(20).clamp(1, 50);
        match request.tool.as_str() {
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
                    .collect::<Vec<_>>();
                serde_json::to_value(items).map_err(|e| e.to_string())
            }
            "get_context_pack" => {
                Ok(serde_json::json!({"markdown":self.context_pack(&request.query,allow_chats)?}))
            }
            "search_conversations" => {
                if !allow_chats {
                    return Err("This client is approved for memory only. Chat access must be enabled by the owner in Pre-Drink.".into());
                }
                if request.query.trim().is_empty() {
                    return Err("Provide a search query.".into());
                }
                serde_json::to_value(
                    self.list(&request.query, false, 0)?
                        .into_iter()
                        .take(limit)
                        .collect::<Vec<_>>(),
                )
                .map_err(|e| e.to_string())
            }
            "mark_private" => {
                if !allow_chats {
                    return Err("This client is approved for memory only.".into());
                }
                self.mark_private(&request.conversation_id, true)?;
                Ok(serde_json::json!({"private":true}))
            }
            _ => Err("Unknown tool.".into()),
        }
    }
}
