pub mod briefs;
pub mod crypto;
pub mod db;
pub mod import;
pub mod memory;
pub mod model;
pub mod privacy;
pub mod privacy_recheck;
pub mod review;

pub mod agents;
#[cfg(unix)]
pub mod broker;
#[cfg(unix)]
pub mod mcp;
pub mod session;
pub mod sharing;

#[cfg(feature = "desktop")]
mod desktop;
#[cfg(feature = "desktop")]
pub use desktop::run;

#[cfg(all(feature = "desktop", target_os = "macos"))]
mod mac_lock;
