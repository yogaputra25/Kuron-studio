//! M4-1: OS keychain via `keyring` (service `id.kuron.studio`,
//! account `provider:{id}`) + sqlite fallback (M2 compat).
//!
//! Rules:
//! - `save_key` writes OS keychain; returns Err on failure (caller keeps
//!   sqlite fallback column empty so a stale key is never left behind).
//! - `read_key` tries keychain, falls back to sqlite (legacy rows).
//! - `delete_key` removes both (keychain NoEntry is fine).
//! - M4-2: never log the key anywhere in this module.

pub const SERVICE: &str = "id.kuron.studio";

pub fn account(provider_id: &str) -> String {
    format!("provider:{provider_id}")
}

fn entry(provider_id: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, &account(provider_id)).map_err(|e| e.to_string())
}

/// Read: keychain first, sqlite legacy column as fallback. Never logs key.
/// Never fails on backend absence: any keychain error → sqlite value.
pub fn read_key(
    provider_id: &str,
    sqlite_fallback: &str,
) -> Result<String, String> {
    let Ok(e) = entry(provider_id) else {
        return Ok(sqlite_fallback.to_string());
    };
    match e.get_password() {
        Ok(k) => Ok(k),
        Err(_) => Ok(sqlite_fallback.to_string()),
    }
}

/// Presence check for `has_key` badge: keychain hit wins, else sqlite.
pub fn has_key(provider_id: &str, sqlite_fallback: &str) -> bool {
    if let Ok(e) = entry(provider_id) {
        if let Ok(k) = e.get_password() {
            return !k.is_empty();
        }
    }
    !sqlite_fallback.is_empty()
}

/// Save: keychain write-through. Err → caller leaves sqlite empty.
pub fn save_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    entry(provider_id)?
        .set_password(api_key)
        .map_err(|e| e.to_string())
}

/// Delete both stores; missing keychain entry is not an error.
pub fn delete_key(
    conn: &rusqlite::Connection,
    provider_id: &str,
) -> Result<(), String> {
    if let Ok(e) = entry(provider_id) {
        match e.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    crate::cache::delete_provider(conn, provider_id)
}

/// M4-2: redacted view for logs/UI. Full key never appears; body previews
/// elsewhere already cap at 200 chars (`err_preview` in provider/*).
pub fn redacted(key: &str) -> &'static str {
    if key.is_empty() {
        "missing"
    } else {
        "stored:key"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_format() {
        assert_eq!(account("abc"), "provider:abc");
    }

    #[test]
    fn redacted_never_contains_key() {
        let k = "sk-live-super-secret-12345";
        let r = redacted(k);
        assert!(!r.contains(k));
        assert!(!r.contains("sk-live"));
        assert_eq!(redacted(""), "missing");
    }

    #[test]
    fn fallback_read_without_keychain_entry() {
        // Fresh id has no keychain entry → legacy sqlite value returned.
        let v = read_key("no-such-provider-m4-test", "legacy-k").unwrap();
        assert_eq!(v, "legacy-k");
    }

    /// Regresi: tanpa feature `apple-native`, crate `keyring` memakai store
    /// `mock` di macOS — `set_password` tetap `Ok(())` (jadi tidak ada error
    /// sama sekali), tapi tidak ada yang tersimpan. Gejalanya: API key hilang
    /// tiap restart app. Test ini memakai entry yang sama dua kali lewat
    /// `Entry::new` baru: mock selalu mengembalikan `NoEntry`.
    #[test]
    #[cfg(target_os = "macos")]
    fn keychain_backend_is_real() {
        let id = "kuron-studio-backend-probe";
        let _ = read_key(id, "").ok();
        let e = entry(id).expect("keychain entry harus bisa dibuat");
        e.set_password("probe-value")
            .expect("set_password harus berhasil (store sungguhan, bukan mock)");
        // Entry BARU ke store yang sama — mock tidak punya persistensi.
        let again = entry(id).unwrap().get_password();
        assert_eq!(
            again.as_deref().ok(),
            Some("probe-value"),
            "credential hilang saat entry dibuat ulang = store `mock`, \
             bukan macOS Keychain. Cek feature `apple-native` di Cargo.toml."
        );
        let _ = e.delete_credential();
    }
}
