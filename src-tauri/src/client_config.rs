//! Client-specific settings; never write into another application's configuration.
use crate::{agents::NewClient, db::Vault};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientFormat {
    Generic,
    Codex,
    Grok,
    Claude,
}

#[derive(Serialize)]
struct Stdio<'a> {
    command: &'a str,
    args: [&'static str; 1],
    env: BTreeMap<&'static str, &'a str>,
}

fn path_text(path: &Path) -> Result<&str, String> {
    path.to_str()
        .filter(|text| path.is_absolute() && !text.chars().any(char::is_control))
        .ok_or_else(|| "Agent setup requires absolute paths without control characters and with valid Unicode. Move the app or vault to a supported location.".into())
}

fn render(
    format: ClientFormat,
    executable: &Path,
    socket: &Path,
    client_id: &str,
    token: &str,
) -> Result<String, String> {
    let command = path_text(executable)?;
    let socket = path_text(socket)?;
    // Grok expands these expressions even inside quoted TOML strings. Refuse
    // ambiguous paths rather than launch a different executable or socket.
    if matches!(format, ClientFormat::Grok)
        && [command, socket].iter().any(|text| text.contains("${"))
    {
        return Err("Grok expands environment expressions in paths. Move the app or vault to a path without ${ before creating its configuration.".into());
    }
    let servers = BTreeMap::from([(
        "pre-drink",
        Stdio {
            command,
            args: ["--mcp"],
            env: BTreeMap::from([
                ("PRE_DRINK_SOCKET", socket),
                ("PRE_DRINK_CLIENT_ID", client_id),
                ("PRE_DRINK_CLIENT_TOKEN", token),
            ]),
        },
    )]);
    match format {
        ClientFormat::Generic | ClientFormat::Claude => {
            serde_json::to_string_pretty(&BTreeMap::from([("mcpServers", servers)]))
                .map_err(|_| "Could not format client settings.".into())
        }
        ClientFormat::Codex | ClientFormat::Grok => {
            toml::to_string_pretty(&BTreeMap::from([("mcp_servers", servers)]))
                .map_err(|_| "Could not format client settings.".into())
        }
    }
}

pub fn create_configuration(
    vault: &Vault,
    name: &str,
    allow_chats: bool,
    format: ClientFormat,
    executable: &Path,
    socket: &Path,
) -> Result<String, String> {
    create_scoped_configuration(vault, name, allow_chats, false, format, executable, socket)
}

pub fn create_scoped_configuration(
    vault: &Vault,
    name: &str,
    allow_chats: bool,
    can_curate: bool,
    format: ClientFormat,
    executable: &Path,
    socket: &Path,
) -> Result<String, String> {
    create_setup(
        vault,
        name,
        allow_chats,
        can_curate,
        format,
        executable,
        socket,
    )
    .map(|setup| setup.configuration)
}

#[derive(Serialize)]
pub struct ClientSetup {
    pub client: crate::agents::AgentClient,
    pub configuration: String,
}

pub fn create_setup(
    vault: &Vault,
    name: &str,
    allow_chats: bool,
    can_curate: bool,
    format: ClientFormat,
    executable: &Path,
    socket: &Path,
) -> Result<ClientSetup, String> {
    // Format is owner-selected, never supplied by an MCP caller. Generic clients
    // remain shared-only until an explicit provider enrollment is designed.
    let provider = if allow_chats || can_curate {
        match format {
            ClientFormat::Codex => Some("chatgpt"),
            ClientFormat::Grok => Some("grok"),
            ClientFormat::Claude => Some("claude"),
            ClientFormat::Generic => {
                return Err("Choose Codex, Grok or Claude for provider-specific access.".into())
            }
        }
    } else {
        None
    };
    // Validate paths before registering. A formatting failure must leave no grant.
    render(format, executable, socket, "validation", "validation")?;
    let NewClient { client, token } = vault.create_scoped_client(name, provider, can_curate)?;
    let token = Zeroizing::new(token);
    match render(format, executable, socket, &client.id, &token) {
        Ok(configuration) => Ok(ClientSetup {
            client,
            configuration,
        }),
        Err(error) => {
            vault.revoke_client(&client.id)?;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::AgentRequest;
    use serde_json::{json, Value};

    fn decode(format: ClientFormat, text: &str) -> Value {
        match format {
            ClientFormat::Generic | ClientFormat::Claude => {
                serde_json::from_str::<Value>(text).unwrap()["mcpServers"].clone()
            }
            _ => serde_json::to_value(toml::from_str::<toml::Value>(text).unwrap()).unwrap()
                ["mcp_servers"]
                .clone(),
        }
    }

    #[test]
    fn formats_round_trip_paths_and_keep_credentials_out_of_command_arguments() {
        let command =
            Path::new("/Applications/Fictional \"Garden\" \\ 花.app/Contents/MacOS/pre-drink");
        let socket = Path::new("/tmp/Fictional \"Garden\" \\ 🌱/agents.sock");
        for format in [
            ClientFormat::Generic,
            ClientFormat::Codex,
            ClientFormat::Grok,
            ClientFormat::Claude,
        ] {
            let config =
                render(format, command, socket, "fictional-id", "fictional-token").unwrap();
            assert_eq!(
                decode(format, &config),
                json!({"pre-drink":{
                    "command": command, "args":["--mcp"], "env":{
                        "PRE_DRINK_SOCKET":socket,
                        "PRE_DRINK_CLIENT_ID":"fictional-id",
                        "PRE_DRINK_CLIENT_TOKEN":"fictional-token"
                    }
                }})
            );
        }
    }

    #[test]
    fn generated_credentials_obey_scope_and_revocation_for_every_format() {
        let vault = Vault::memory().unwrap();
        for format in [
            ClientFormat::Generic,
            ClientFormat::Codex,
            ClientFormat::Grok,
            ClientFormat::Claude,
        ] {
            for allow_chats in [false, true] {
                if matches!(format, ClientFormat::Generic) && allow_chats {
                    continue;
                }
                let text = create_configuration(
                    &vault,
                    "Fictional client",
                    allow_chats,
                    format,
                    Path::new("/Applications/Fictional.app/pre-drink"),
                    Path::new("/tmp/fictional/agents.sock"),
                )
                .unwrap();
                let server = &decode(format, &text)["pre-drink"];
                let request = AgentRequest {
                    client_id: server["env"]["PRE_DRINK_CLIENT_ID"]
                        .as_str()
                        .unwrap()
                        .into(),
                    token: server["env"]["PRE_DRINK_CLIENT_TOKEN"]
                        .as_str()
                        .unwrap()
                        .into(),
                    tool: "search_conversations".into(),
                    query: "garden".into(),
                    limit: None,
                    conversation_id: String::new(),
                    curation: None,
                };
                assert_eq!(vault.clients().unwrap()[0].allow_chats, allow_chats);
                let scoped = vault.clients().unwrap().remove(0);
                assert_eq!(
                    scoped.provider.as_deref(),
                    if allow_chats {
                        Some(match format {
                            ClientFormat::Grok => "grok",
                            ClientFormat::Claude => "claude",
                            _ => "chatgpt",
                        })
                    } else {
                        None
                    }
                );
                assert!(!scoped.can_curate);
                if allow_chats {
                    assert_eq!(vault.agent_call(&request).unwrap(), json!([]));
                } else {
                    assert!(vault
                        .agent_call(&request)
                        .unwrap_err()
                        .contains("memory only"));
                }
                let facts = AgentRequest {
                    tool: "get_facts".into(),
                    ..request
                };
                assert_eq!(vault.agent_call(&facts).unwrap(), json!([]));
                vault.revoke_client(&facts.client_id).unwrap();
                assert!(vault.agent_call(&facts).is_err());
            }
        }
    }

    #[test]
    fn setup_returns_the_exact_registered_identity_and_scope() {
        let vault = Vault::memory().unwrap();
        for (format, provider) in [
            (ClientFormat::Codex, "chatgpt"),
            (ClientFormat::Grok, "grok"),
            (ClientFormat::Claude, "claude"),
        ] {
            let setup = create_setup(
                &vault,
                "Fictional setup",
                true,
                true,
                format,
                Path::new("/app/pre-drink"),
                Path::new("/tmp/fictional/agents.sock"),
            )
            .unwrap();
            let decoded = decode(format, &setup.configuration);
            assert_eq!(
                decoded["pre-drink"]["env"]["PRE_DRINK_CLIENT_ID"],
                setup.client.id
            );
            assert_eq!(setup.client.provider.as_deref(), Some(provider));
            assert!(setup.client.allow_chats && setup.client.can_curate);
        }
    }

    #[test]
    fn unsupported_paths_roll_back_registration_and_unknown_formats_are_rejected() {
        let vault = Vault::memory().unwrap();
        for (format, command, socket) in [
            (
                ClientFormat::Generic,
                "relative/pre-drink",
                "/tmp/agents.sock",
            ),
            (ClientFormat::Codex, "/app/pre-drink\n", "/tmp/agents.sock"),
            (
                ClientFormat::Grok,
                "/app/${OTHER}/pre-drink",
                "/tmp/agents.sock",
            ),
            (
                ClientFormat::Grok,
                "/app/pre-drink",
                "/tmp/${OTHER}/agents.sock",
            ),
        ] {
            assert!(create_configuration(
                &vault,
                "Fictional client",
                false,
                format,
                Path::new(command),
                Path::new(socket)
            )
            .is_err());
            assert!(vault.clients().unwrap().is_empty());
        }
        assert!(serde_json::from_str::<ClientFormat>("\"unsupported\"").is_err());
    }
}
