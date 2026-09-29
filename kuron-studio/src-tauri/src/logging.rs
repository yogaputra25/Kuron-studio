//! Log terstruktur untuk debugging.
//!
//! Dua tujuan:
//! 1. Semua error invoke harus terlihat di console tanpa harus attach debugger.
//! 2. Isi log harus bisa dibaca ulang setelah app ditutup (bug Report terpisah
//!    dari terminal).
//!
//! Format JSON Lines: satu objek JSON per baris, jadi bisa di-grep, di-parse
//! jq, atau ditempel ke issue tracker apa adanya.
//!
//! Rahasia: `redact()` membilas field sensitif sebelum ditulis. API key tidak
//! boleh pernah sampai ke disk — bukan hanya di UI.

use std::io::Write;
use std::sync::Mutex;
use std::sync::OnceLock;

/// Field yang isinya tidak boleh masuk log dalam keadaan apa pun.
const SECRET_FIELDS: [&str; 6] = [
    "api_key", "apiKey", "key", "authorization", "bearer", "password",
];

/// Maksimal karakter per nilai sebelum dipotong (body HTTP bisa MB-an).
const MAX_VALUE: usize = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
        }
    }
}

static SINK: OnceLock<Mutex<Option<std::fs::File>>> = OnceLock::new();

/// Buka file log di app data dir. Gagal? Tidak apa-apa — log tetap ke stderr,
/// dan `path()` mengembalikan `None` supaya UI bisa jujur bilang "tidak aktif".
pub fn init(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let _ = std::fs::create_dir_all(dir);
    let path = dir.join("kuron-studio.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()?;
    if let Ok(mut slot) = SINK.get_or_init(|| Mutex::new(None)).lock() {
        *slot = Some(file);
    }
    Some(path)
}

/// `Some` kalau file log sempat dibuka. Path absolut disimpan terpisah supaya
/// `diagnostics` bisa melaporkannya.
pub fn is_active() -> bool {
    SINK.get().is_some_and(|s| s.lock().map(|g| g.is_some()).unwrap_or(false))
}

/// Nilai string yang aman untuk ditulis: dipotong + disensor.
pub fn redact(s: &str) -> String {
    let t = s.trim();
    if t.is_empty() {
        return String::new();
    }
    let mut out: String = t.chars().take(MAX_VALUE).collect();
    if t.chars().count() > MAX_VALUE {
        out.push('…');
    }
    out
}

/// Objek JSON apa pun → string, dengan field sensitif disensor.
pub fn redact_value(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, val) in map {
                if SECRET_FIELDS.iter().any(|s| s.eq_ignore_ascii_case(k)) {
                    out.insert(k.clone(), serde_json::Value::String(mask(val)));
                } else {
                    out.insert(k.clone(), redact_value(val));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(a) => {
            serde_json::Value::Array(a.iter().map(redact_value).collect())
        }
        serde_json::Value::String(s) => serde_json::Value::String(redact(s)),
        other => other.clone(),
    }
}

fn mask(v: &serde_json::Value) -> String {
    let empty = match v {
        serde_json::Value::String(s) => s.is_empty(),
        serde_json::Value::Null => true,
        _ => false,
    };
    if empty {
        "<empty>".to_string()
    } else {
        "<redacted>".to_string()
    }
}

fn stamp() -> String {
    // Waktu epoch milidetik — tanpa dependency chrono, dan cukup untuk
    // mengurutkan serta mengukur durasi invoke.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
        .to_string()
}

/// Tulis satu baris JSON ke stderr + file log.
pub fn log(level: Level, event: &str, fields: serde_json::Value) {
    let mut obj = match fields {
        serde_json::Value::Object(m) => m,
        other => {
            let mut m = serde_json::Map::new();
            m.insert("detail".into(), other);
            m
        }
    };
    obj.insert("ts".into(), serde_json::Value::String(stamp()));
    obj.insert("level".into(), serde_json::Value::String(level.as_str().into()));
    obj.insert("event".into(), serde_json::Value::String(event.into()));

    let line = serde_json::to_string(&serde_json::Value::Object(obj))
        .unwrap_or_else(|_| r#"{"level":"error","event":"log_encode_failed"}"#.into());

    // Selalu ke stderr: `pnpm tauri dev` menampilkannya di terminal yang sama.
    let _ = writeln!(std::io::stderr(), "{line}");

    if let Some(slot) = SINK.get() {
        if let Ok(mut guard) = slot.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = writeln!(file, "{line}");
                let _ = file.flush();
            }
        }
    }
}

pub fn debug(event: &str, fields: serde_json::Value) {
    log(Level::Debug, event, fields);
}
pub fn info(event: &str, fields: serde_json::Value) {
    log(Level::Info, event, fields);
}
pub fn warn(event: &str, fields: serde_json::Value) {
    log(Level::Warn, event, fields);
}
pub fn error(event: &str, fields: serde_json::Value) {
    log(Level::Error, event, fields);
}

/// Hasil invoke: selalu catat sukses maupun gagal, dengan durasi.
pub fn invoke_result(
    command: &str,
    args: &serde_json::Value,
    outcome: Result<serde_json::Value, String>,
    elapsed_ms: u128,
) -> serde_json::Value {
    let mut f = serde_json::Map::new();
    f.insert("command".into(), serde_json::Value::String(command.into()));
    f.insert("args".into(), redact_value(args));
    f.insert("elapsed_ms".into(), serde_json::Value::from(elapsed_ms as u64));
    match &outcome {
        Ok(v) => {
            f.insert("ok".into(), serde_json::Value::Bool(true));
            // Sukses tetap diringkas supaya tidak membanjiri log dengan
            // base64 thumbnail yang besar.
            let preview = match v {
                serde_json::Value::String(s) => format!("<string len={}>", s.len()),
                other => {
                    let s = serde_json::to_string(other).unwrap_or_default();
                    format!("<{} len={}>", type_name(other), s.len())
                }
            };
            f.insert("result".into(), serde_json::Value::String(preview));
            info("invoke_ok", serde_json::Value::Object(f));
            v.clone()
        }
        Err(e) => {
            f.insert("ok".into(), serde_json::Value::Bool(false));
            f.insert("error".into(), serde_json::Value::String(redact(e)));
            error("invoke_err", serde_json::Value::Object(f));
            serde_json::Value::String(e.clone())
        }
    }
}

fn type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Info environment untuk bug report. Tidak ada secret di sini.
pub fn diagnostics() -> serde_json::Value {
    serde_json::json!({
        "app_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "log_active": is_active(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_secret_fields() {
        let v = serde_json::json!({
            "name": "local",
            "api_key": "sk-super-secret",
            "apiKey": "sk-also-secret",
            "nested": {"key": "hunter2", "model": "muse"},
        });
        let out = redact_value(&v);
        let s = serde_json::to_string(&out).unwrap();
        assert!(!s.contains("super-secret"), "api key bocor: {s}");
        assert!(!s.contains("also-secret"), "apiKey bocor: {s}");
        assert!(!s.contains("hunter2"), "nested key bocor: {s}");
        // Field non-rahasia harus utuh.
        assert!(s.contains("local"));
        assert!(s.contains("muse"));
    }

    #[test]
    fn empty_secret_marked_not_redacted() {
        let v = serde_json::json!({"api_key": ""});
        let s = serde_json::to_string(&redact_value(&v)).unwrap();
        assert!(s.contains("<empty>"));
    }

    #[test]
    fn redact_truncates_long_values() {
        let long = "x".repeat(1000);
        let out = redact(&long);
        assert!(out.chars().count() <= MAX_VALUE + 1, "tidak dipotong");
        assert!(out.ends_with('…'));
    }

    #[test]
    fn every_secret_field_name_is_covered() {
        for f in SECRET_FIELDS {
            let v = serde_json::json!({ f: "rahasia" });
            let s = serde_json::to_string(&redact_value(&v)).unwrap();
            assert!(!s.contains("rahasia"), "field `{f}` bocor: {s}");
        }
    }

    #[test]
    fn invoke_result_logs_both_outcomes() {
        let ok = invoke_result(
            "list_projects",
            &serde_json::json!({}),
            Ok(serde_json::json!([])),
            3,
        );
        assert!(ok.is_array());
        let err = invoke_result(
            "save_provider",
            &serde_json::json!({"name": "x", "api_key": "sk-secret"}),
            Err("boom".into()),
            5,
        );
        assert_eq!(err.as_str(), Some("boom"));
    }

    #[test]
    fn json_line_is_valid() {
        // Baris log harus bisa di-parse oleh jq tanpa intervensi.
        let f = serde_json::json!({"command": "test", "ok": false});
        let line = serde_json::to_string(&f).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed["command"], "test");
    }
}
