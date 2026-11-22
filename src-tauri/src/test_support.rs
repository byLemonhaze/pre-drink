//! Shared test-only process synchronization; production KDF settings stay intact.
use std::{
    path::Path,
    process::Child,
    time::{Duration, Instant},
};

pub(crate) fn wait_for_checkpoint(child: &mut Child, ready: &Path) {
    // Wait for the actual fault-boundary signal, never an assumed sleep period.
    // Loaded machines can spend longer deriving keys. This is only a deadlock
    // watchdog; a child exit fails immediately and readiness returns immediately.
    let deadline = Instant::now() + Duration::from_secs(300);
    while !ready.exists() {
        assert!(
            child.try_wait().unwrap().is_none(),
            "Fictional child exited before its fault-boundary signal"
        );
        assert!(
            Instant::now() < deadline,
            "Fictional child missed its fault-boundary signal within 300 seconds"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
