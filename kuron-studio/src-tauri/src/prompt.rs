use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TranslateStyle {
    Standard,
    Formal,
    Casual,
    Dramatic,
    Humorous,
    Literal,
    Concise,
}

impl TranslateStyle {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "formal" => Self::Formal,
            "casual" => Self::Casual,
            "dramatic" => Self::Dramatic,
            "humorous" => Self::Humorous,
            "literal" => Self::Literal,
            "concise" => Self::Concise,
            _ => Self::Standard,
        }
    }

    fn line(&self) -> &'static str {
        match self {
            Self::Standard => "Style: natural, faithful manga translation.",
            Self::Formal => "Style: formal and polite register.",
            Self::Casual => "Style: casual, conversational tone.",
            Self::Dramatic => "Style: dramatic, expressive tone.",
            Self::Humorous => "Style: humorous, playful tone.",
            Self::Literal => "Style: literal, word-for-word faithful.",
            Self::Concise => "Style: concise, minimal wording.",
        }
    }
}

pub fn sfx_rule(skip: bool) -> &'static str {
    if skip {
        "Skip sound-effect text; translate dialogue only."
    } else {
        "Translate sound effects too, marked naturally."
    }
}

pub fn build_mosaic_prompt(
    target_lang: &str,
    style: TranslateStyle,
    skip_sfx: bool,
    bubble_count: usize,
) -> String {
    format!(
        "You are translating manga speech bubbles into {target_lang}.\n\
There are {bubble_count} numbered crops in reading order.\n\
Respond ONLY with strict JSON: {{\"1\":{{\"original\":\"...\",\"reading\":\"...\",\"translated\":\"...\"}}}} \
with keys \"1\"..\"{bubble_count}\", no markdown, no extra text.\n\
{style_line}\n\
{sfx_line}",
        style_line = style.line(),
        sfx_line = sfx_rule(skip_sfx),
    )
}

pub fn full_image_prompt(target_lang: &str, style: TranslateStyle, skip_sfx: bool) -> String {
    format!(
        "You are translating all speech bubbles on a full manga page into {target_lang}.\n\
Respond ONLY with a strict JSON array: \
[{{\"x\":0,\"y\":0,\"w\":0,\"h\":0,\"original\":\"...\",\"reading\":\"...\",\"translated\":\"...\"}}] \
with x,y,w,h as percent 0-100 of page size, no markdown, no extra text.\n\
{style_line}\n\
{sfx_line}",
        style_line = style.line(),
        sfx_line = sfx_rule(skip_sfx),
    )
}

pub fn append_glossary(mut prompt: String, glossary: Option<&str>) -> String {
    match glossary {
        Some(g) if !g.trim().is_empty() => {
            prompt.push_str("\n\nGlossary:\n");
            prompt.push_str(g.trim());
            prompt
        }
        _ => prompt,
    }
}

/// Instruksi bebas user (mis. nuansa bahasa output) — ditempel SETELAH
/// glossary supaya paling dekat dengan ekspektasi akhir, tapi larangan
/// format (strict JSON) tetap dipertahankan agar parser tidak jebol.
/// Batas 2000 char: pelindung biaya/token + JSON raksasa sulit diparse.
/// Potong di batas char (bukan byte) supaya tidak membelah UTF-8.
pub const MAX_CUSTOM_PROMPT_CHARS: usize = 2000;

pub fn append_custom(mut prompt: String, custom: Option<&str>) -> String {
    match custom {
        Some(c) if !c.trim().is_empty() => {
            let trimmed = c.trim();
            let cut: String = trimmed.chars().take(MAX_CUSTOM_PROMPT_CHARS).collect();
            prompt.push_str(
                "\n\nAdditional user instructions for wording/tone \
                 (output must still be strict JSON, no extra text):\n",
            );
            prompt.push_str(&cut);
            prompt
        }
        _ => prompt,
    }
}


// ponytail: keyring milik M4; jangan simpan key di sini.
// ponytail: swap expected strings dengan fixture Dart Kuron saat tersedia.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mosaic_snapshot() {
        let p = build_mosaic_prompt("Indonesian", TranslateStyle::Standard, true, 3);
        assert!(p.contains("Indonesian"));
        assert!(p.contains("\"1\"..\"3\""));
        assert!(p.contains("no markdown"));
        assert!(p.contains("Skip sound-effect"));
        assert!(p.contains("natural, faithful"));
    }

    #[test]
    fn full_image_snapshot() {
        let p = full_image_prompt("English", TranslateStyle::Casual, false);
        assert!(p.contains("percent 0-100"));
        assert!(p.contains("JSON array"));
        assert!(p.contains("casual"));
    }

    #[test]
    fn glossary_rules() {
        let base = "P".to_string();
        assert_eq!(append_glossary(base.clone(), None), "P");
        assert_eq!(append_glossary(base.clone(), Some("  ")), "P");
        assert_eq!(
            append_glossary(base.clone(), Some("A=B")),
            "P\n\nGlossary:\nA=B"
        );
    }

    #[test]
    fn style_from_str_fallback() {
        assert_eq!(TranslateStyle::from_str("formal"), TranslateStyle::Formal);
        assert_eq!(TranslateStyle::from_str("bogus"), TranslateStyle::Standard);
    }

    #[test]
    fn custom_rules() {
        let base = "P".to_string();
        assert_eq!(append_custom(base.clone(), None), "P");
        assert_eq!(append_custom(base.clone(), Some("  ")), "P");
        let out = append_custom(base.clone(), Some("pakai bahasa santai"));
        assert!(out.starts_with("P\n\nAdditional user instructions"));
        assert!(out.contains("pakai bahasa santai"));
        assert!(out.contains("strict JSON"));
    }

    #[test]
    fn custom_capped_at_char_boundary() {
        let long = "é".repeat(MAX_CUSTOM_PROMPT_CHARS + 10);
        let out = append_custom(String::new(), Some(&long));
        let tail = out.rsplit('\n').next().unwrap_or("");
        assert_eq!(tail.chars().count(), MAX_CUSTOM_PROMPT_CHARS);
    }
}
