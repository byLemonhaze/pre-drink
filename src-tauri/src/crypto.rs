//! Vault format 1: SQLCipher 4 pages, Argon2id v19 (64 MiB, 3 passes, 1 lane).
//! The public 16-byte SQLCipher salt is also the Argon2 salt. No key file exists.
use argon2::{Algorithm, Argon2, Params, Version};
use rusqlite::Connection;
use std::{fs::File, io::Read, path::Path};
use zeroize::Zeroizing;

pub const PASSWORD_MIN: usize = 14;

pub fn validate_password(password: &str) -> Result<(), String> {
    if password.chars().count() < PASSWORD_MIN || password.len() > 1024 {
        return Err(
            "Use a vault password of at least 14 characters (a long, unique passphrase is best)."
                .into(),
        );
    }
    Ok(())
}

pub fn salt(path: &Path) -> Result<[u8; 16], String> {
    let mut salt = [0u8; 16];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut salt))
        .map_err(|_| "The encrypted vault is missing or incomplete. Restore your backup.")?;
    if &salt == b"SQLite format 3\0" {
        return Err("This file is unencrypted. Use the migration flow.".into());
    }
    Ok(salt)
}

pub fn new_salt() -> Result<[u8; 16], String> {
    let mut salt = [0; 16];
    getrandom::fill(&mut salt).map_err(|_| "Secure randomness is unavailable.")?;
    Ok(salt)
}

pub fn key(password: &str, salt: &[u8; 16]) -> Result<Zeroizing<String>, String> {
    let mut output = Zeroizing::new([0u8; 32]);
    let params =
        Params::new(65536, 3, 1, Some(32)).map_err(|_| "Invalid key derivation settings.")?;
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, &mut *output)
        .map_err(|_| "Couldn't derive the encryption key.")?;
    let mut raw = Zeroizing::new([0u8; 48]);
    raw[..32].copy_from_slice(&*output);
    raw[32..].copy_from_slice(salt);
    let mut encoded = Zeroizing::new(String::from("x'"));
    use std::fmt::Write;
    for byte in raw.iter() {
        write!(&mut *encoded, "{byte:02x}").map_err(|_| "Couldn't encode the key.")?;
    }
    encoded.push('\'');
    Ok(encoded)
}

pub fn configure(connection: &Connection, key: &str) -> Result<(), String> {
    let version: String = connection
        .query_row("PRAGMA cipher_version", [], |r| r.get(0))
        .map_err(|_| "Encryption support is missing. The vault will not open.")?;
    if !version.starts_with("4.") {
        return Err("Unsupported SQLCipher version; refusing to open the vault.".into());
    }
    connection
        .pragma_update(None, "key", key)
        .map_err(|_| "Couldn't initialize encryption.")?;
    connection
        .execute_batch(
            "PRAGMA cipher_memory_security=ON; PRAGMA temp_store=MEMORY; PRAGMA secure_delete=ON;",
        )
        .map_err(|_| "Couldn't configure secure storage.")?;
    connection
        .query_row("SELECT count(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        })
        .map_err(|_| "Incorrect password, or the vault is damaged. Nothing was changed.")?;
    Ok(())
}
