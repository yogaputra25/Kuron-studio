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

pub fn is_vision_capable(model_id: &str) -> bool {
    let l = model_id.to_ascii_lowercase();
    !(l.contains("embedding") || l.contains("whisper") || l.contains("tts") || l.contains("moderation"))
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
            return self.base_url.trim().trim_end_matches('/').to_string();
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
    fn vision_filter() {
        assert!(!is_vision_capable("text-embedding-3-small"));
        assert!(!is_vision_capable("whisper-1"));
        assert!(is_vision_capable("gpt-4o"));
    }
}
