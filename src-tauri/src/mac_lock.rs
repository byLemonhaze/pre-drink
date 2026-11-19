use crate::session::Store;
use block2::RcBlock;
use objc2_app_kit::{
    NSWorkspace, NSWorkspaceScreensDidSleepNotification,
    NSWorkspaceSessionDidResignActiveNotification, NSWorkspaceWillSleepNotification,
};
use objc2_foundation::NSNotification;
use std::ptr::NonNull;

pub fn register(store: Store) {
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
}
