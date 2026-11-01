//! Provider secrets terenkripsi file (pengganti OS keychain).
//!
//! Format kolom sqlite `api_key`: `enc1:base64(nonce12 || ciphertext)`
//! (AES-256-GCM). Kunci enkripsi (DEK 32-byte) di
//! `<app_data>/keys/master.key` permission `0600`.
//!
//! Rules:
//! - `encrypt_key` selalu menghasilkan `enc1:`; `decrypt_stored` membuka
//!   `enc1:` dan melewatkan plaintext legacy apa adanya (kompatibel bacaan
//!   baris lama tanpa migrasi skema).
//! - `read_key` TIDAK menyentuh keychain bila nilai sqlite sudah `enc1:`
//!   (path panas: `get_providers`, tiap translate). Keychain hanya dibaca
//!   untuk baris legacy — dan `migrate_provider_keys` (di
//!   `commands::provider`) memindahkan semuanya saat startup, jadi setelah
//!   migrasi tidak ada lagi dialog akses Keychain di macOS.
//! - `delete_key` menghapus sisa entri keychain pra-migrasi (best-effort).
//! - Kunci tidak pernah di-log di modul ini.

use std::path::{Path, PathBuf};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

use crate::error::AppError;

/// Prefix format ciphertext di kolom sqlite.
pub const ENC_PREFIX: &str = "enc1:";
const NONCE_LEN: usize = 12;

pub const SERVICE: &str = "id.kuron.studio";

pub fn account(provider_id: &str) -> String {
    format!("provider:{provider_id}")
}

fn entry(provider_id: &str) -> Result<keyring::Entry, AppError> {
    Ok(keyring::Entry::new(SERVICE, &account(provider_id))?)
}

fn master_path(data_dir: &Path) -> PathBuf {
    data_dir.join("keys").join("master.key")
}

/// Muat DEK, atau buat sekali (32 byte acak) dengan permission `0600`.
fn master_key(data_dir: &Path) -> Result<[u8; 32], AppError> {
    let p = master_path(data_dir);
    if let Ok(bytes) = std::fs::read(&p) {
        if bytes.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            return Ok(key);
        }
        return Err(AppError::store(format!(
            "master key corrupt ({} bytes): {}",
            bytes.len(),
            p.display()
        )));
    }
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, key)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(key)
}

/// Enkripsi API key plaintext → `enc1:...` siap simpan ke sqlite.
pub fn encrypt_key(data_dir: &Path, plaintext: &str) -> Result<String, AppError> {
    let key = master_key(data_dir)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::store(e.to_string()))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::getrandom(&mut nonce_bytes)?;
    let ct = cipher.encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_bytes())?;
    let mut blob = Vec::with_capacity(NONCE_LEN + ct.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ct);
    Ok(format!("{ENC_PREFIX}{}", B64.encode(blob)))
}

/// Buka nilai kolom sqlite: `enc1:` → dekripsi; selain itu plaintext legacy.
pub fn decrypt_stored(data_dir: &Path, stored: &str) -> Result<String, AppError> {
    let Some(b64) = stored.strip_prefix(ENC_PREFIX) else {
        return Ok(stored.to_string());
    };
    let blob = B64.decode(b64)?;
    if blob.len() < NONCE_LEN {
        return Err(AppError::store("ciphertext provider key terlalu pendek"));
    }
    let key = master_key(data_dir)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::store(e.to_string()))?;
    let pt = cipher.decrypt(
        Nonce::from_slice(&blob[..NONCE_LEN]),
        &blob[NONCE_LEN..],
    )?;
    String::from_utf8(pt).map_err(|e| AppError::store(e.to_string()))
}

/// Read: `enc1:` langsung dekripsi (TANPA sentuh keychain → tanpa popup
/// macOS). Baris legacy: keychain dulu (sisa pra-migrasi), lalu sqlite.
pub fn read_key(
    data_dir: &Path,
    provider_id: &str,
    sqlite_value: &str,
) -> Result<String, AppError> {
    if sqlite_value.starts_with(ENC_PREFIX) {
        return decrypt_stored(data_dir, sqlite_value);
    }
    if let Some(k) = read_keychain_only(provider_id) {
        return Ok(k);
    }
    Ok(sqlite_value.to_string())
}

/// Presence check untuk badge `has_key`: `enc1:` → coba dekripsi; legacy →
/// keychain dulu, lalu sqlite. Tidak pernah me-log key.
pub fn has_key(data_dir: &Path, provider_id: &str, sqlite_value: &str) -> bool {
    if sqlite_value.starts_with(ENC_PREFIX) {
        return decrypt_stored(data_dir, sqlite_value)
            .map(|k| !k.is_empty())
            .unwrap_or(false);
    }
    if read_keychain_only(provider_id).is_some() {
        return true;
    }
    !sqlite_value.is_empty()
}

/// Baca keychain saja (jalur migrasi). `None` = tidak ada / tak bisa dibaca.
/// Setiap pemanggilan ini di macOS berpotensi memicu dialog akses Keychain —
/// hanya dipakai untuk baris legacy, bukan path panas.
pub fn read_keychain_only(provider_id: &str) -> Option<String> {
    let e = entry(provider_id).ok()?;
    let k = e.get_password().ok()?;
    if k.is_empty() {
        None
    } else {
        Some(k)
    }
}

/// Delete sisa entri keychain pra-migrasi — tidak menyentuh DB (baris sqlite
/// dihapus caller via repo, seperti sebelumnya).
pub fn delete_key(provider_id: &str) -> Result<(), AppError> {
    if let Ok(e) = entry(provider_id) {
        match e.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(AppError::from(e)),
        }
    }
    Ok(())
}

/// M4-2: redacted view untuk test. Full key tidak pernah muncul. Hanya dipakai
/// test — tidak ada pemanggil produksi.
#[cfg(test)]
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

    fn tmpdir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("kuron-secrets-test-{name}"));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn account_format() {
        assert_eq!(account("abc"), "provider:abc");
    }

    #[test]
    fn roundtrip_enc1() {
        let d = tmpdir("roundtrip");
        let c = encrypt_key(&d, "sk-live-123").unwrap();
        assert!(c.starts_with(ENC_PREFIX));
        assert!(!c.contains("sk-live-123"));
        assert_eq!(decrypt_stored(&d, &c).unwrap(), "sk-live-123");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn tamper_terdeteksi_gcm_tag() {
        let d = tmpdir("tamper");
        let mut c = encrypt_key(&d, "secret").unwrap();
        let last = c.pop().unwrap();
        c.push(if last == 'A' { 'B' } else { 'A' });
        assert!(decrypt_stored(&d, &c).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn legacy_plaintext_passthrough_tanpa_master_key() {
        let d = tmpdir("legacy");
        // Tidak ada file kunci dibuat — pure passthrough.
        assert_eq!(decrypt_stored(&d, "k-lama").unwrap(), "k-lama");
        assert!(!master_path(&d).exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    #[cfg(unix)]
    fn master_key_permission_0600() {
        use std::os::unix::fs::PermissionsExt;
        let d = tmpdir("perm");
        let _ = encrypt_key(&d, "x").unwrap();
        let mode = std::fs::metadata(master_path(&d))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn corrupt_master_key_ditolak_bukan_ditimpa() {
        // Regenerasi diam-diam akan meng-yatimkan semua baris enc1:.
        let d = tmpdir("corrupt");
        std::fs::create_dir_all(d.join("keys")).unwrap();
        std::fs::write(master_path(&d), b"pendek").unwrap();
        assert!(encrypt_key(&d, "x").is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn redacted_never_contains_key() {
        let k = "sk-live-super-secret-12345";
        let r = redacted(k);
        assert!(!r.contains(k));
        assert!(!r.contains("sk-live"));
        assert_eq!(redacted(""), "missing");
    }
}
