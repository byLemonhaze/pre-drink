use crate::{
    agents::{AgentClient, NewClient},
    db::Vault,
    import::read_chatgpt_zip,
    memory::{Memory, MemoryInput},
    model::{Conversation, ImportReport, Message, Stats},
    session::{Session, Store, VaultStatus},
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{Emitter, Manager, State};
use zeroize::Zeroizing;

async fn session_op<T: Send + 'static>(
    store: Store,
    f: impl FnOnce(&mut Session) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = store.lock().map_err(|_| "Vault unavailable.")?;
        f(&mut session)
    })
    .await
    .map_err(|_| "The vault operation failed.")?
}
async fn owner_op<T: Send + 'static>(
    store: Store,
    f: impl FnOnce(&mut Vault) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    session_op(store, move |s| {
        let result = f(s.access()?);
        s.touch();
        result
    })
    .await
}
#[tauri::command]
async fn vault_status(store: State<'_, Store>) -> Result<VaultStatus, String> {
    session_op(store.inner().clone(), |s| Ok(s.status())).await
}
#[tauri::command]
async fn unlock_vault(
    password: String,
    create: bool,
    remove_legacy: bool,
    store: State<'_, Store>,
) -> Result<(), String> {
    let password = Zeroizing::new(password);
    session_op(store.inner().clone(), move |s| {
        s.unlock(&password, create, remove_legacy)
    })
    .await
}
#[tauri::command]
async fn lock_vault(store: State<'_, Store>) -> Result<(), String> {
    session_op(store.inner().clone(), |s| {
        s.lock();
        Ok(())
    })
    .await
}
#[tauri::command]
async fn touch_vault(store: State<'_, Store>) -> Result<(), String> {
    session_op(store.inner().clone(), |s| {
        s.access()?;
        s.touch();
        Ok(())
    })
    .await
}
#[tauri::command]
async fn import_zip(path: String, store: State<'_, Store>) -> Result<ImportReport, String> {
    let store = store.inner().clone();
    let generation = session_op(store.clone(), |s| {
        s.access()?;
        Ok(s.generation)
    })
    .await?;
    tauri::async_runtime::spawn_blocking(move || {
        let (items, report) = read_chatgpt_zip(&PathBuf::from(path))?;
        let mut session = store.lock().map_err(|_| "Vault unavailable.")?;
        if session.generation != generation {
            return Err("Vault locked during import. Nothing was imported.".into());
        }
        let result = session.access()?.import(&items, report);
        session.touch();
        result
    })
    .await
    .map_err(|_| "Import could not finish.")?
}
#[tauri::command]
async fn list_conversations(
    query: String,
    include_restricted: bool,
    offset: u32,
    store: State<'_, Store>,
) -> Result<Vec<Conversation>, String> {
    owner_op(store.inner().clone(), move |v| {
        v.list(&query, include_restricted, offset)
    })
    .await
}
#[tauri::command]
async fn get_messages(
    id: String,
    include_restricted: bool,
    store: State<'_, Store>,
) -> Result<Vec<Message>, String> {
    owner_op(store.inner().clone(), move |v| {
        v.messages(&id, include_restricted)
    })
    .await
}
#[tauri::command]
async fn mark_private(id: String, private: bool, store: State<'_, Store>) -> Result<(), String> {
    owner_op(store.inner().clone(), move |v| v.mark_private(&id, private)).await
}
#[tauri::command]
async fn get_stats(store: State<'_, Store>) -> Result<Stats, String> {
    owner_op(store.inner().clone(), |v| v.stats()).await
}
#[tauri::command]
async fn list_memory(store: State<'_, Store>) -> Result<Vec<Memory>, String> {
    owner_op(store.inner().clone(), |v| v.memories(None, "", false)).await
}
#[tauri::command]
async fn save_memory(input: MemoryInput, store: State<'_, Store>) -> Result<String, String> {
    owner_op(store.inner().clone(), move |v| v.save_memory(input)).await
}
#[tauri::command]
async fn approval_review(
    kind: String,
    id: String,
    store: State<'_, Store>,
) -> Result<crate::review::ApprovalReview, String> {
    owner_op(store.inner().clone(), move |v| {
        v.approval_review(&kind, &id)
    })
    .await
}
#[tauri::command]
async fn approve_review(
    kind: String,
    id: String,
    review_hash: String,
    store: State<'_, Store>,
) -> Result<(), String> {
    owner_op(store.inner().clone(), move |v| {
        v.approve_review(&kind, &id, &review_hash)
    })
    .await
}
#[tauri::command]
async fn preview_privacy_recheck(
    store: State<'_, Store>,
) -> Result<crate::privacy_recheck::PrivacyRecheck, String> {
    owner_op(store.inner().clone(), |v| v.privacy_recheck_preview()).await
}
#[tauri::command]
async fn apply_privacy_recheck(
    review_hash: String,
    store: State<'_, Store>,
) -> Result<crate::privacy_recheck::PrivacyRecheck, String> {
    session_op(store.inner().clone(), move |s| {
        s.apply_privacy_recheck(&review_hash)
    })
    .await
}
#[tauri::command]
async fn delete_memory(id: String, store: State<'_, Store>) -> Result<(), String> {
    owner_op(store.inner().clone(), move |v| v.delete_memory(&id)).await
}
#[tauri::command]
async fn preview_context(topic: String, store: State<'_, Store>) -> Result<String, String> {
    owner_op(store.inner().clone(), move |v| {
        v.context_pack(&topic, false)
    })
    .await
}
#[tauri::command]
async fn list_clients(store: State<'_, Store>) -> Result<Vec<AgentClient>, String> {
    owner_op(store.inner().clone(), |v| v.clients()).await
}
#[tauri::command]
async fn create_client(
    name: String,
    allow_chats: bool,
    store: State<'_, Store>,
) -> Result<serde_json::Value, String> {
    session_op(store.inner().clone(),move|s|{
        let NewClient{client,token}=s.access()?.create_client(&name,allow_chats)?;
        let config=serde_json::json!({"mcpServers":{"pre-drink":{"command":std::env::current_exe().map_err(|e|e.to_string())?,"args":["--mcp"],"env":{"PRE_DRINK_SOCKET":s.directory.join("agents.sock"),"PRE_DRINK_CLIENT_ID":client.id,"PRE_DRINK_CLIENT_TOKEN":token}}}});
        s.touch();Ok(config)
    }).await
}
#[tauri::command]
async fn revoke_client(id: String, store: State<'_, Store>) -> Result<(), String> {
    owner_op(store.inner().clone(), move |v| v.revoke_client(&id)).await
}
#[tauri::command]
async fn set_sharing(enabled: bool, store: State<'_, Store>) -> Result<(), String> {
    session_op(store.inner().clone(), move |s| {
        s.access()?;
        s.sharing = enabled;
        s.touch();
        Ok(())
    })
    .await
}
#[tauri::command]
async fn backup_vault(
    path: String,
    password: String,
    store: State<'_, Store>,
) -> Result<(), String> {
    let password = Zeroizing::new(password);
    owner_op(store.inner().clone(), move |v| {
        v.encrypted_backup(&PathBuf::from(path), &password)
    })
    .await
}

#[tauri::command]
async fn list_briefs(store: State<'_, Store>) -> Result<Vec<crate::briefs::Brief>, String> {
    owner_op(store.inner().clone(), |v| v.briefs(false, "")).await
}
#[tauri::command]
async fn save_brief(
    input: crate::briefs::BriefInput,
    store: State<'_, Store>,
) -> Result<String, String> {
    owner_op(store.inner().clone(), move |v| v.save_brief(input)).await
}
#[tauri::command]
async fn delete_brief(id: String, store: State<'_, Store>) -> Result<(), String> {
    owner_op(store.inner().clone(), move |v| v.delete_brief(&id)).await
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data = app.path().app_data_dir()?;
            #[cfg(debug_assertions)]
            let data = std::env::var_os("PRE_DRINK_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or(data);
            std::fs::create_dir_all(&data)?;
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o700))?;
            let instance = std::fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .mode(0o600)
                .open(data.join("instance.lock"))?;
            fs2::FileExt::try_lock_exclusive(&instance).map_err(|_| {
                std::io::Error::other("Pre-Drink is already running. Use its existing window.")
            })?;
            app.manage(instance);
            let mut session = Session::new(data.clone());
            session.isolated_storage =
                cfg!(debug_assertions) && std::env::var_os("PRE_DRINK_DATA_DIR").is_some();
            let handle = app.handle().clone();
            session.on_lock = Some(Box::new(move || {
                let _ = handle.emit("vault-locked", ());
            }));
            let store = Arc::new(Mutex::new(session));
            #[cfg(target_os = "macos")]
            crate::mac_lock::register(store.clone());
            crate::broker::serve(&data.join("agents.sock"), store.clone())
                .map_err(std::io::Error::other)?;
            crate::session::watchdog(store.clone());
            app.manage(store);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            vault_status,
            unlock_vault,
            lock_vault,
            touch_vault,
            import_zip,
            list_conversations,
            get_messages,
            mark_private,
            get_stats,
            list_memory,
            save_memory,
            approval_review,
            approve_review,
            preview_privacy_recheck,
            apply_privacy_recheck,
            delete_memory,
            preview_context,
            list_clients,
            create_client,
            revoke_client,
            set_sharing,
            backup_vault,
            list_briefs,
            save_brief,
            delete_brief
        ])
        .run(tauri::generate_context!())
        .expect("Could not start Pre-Drink");
}
