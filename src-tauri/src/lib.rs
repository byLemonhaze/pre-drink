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
pub mod client_config;
pub mod curation;
mod curation_jobs;
#[cfg(unix)]
pub mod mcp;
#[cfg(unix)]
mod peer;
pub mod session;
pub mod sharing;

#[cfg(feature = "desktop")]
mod desktop;
#[cfg(feature = "desktop")]
pub use desktop::run;

#[cfg(all(feature = "desktop", target_os = "macos"))]
mod mac_lock;

pub mod lifecycle;

pub mod retention;

pub mod retained_vaults;

#[cfg(test)]
mod storage_failure;

#[cfg(test)]
mod curation_tests;

mod provider_import;

pub mod reader;

pub mod library;
