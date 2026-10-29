use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AiProviderType {
    Gemini,
    OpenAi,
    MetaAi,
    Cohere,
    Custom,
}

impl<'de> Deserialize<'de> for AiProviderType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "gemini" => Ok(Self::Gemini),
            "openAi" => Ok(Self::OpenAi),
            "metaAi" => Ok(Self::MetaAi),
            "cohere" => Ok(Self::Cohere),
            "custom" => Ok(Self::Custom),
            // Preset hapus (#26/#27): simpanan lama tetap terbaca, jadi Custom.
            "zen" | "openCodeGo" | "openRouter" | "clinePass" => Ok(Self::Custom),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["gemini", "openAi", "metaAi", "cohere", "custom"],
            )),
        }
    }
}

impl AiProviderType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "gemini" => Some(Self::Gemini),
            "openAi" => Some(Self::OpenAi),
            "metaAi" => Some(Self::MetaAi),
            "cohere" => Some(Self::Cohere),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::Gemini => "https://generativelanguage.googleapis.com",
            Self::OpenAi => "https://api.openai.com/v1",
            Self::MetaAi => "https://api.meta.ai/v1",
            Self::Cohere => "https://api.cohere.ai",
            Self::Custom => "",
        }
    }

    pub fn models_url(&self, base: &str) -> String {
        match self {
            Self::Gemini => format!("{base}/v1beta/models"),
            Self::Cohere => format!("{base}/v1/models"),
            _ => format!("{base}/models"),
        }
    }

    pub fn needs_key_for_listing(&self) -> bool {
        !matches!(self, Self::Custom)
    }

    pub fn is_openai_compatible(&self) -> bool {
        !matches!(self, Self::Gemini | Self::Cohere)
    }
}

pub fn vision_capable(provider: AiProviderType, model_id: &str) -> bool {
    let l = model_id.to_ascii_lowercase();
    // 1. Keluarga non-vision eksplisit — berlaku semua provider.
    for pat in [
        "embedding",
        "whisper",
        "tts",
        "transcribe",
        "moderation",
        "muse-image",
        "muse-voice",
        "sam-",
        "dall-e",
    ] {
        if l.contains(pat) {
            return false;
        }
    }
    // 2. Keluarga vision eksplisit (terverifikasi dev.meta.ai / docs.cohere.com Okt 2026).
    for pat in [
        "muse-spark",
        "muse-wonder",
        "command-a-vision",
        "aya-vision",
        "vision",
        "gpt-4o",
        "gpt-4.1",
        "gpt-4-turbo",
        "gpt-5",
        "gemini",
        "claude-3",
        "claude-4",
    ] {
        if l.contains(pat) {
            return true;
        }
    }
    // 3. Keluarga teks saja per provider (lolos dari dua saringan di atas).
    let text_only: &[&str] = match provider {
        AiProviderType::Cohere => &["command-r", "command-a", "command-light", "embed", "rerank"],
        AiProviderType::OpenAi => &["gpt-3.5", "gpt-4"],
        AiProviderType::Gemini | AiProviderType::MetaAi | AiProviderType::Custom => &[],
    };
    if text_only.iter().any(|pat| l.contains(pat)) {
        return false;
    }
    // 4. Fallback = perilaku lama: anggap vision (jangan takuti user model baru).
    true
}

pub fn is_vision_capable(model_id: &str) -> bool {
    vision_capable(AiProviderType::Custom, model_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRecord {
    pub id: String,
    pub provider_type: AiProviderType,
    pub name: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model: String,
}

impl ProviderRecord {
    pub fn effective_base_url(&self) -> String {
        if !self.base_url.trim().is_empty() {
            let b = self.base_url.trim().trim_end_matches('/').to_string();
            // Migrasi diam (#26): preset lama api.llama.com -> api.meta.ai.
            return b.replace("api.llama.com", "api.meta.ai");
        }
        self.provider_type.default_base_url().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiModelOption {
    pub id: String,
    pub label: String,
    pub vision: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removed_preset_names_still_deserialize_to_custom() {
        // Preset hapus (#26/#27): simpanan lama tetap terbaca sebagai Custom.
        for legacy in ["zen", "openCodeGo", "openRouter", "clinePass"] {
            let t: AiProviderType = serde_json::from_str(&format!("\"{legacy}\"")).unwrap();
            assert_eq!(t, AiProviderType::Custom, "legacy {legacy}");
            assert_eq!(AiProviderType::from_str(legacy), None, "from_str {legacy}");
        }
        // Penyintas tidak berubah.
        let t: AiProviderType = serde_json::from_str("\"metaAi\"").unwrap();
        assert_eq!(t, AiProviderType::MetaAi);
    }

    #[test]
    fn urls() {
        assert_eq!(AiProviderType::MetaAi.default_base_url(), "https://api.meta.ai/v1");
        assert!(AiProviderType::OpenAi.models_url("https://api.openai.com/v1").ends_with("/models"));
        assert!(AiProviderType::Gemini.models_url("https://x").ends_with("/v1beta/models"));
        assert!(!AiProviderType::Custom.needs_key_for_listing());
        assert!(AiProviderType::MetaAi.needs_key_for_listing());
    }
    #[test]
    fn legacy_meta_base_normalizes() {
        let rec = ProviderRecord {
            id: "x".into(),
            provider_type: AiProviderType::MetaAi,
            name: "m".into(),
            base_url: "https://api.llama.com/v1".into(),
            api_key: String::new(),
            model: String::new(),
        };
        assert_eq!(rec.effective_base_url(), "https://api.meta.ai/v1");
    }

    #[test]
    fn vision_filter() {
        assert!(!is_vision_capable("text-embedding-3-small"));
        assert!(!is_vision_capable("whisper-1"));
        assert!(is_vision_capable("gpt-4o"));
    }
    #[test]
    fn vision_allowlist_per_provider() {
        use AiProviderType::*;
        assert!(vision_capable(MetaAi, "muse-spark-1.3"));
        assert!(vision_capable(MetaAi, "muse-wonder-1.0"));
        assert!(!vision_capable(MetaAi, "muse-image-1.0"));
        assert!(!vision_capable(MetaAi, "muse-voice-transcribe-1.0"));
        assert!(!vision_capable(MetaAi, "sam-3.1"));
        assert!(vision_capable(Cohere, "command-a-vision-07-2025"));
        assert!(!vision_capable(Cohere, "command-r-plus-08-2024"));
        assert!(vision_capable(Gemini, "gemini-2.0-flash"));
        assert!(!vision_capable(Gemini, "text-embedding-004"));
        assert!(vision_capable(OpenAi, "gpt-4o"));
        assert!(!vision_capable(OpenAi, "gpt-3.5-turbo"));
        assert!(!vision_capable(OpenAi, "dall-e-3"));
        assert!(vision_capable(Custom, "model-tak-dikenal"));
        assert!(!vision_capable(Custom, "whisper-1"));
    }
}
