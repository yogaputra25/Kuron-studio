//! Tulis file atomik + rotasi backup satu generasi.
//!
//! Skema: tulis `*.tmp` + `sync_all`, lalu `rename` ke nama akhir (`rename`
//! di satu filesystem itu atomik — crash tak pernah menyisakan file target
//! terpotong). Sebelum timpa, `main` yang masih valid diputar ke `*.bak`;
//! `main` yang korup TIDAK diputar (jangan hancurkan backup bagus dengan
//! data rusak). Sisa `*.tmp` dari crash diabaikan — tulis berikut menimpanya.
use std::fs;
use std::path::Path;

use crate::error::AppError;

/// Tulis `bytes` ke `path` secara atomik. `dir` dibuat bila belum ada.
/// Setiap langkah menyebut operasi + path bila gagal (tanpa ini error Windows
/// seperti "Access is denied" tak bisa dilacak sumbernya).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let dir = path
        .parent()
        .ok_or_else(|| AppError::io(format!("tanpa parent: {}", path.display())))?;
    fs::create_dir_all(dir)
        .map_err(|e| AppError::io(format!("buat dir {}: {e}", dir.display())))?;
    let tmp = tmp_file(path);
    fs::write(&tmp, bytes)
        .map_err(|e| AppError::io(format!("tulis tmp {}: {e}", tmp.display())))?;
    // sync_all = FlushFileBuffers: butuh handle WRITE. File::open (read-only)
    // selalu Access-denied di Windows — buka read+write khusus untuk fsync.
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&tmp)
        .and_then(|f| f.sync_all())
        .map_err(|e| AppError::io(format!("sinkron {}: {e}", tmp.display())))?;
    rename_with_retry(&tmp, path).map_err(|e| {
        AppError::io(format!(
            "simpan {} (rename {} -> {}): {e}; bila menetap, periksa antivirus/sync yang mengunci {}",
            path.display(),
            tmp.display(),
            path.display(),
            path.display()
        ))
    })?;
    // fsync dir supaya entri rename benar-benar mendarat di disk (Unix).
    // Windows: FlushFileBuffers butuh handle direktori khusus dan tidak pernah
    // jalan di sini — rename NTFS sudah atomik. Kode lama (File::open read-only)
    // selalu Access-denied di Windows, jadi gate ini tanpa kehilangan nyata.
    #[cfg(not(windows))]
    fs::File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| AppError::io(format!("sinkron dir {}: {e}", dir.display())))?;
    Ok(())
}

/// Jeda antar percobaan rename (mdetik). 5x percobaan; total terburuk 1,55 dtk.
/// Happy path (percobaan pertama sukses): tanpa sleep sama sekali.
const RENAME_WAIT_MS: [u64; 5] = [50, 100, 200, 400, 800];

/// Lock efemeral (antivirus/indexer/sync menahan file sesaat). Di Windows
/// muncul sebagai PermissionDenied; di Unix `rename` atomik tak kena pola ini.
#[cfg(windows)]
fn is_ephemeral_lock(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::PermissionDenied
}
#[cfg(not(windows))]
fn is_ephemeral_lock(_e: &std::io::Error) -> bool {
    false
}

/// Rename dengan retry bounded khusus lock efemeral. Error non-lock langsung
/// pulang tanpa retry; 5x gagal lock → Err terakhir.
fn rename_with_retry(tmp: &Path, path: &Path) -> std::io::Result<()> {
    rename_with_retry_impl(
        &mut || fs::rename(tmp, path),
        &is_ephemeral_lock,
        &mut |ms| std::thread::sleep(std::time::Duration::from_millis(ms)),
    )
}

/// Inti teruji: `op`/`is_retryable`/`sleep` di-inject agar unit test bisa
/// mensimulasi lock tanpa filesystem khusus OS.
fn rename_with_retry_impl(
    op: &mut dyn FnMut() -> std::io::Result<()>,
    is_retryable: &dyn Fn(&std::io::Error) -> bool,
    sleep: &mut dyn FnMut(u64),
) -> std::io::Result<()> {
    let mut last: Option<std::io::Error> = None;
    for wait in RENAME_WAIT_MS {
        match op() {
            Ok(()) => return Ok(()),
            Err(e) if is_retryable(&e) => {
                sleep(wait);
                last = Some(e);
            }
            Err(e) => return Err(e),
        }
    }
    Err(last.expect("loop 5x tanpa sukses harus sisakan error"))
}

/// Putar `main` -> `*.bak`, tapi HANYA bila `main` masih parse sebagai JSON
/// valid. Dipanggil sebelum `write_atomic` menimpa `main`.
pub fn rotate_backup(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(path)?;
    if serde_json::from_str::<serde_json::Value>(&raw).is_err() {
        // Main korup (crash lama): biarkan *.bak apa adanya.
        return Ok(());
    }
    fs::rename(path, bak_file(path))?;
    Ok(())
}

/// Path `*.bak` untuk satu file data.
pub fn bak_file(path: &Path) -> std::path::PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".bak");
    std::path::PathBuf::from(s)
}

fn tmp_file(path: &Path) -> std::path::PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".tmp");
    std::path::PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("kuron-atomic-{tag}-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn roundtrip_dan_rotate() {
        let dir = temp_dir("rt");
        let f = dir.join("d.json");
        write_atomic(&f, br#"{"g":1}"#).unwrap();
        assert_eq!(fs::read(&f).unwrap(), br#"{"g":1}"#);
        // Belum ada bak sebelum rotasi pertama.
        rotate_backup(&f).unwrap();
        write_atomic(&f, br#"{"g":2}"#).unwrap();
        rotate_backup(&f).unwrap();
        write_atomic(&f, br#"{"g":3}"#).unwrap();
        assert_eq!(fs::read(&f).unwrap(), br#"{"g":3}"#);
        assert_eq!(fs::read(bak_file(&f)).unwrap(), br#"{"g":2}"#);
    }

    #[test]
    fn main_korup_tak_hancurkan_bak_bagus() {
        let dir = temp_dir("guard");
        let f = dir.join("d.json");
        fs::write(&f, r#"{"ok":true}"#).unwrap();
        rotate_backup(&f).unwrap(); // bak = JSON valid
        fs::write(&f, "{korup").unwrap(); // crash: main rusak
        rotate_backup(&f).unwrap(); // JANGAN timpa bak bagus
        let bak = fs::read_to_string(bak_file(&f)).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&bak).is_ok());
    }
    #[test]
    fn pesan_gagal_memuat_path() {
        let dir = temp_dir("ctx");
        let f = dir.join("d.json");
        // Jadikan target tmp sebuah DIREKTORI supaya tulis gagal di semua OS.
        std::fs::create_dir_all(dir.join("d.json.tmp")).unwrap();
        let e = write_atomic(&f, br#"{"g":1}"#).unwrap_err().to_string();
        assert!(e.contains("d.json"), "pesan harus menyebut path: {e}");
        assert!(e.contains("tulis tmp"), "pesan harus menyebut operasi: {e}");
    }

    fn denied(msg: &str) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, msg)
    }

    #[test]
    fn retry_lock_dua_kali_lalu_sukses() {
        let mut calls = 0;
        let mut sleeps = Vec::new();
        let mut op = || {
            calls += 1;
            if calls <= 2 {
                Err(denied("Access is denied. (os error 5)"))
            } else {
                Ok(())
            }
        };
        let r = rename_with_retry_impl(&mut op, &|_| true, &mut |ms| sleeps.push(ms));
        assert!(r.is_ok());
        assert_eq!(calls, 3);
        assert_eq!(sleeps, vec![50, 100]);
    }

    #[test]
    fn error_non_lock_tanpa_retry() {
        let mut calls = 0;
        let mut sleeps = Vec::new();
        let mut op = || {
            calls += 1;
            Err::<(), _>(denied("permanen"))
        };
        let r = rename_with_retry_impl(&mut op, &|_| false, &mut |ms| sleeps.push(ms));
        assert!(r.is_err());
        assert_eq!(calls, 1);
        assert!(sleeps.is_empty(), "happy/non-retry path tanpa sleep");
    }

    #[test]
    fn lock_menetap_menyerah_setelah_lima_kali() {
        let mut calls = 0;
        let mut sleeps = Vec::new();
        let mut op = || {
            calls += 1;
            Err::<(), _>(denied("Access is denied. (os error 5)"))
        };
        let e = rename_with_retry_impl(&mut op, &|_| true, &mut |ms| sleeps.push(ms))
            .unwrap_err()
            .to_string();
        assert_eq!(calls, 5);
        assert_eq!(sleeps, vec![50, 100, 200, 400, 800]);
        assert!(e.contains("os error 5"), "error terakhir diteruskan: {e}");
    }
}
