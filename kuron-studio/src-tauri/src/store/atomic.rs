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
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let dir = path
        .parent()
        .ok_or_else(|| AppError::io(format!("tanpa parent: {}", path.display())))?;
    fs::create_dir_all(dir)?;
    let tmp = tmp_file(path);
    fs::write(&tmp, bytes)?;
    fs::File::open(&tmp).and_then(|f| f.sync_all())?;
    fs::rename(&tmp, path)?;
    // fsync dir supaya entri rename benar-benar mendarat di disk.
    fs::File::open(dir).and_then(|f| f.sync_all())?;
    Ok(())
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
}
