use crate::session::Store;
use block2::RcBlock;
use objc2_app_kit::{
    NSWorkspace, NSWorkspaceScreensDidSleepNotification,
    NSWorkspaceSessionDidResignActiveNotification, NSWorkspaceWillSleepNotification,
};
use objc2_core_foundation::{
    CFDictionary, CFNotificationCenter, CFNotificationName, CFNotificationSuspensionBehavior,
    CFString,
};
use objc2_foundation::NSNotification;
use std::{ffi::c_void, ptr::NonNull};

// This system notification name is used by macOS and Chromium, but is not an
// Apple-documented compatibility guarantee. Retain workspace/idle/gap locking
// and validate actual screen-lock delivery on each supported macOS release.
const SCREEN_LOCKED: &str = "com.apple.screenIsLocked";

unsafe extern "C-unwind" fn screen_locked(
    _center: *mut CFNotificationCenter,
    observer: *mut c_void,
    _name: *const CFNotificationName,
    _object: *const c_void,
    _info: *const CFDictionary,
) {
    // Registered below with a process-lifetime Store. Ignore all notification
    // payloads: another process can spoof this signal, but can only lock us.
    if let Some(store) = unsafe { observer.cast::<Store>().as_ref() } {
        if let Ok(mut session) = store.lock() {
            session.lock();
        }
    }
}

fn observe_screen_lock(center: &CFNotificationCenter, store: Store) {
    let observer: &'static Store = Box::leak(Box::new(store));
    let name = CFString::from_static_str(SCREEN_LOCKED);
    // The pointer remains valid for the registration's process lifetime.
    // Request immediate delivery even while the app is suspended for delivery
    // purposes; a native Save panel must not defer invalidating export reviews.
    unsafe {
        center.add_observer(
            (observer as *const Store).cast(),
            Some(screen_locked),
            Some(&name),
            std::ptr::null(),
            CFNotificationSuspensionBehavior::DeliverImmediately,
        );
    }
}

pub fn register(store: Store) -> Result<(), String> {
    let distributed = CFNotificationCenter::distributed_center()
        .ok_or("Cannot initialize Mac screen-lock monitoring.")?;
    observe_screen_lock(&distributed, store.clone());
    let center = NSWorkspace::sharedWorkspace().notificationCenter();
    // Apple's workspace notifications are process-lifetime observers. The callback
    // is Send + Sync (only an Arc<Mutex<Session>> is captured), with no raw pointers retained.
    for name in unsafe {
        [
            NSWorkspaceWillSleepNotification,
            NSWorkspaceScreensDidSleepNotification,
            NSWorkspaceSessionDidResignActiveNotification,
        ]
    } {
        let store = store.clone();
        let callback = RcBlock::new(move |_: NonNull<NSNotification>| {
            if let Ok(mut session) = store.lock() {
                session.lock();
            }
        });
        let observer = unsafe {
            center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &callback)
        };
        // Keep these three registrations for the app's entire process lifetime.
        std::mem::forget(observer);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::Vault,
        memory::MemoryInput,
        session::Session,
        sharing::{ShareItem, ShareKind, ShareRequest, ShareSelection},
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };

    #[test]
    fn notification_locks_session_and_invalidates_pending_export_without_unlocking() {
        let mut vault = Vault::memory().unwrap();
        let id = vault
            .save_memory(MemoryInput {
                id: None,
                kind: "fact".into(),
                content: "Fictional cedar garden.".into(),
                source_message_id: None,
                private: false,
            })
            .unwrap();
        let review = vault.approval_review("memory", &id).unwrap();
        vault
            .approve_review("memory", &id, &review.review_hash)
            .unwrap();
        let mut session = Session::new(std::path::PathBuf::new());
        session.vault = Some(vault);
        session.start_agent_session("until_lock").unwrap();
        let selection = ShareSelection {
            items: vec![ShareItem {
                kind: ShareKind::Memory,
                id,
            }],
        };
        let preview = session.prepare_share(&selection).unwrap();
        let request = ShareRequest {
            selection,
            review_hash: preview.preview.review_hash,
            session_generation: preview.session_generation,
        };
        let calls = Arc::new(AtomicUsize::new(0));
        let callback_calls = calls.clone();
        session.on_lock = Some(Box::new(move || {
            callback_calls.fetch_add(1, Ordering::SeqCst);
        }));
        let store = Arc::new(Mutex::new(session));
        // A process-local center exercises the registered native callback without
        // broadcasting fake system lock events to other applications.
        let center = CFNotificationCenter::local_center().unwrap();
        observe_screen_lock(&center, store.clone());
        let post = |name| unsafe {
            center.post_notification(
                Some(&CFString::from_static_str(name)),
                std::ptr::null(),
                None,
                true,
            );
        };
        post("com.apple.screenIsUnlocked");
        assert!(store.lock().unwrap().sharing);
        post(SCREEN_LOCKED);
        {
            let mut session = store.lock().unwrap();
            assert!(session.vault.is_none());
            assert!(!session.sharing);
            assert!(session
                .with_validated_share(&request, |_| -> Result<(), String> {
                    panic!("a pre-lock export must not reach its writer")
                })
                .is_err());
        }
        post("com.apple.screenIsUnlocked");
        post(SCREEN_LOCKED);
        assert!(store.lock().unwrap().vault.is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
