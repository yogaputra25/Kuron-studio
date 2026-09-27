use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiProviderType {
    Zen,
    OpenCodeGo,
    Gemini,
    OpenAi,
    OpenRouter,
    MetaAi,
    ClinePass,
    Cohere,
    Custom,
}

impl AiProviderType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "zen" => Some(Self::Zen),
            "openCodeGo" => Some(Self::OpenCodeGo),
            "gemini" => Some(Self::Gemini),
            "openAi" => Some(Self::OpenAi),
            "openRouter" => Some(Self::OpenRouter),
            "metaAi" => Some(Self::MetaAi),
            "clinePass" => Some(Self::ClinePass),
            "cohere" => Some(Self::Cohere),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::Zen => "https://opencode.ai/zen/v1",
            Self::OpenCodeGo => "https://opencode.ai/zen/go/v1",
            Self::Gemini => "https://generativelanguage.googleapis.com",
            Self::OpenAi => "https://api.openai.com/v1",
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::MetaAi => "https://api.llama.com/v1",
            Self::ClinePass => "https://api.cline.bot/v1",
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
    fn urls() {
        assert_eq!(AiProviderType::Zen.default_base_url(), "https://opencode.ai/zen/v1");
        assert!(AiProviderType::OpenAi.models_url("https://api.openai.com/v1").ends_with("/models"));
        assert!(AiProviderType::Gemini.models_url("https://x").ends_with("/v1beta/models"));
        assert!(!AiProviderType::Custom.needs_key_for_listing());
        assert!(AiProviderType::OpenRouter.needs_key_for_listing());
    }

    #[test]
    fn vision_filter() {
        assert!(!is_vision_capable("text-embedding-3-small"));
        assert!(!is_vision_capable("whisper-1"));
        assert!(is_vision_capable("gpt-4o"));
    }
}
