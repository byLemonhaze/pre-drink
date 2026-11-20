use std::{
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
fn git(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}
fn main() {
    // Include enough identity to distinguish an installed release from a local
    // development bundle; no credentials, local paths or author identity.
    for path in [
        "src",
        "../src",
        "../public",
        "../index.html",
        "../dist",
        "Cargo.toml",
        "Cargo.lock",
        "tauri.conf.json",
        "../package.json",
        "../package-lock.json",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    for name in ["HEAD", "refs/heads/main", "index"] {
        if let Some(path) = git(&["rev-parse", "--git-path", name]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    let revision =
        git(&["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| "unversioned".into());
    let dirty = git(&["status", "--porcelain"]).is_none_or(|s| !s.is_empty());
    println!(
        "cargo:rustc-env=PRE_DRINK_BUILD_REVISION={revision}{}",
        if dirty { "+local-changes" } else { "" }
    );
    println!(
        "cargo:rustc-env=PRE_DRINK_BUILT_AT={}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    #[cfg(feature = "desktop")]
    tauri_build::build()
}
