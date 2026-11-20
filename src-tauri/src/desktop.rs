use crate::{
    agents::AgentClient,
    client_config::{create_scoped_configuration, ClientFormat},
    db::Vault,
    import::read_export_zip,
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
    session_op(store.inner().clone(), |s| s.status()).await
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
        let (items, report) = read_export_zip(&PathBuf::from(path))?;
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
    clear_ids: Vec<String>,
    store: State<'_, Store>,
) -> Result<crate::privacy_recheck::PrivacyRecheck, String> {
    session_op(store.inner().clone(), move |s| {
        s.apply_privacy_recheck(&review_hash, &clear_ids)
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
async fn prepare_share(
    selection: crate::sharing::ShareSelection,
    store: State<'_, Store>,
) -> Result<crate::sharing::PreparedShare, String> {
    session_op(store.inner().clone(), move |s| s.prepare_share(&selection)).await
}
#[tauri::command]
async fn copy_share(
    request: crate::sharing::ShareRequest,
    store: State<'_, Store>,
) -> Result<(), String> {
    session_op(store.inner().clone(), move |s| {
        s.with_validated_share(&request, |markdown| {
            #[cfg(target_os = "macos")]
            {
                crate::sharing::copy_markdown(markdown)
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = markdown;
                Err("Clipboard export is currently supported on macOS only.".into())
            }
        })
    })
    .await
}
#[tauri::command]
async fn save_share(
    request: crate::sharing::ShareRequest,
    path: String,
    store: State<'_, Store>,
) -> Result<(), String> {
    session_op(store.inner().clone(), move |s| {
        s.with_validated_share(&request, |markdown| {
            crate::sharing::save_markdown(std::path::Path::new(&path), markdown)
        })
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
    can_curate: bool,
    format: ClientFormat,
    store: State<'_, Store>,
) -> Result<String, String> {
    session_op(store.inner().clone(), move |s| {
        let executable =
            std::env::current_exe().map_err(|_| "Cannot locate the Pre-Drink executable.")?;
        let socket = s.directory.join("agents.sock");
        let config = create_scoped_configuration(
            s.access()?,
            &name,
            allow_chats,
            can_curate,
            format,
            &executable,
            &socket,
        )?;
        s.touch();
        Ok(config)
    })
    .await
}
#[tauri::command]
async fn list_curation(
    provider: String,
    after: String,
    store: State<'_, Store>,
) -> Result<Vec<crate::curation::CurationNote>, String> {
    owner_op(store.inner().clone(), move |v| {
        v.curation_notes(&provider, &after, true)
    })
    .await
}
#[tauri::command]
async fn list_curation_jobs(
    provider: String,
    store: State<'_, Store>,
) -> Result<serde_json::Value, String> {
    owner_op(store.inner().clone(), move |v| {
        v.curate(
            &provider,
            "owner",
            &crate::curation::CurationOperation::Jobs {},
        )
    })
    .await
}
#[tauri::command]
async fn curation_policies(store: State<'_, Store>) -> Result<serde_json::Value, String> {
    owner_op(store.inner().clone(), |v| v.curation_policies()).await
}
#[tauri::command]
async fn set_curation_policy(
    provider: String,
    archive: bool,
    session: bool,
    store: State<'_, Store>,
) -> Result<(), String> {
    owner_op(store.inner().clone(), move |v| {
        v.set_curation_policy(&provider, archive, session)
    })
    .await
}
#[tauri::command]
async fn review_curation(
    ids: Vec<String>,
    store: State<'_, Store>,
) -> Result<crate::curation::CurationReview, String> {
    owner_op(store.inner().clone(), move |v| v.curation_review(&ids)).await
}
#[tauri::command]
async fn publish_curation(
    ids: Vec<String>,
    review_hash: String,
    store: State<'_, Store>,
) -> Result<(), String> {
    owner_op(store.inner().clone(), move |v| {
        v.publish_curation_review(&ids, &review_hash)
    })
    .await
}
#[tauri::command]
async fn edit_curation(
    id: String,
    revision: i64,
    content: String,
    conflict: bool,
    store: State<'_, Store>,
) -> Result<serde_json::Value, String> {
    owner_op(store.inner().clone(), move |v| {
        v.edit_curation(&id, revision, &content, conflict)
    })
    .await
}
#[tauri::command]
async fn curation_history(
    id: String,
    store: State<'_, Store>,
) -> Result<Vec<crate::curation::CurationNote>, String> {
    owner_op(store.inner().clone(), move |v| v.curation_history(&id)).await
}
#[tauri::command]
async fn withdraw_curation(
    id: String,
    revision: i64,
    store: State<'_, Store>,
) -> Result<(), String> {
    owner_op(store.inner().clone(), move |v| {
        v.withdraw_curation(&id, revision)
    })
    .await
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
    session_op(store.inner().clone(), move |s| {
        s.backup_with_receipt(&PathBuf::from(path), &password)
    })
    .await
}

#[tauri::command]
async fn preview_retained_vaults(
    store: State<'_, Store>,
) -> Result<crate::retained_vaults::RetainedReview, String> {
    session_op(store.inner().clone(), |s| s.preview_retained_vaults()).await
}
#[tauri::command]
async fn remove_retained_vault(
    id: String,
    review_hash: String,
    confirmation: String,
    store: State<'_, Store>,
) -> Result<(), String> {
    session_op(store.inner().clone(), move |s| {
        s.remove_retained_vault(&id, &review_hash, &confirmation)
    })
    .await
}

#[tauri::command]
async fn prepare_restore(
    path: String,
    password: String,
    store: State<'_, Store>,
) -> Result<crate::lifecycle::LifecyclePreview, String> {
    let password = Zeroizing::new(password);
    session_op(store.inner().clone(), move |s| {
        s.prepare_restore(&PathBuf::from(path), &password)
    })
    .await
}
#[tauri::command]
async fn prepare_password_change(
    current_password: String,
    new_password: String,
    store: State<'_, Store>,
) -> Result<crate::lifecycle::LifecyclePreview, String> {
    let current = Zeroizing::new(current_password);
    let next = Zeroizing::new(new_password);
    session_op(store.inner().clone(), move |s| {
        s.prepare_password_change(&current, &next)
    })
    .await
}
#[tauri::command]
async fn apply_replacement(
    review_id: String,
    password: String,
    store: State<'_, Store>,
) -> Result<crate::lifecycle::LifecycleResult, String> {
    let password = Zeroizing::new(password);
    session_op(store.inner().clone(), move |s| {
        s.apply_replacement(&review_id, &password)
    })
    .await
}
#[tauri::command]
async fn cancel_replacement(review_id: String, store: State<'_, Store>) -> Result<(), String> {
    session_op(store.inner().clone(), move |s| {
        s.cancel_prepared(&review_id);
        Ok(())
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

#[tauri::command]
async fn preview_removal(
    input: crate::retention::RemovalInput,
    store: State<'_, Store>,
) -> Result<crate::retention::RemovalPreview, String> {
    session_op(store.inner().clone(), move |s| s.preview_removal(&input)).await
}
#[tauri::command]
async fn apply_removal(
    input: crate::retention::RemovalInput,
    review_hash: String,
    session_generation: u64,
    store: State<'_, Store>,
) -> Result<(), String> {
    session_op(store.inner().clone(), move |s| {
        s.apply_removal(&input, &review_hash, session_generation)
    })
    .await
}

// Opt-in local debug timing only. Never record paths, errors, credentials or content.
fn startup_mark(stage: &'static str) {
    #[cfg(debug_assertions)]
    if std::env::var_os("PRE_DRINK_STARTUP_TRACE").is_some() {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let elapsed = START.get_or_init(std::time::Instant::now).elapsed();
        eprintln!("startup {}ms {stage}", elapsed.as_millis());
    }
    #[cfg(not(debug_assertions))]
    let _ = stage;
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum StartupStage {
    ModuleReady,
    InterfaceReady,
}

#[tauri::command]
fn startup_stage(stage: StartupStage, startup: State<'_, StartupState>) {
    startup.0.store(true, std::sync::atomic::Ordering::Release);
    startup_mark(match stage {
        StartupStage::ModuleReady => "module-ready",
        StartupStage::InterfaceReady => "interface-ready",
    });
}

#[derive(Default)]
struct StartupState(std::sync::atomic::AtomicBool);

fn startup_watchdog(app: tauri::AppHandle) {
    // A page-level fallback cannot run if WebKit never loads the document.
    // Only monitor initial startup, and never interrupt a working interface.
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(15));
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            if handle.state::<StartupState>().0.load(std::sync::atomic::Ordering::Acquire) {
                return;
            }
            use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
            let Some(window) = handle.get_webview_window("main") else {
                return;
            };
            let recovery = handle.clone();
            handle.dialog()
                .message("Pre-Drink’s interface has not finished loading. Try again, or close and reopen the app.")
                .parent(&window)
                .title("Pre-Drink could not finish starting")
                .buttons(MessageDialogButtons::OkCancelCustom("Try again".into(), "Close app".into()))
                .show(move |retry| {
                    if recovery.state::<StartupState>().0.load(std::sync::atomic::Ordering::Acquire) {
                        return;
                    }
                    if retry {
                        if let Some(window) = recovery.get_webview_window("main") {
                            let _ = window.reload();
                            startup_watchdog(recovery.clone());
                        }
                    } else {
                        recovery.exit(0);
                    }
                });
        });
    });
}

pub fn run() {
    startup_mark("process-start");
    tauri::Builder::default()
        .manage(StartupState::default())
        .plugin(tauri_plugin_dialog::init())
        .on_page_load(|webview, payload| {
            if matches!(payload.event(), tauri::webview::PageLoadEvent::Started) {
                // Reloading must not leave a previously unlocked session sharing.
                if let Some(store) = webview.try_state::<Store>() {
                    if let Ok(mut session) = store.lock() {
                        session.lock();
                    }
                }
            }
            startup_mark(match payload.event() {
                tauri::webview::PageLoadEvent::Started => "page-start",
                tauri::webview::PageLoadEvent::Finished => "page-finish",
            });
        })
        .setup(|app| {
            startup_mark("setup-start");
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
            crate::mac_lock::register(store.clone()).map_err(std::io::Error::other)?;
            crate::broker::serve(&data.join("agents.sock"), store.clone())
                .map_err(std::io::Error::other)?;
            crate::session::watchdog(store.clone());
            app.manage(store);
            startup_watchdog(app.handle().clone());
            startup_mark("setup-finish");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            startup_stage,
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
            prepare_share,
            copy_share,
            save_share,
            list_clients,
            list_curation,
            list_curation_jobs,
            curation_policies,
            set_curation_policy,
            review_curation,
            publish_curation,
            withdraw_curation,
            edit_curation,
            curation_history,
            create_client,
            revoke_client,
            set_sharing,
            backup_vault,
            preview_retained_vaults,
            remove_retained_vault,
            prepare_restore,
            prepare_password_change,
            apply_replacement,
            cancel_replacement,
            preview_removal,
            apply_removal,
            list_briefs,
            save_brief,
            delete_brief
        ])
        .run(tauri::generate_context!())
        .expect("Could not start Pre-Drink");
}
