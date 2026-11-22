//! Isolated version-2 vault for the native upgrade-notice acceptance check.
#[allow(dead_code)]
#[path = "../tests/support/historical_vault.rs"]
mod historical;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{fs, os::unix::fs::PermissionsExt};
    let root = tempfile::Builder::new()
        .prefix("pd-native-historical-")
        .tempdir_in("/tmp")?;
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700))?;
    let live = root.path().join("live");
    fs::create_dir(&live)?;
    fs::set_permissions(&live, fs::Permissions::from_mode(0o700))?;
    let session = pre_drink::session::Session::new(live.clone());
    historical::create(&session.path()?, 2, false);
    let root = root.keep();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "fixture_root":root,"app_data_dir":live,"fictional_password":historical::PASSWORD,
            "expected_restrictions":2,"expected_approvals_reset":5
        }))?
    );
    Ok(())
}
