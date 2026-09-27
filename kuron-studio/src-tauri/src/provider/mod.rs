pub mod cohere;
pub mod config;
pub mod gemini;
pub mod openai;

pub use config::{is_vision_capable, AiModelOption, AiProviderType, ProviderRecord};

// ponytail: keyring milik M4 (tanpa keyring dep di sini).
// ponytail: factory = match satu produk; trait object bila provider ke-4 butuh dispatch dinamis.

// OpenAI-compatible dispatch: Zen/OpenCodeGo/OpenAi/OpenRouter/MetaAi/ClinePass/Custom.
pub async fn translate_image(
    client: &reqwest::Client,
    rec: &ProviderRecord,
    prompt: &str,
    jpeg: &[u8],
) -> Result<String, String> {
    let base = rec.effective_base_url();
    match rec.provider_type {
        AiProviderType::Gemini => gemini::chat_completions_image(client, &base, &rec.api_key, &rec.model, prompt, jpeg).await,
        AiProviderType::Cohere => cohere::chat_completions_image(client, &base, &rec.api_key, &rec.model, prompt, jpeg).await,
        _ => openai::chat_completions_image(client, &base, &rec.api_key, &rec.model, prompt, jpeg).await,
    }
}

pub async fn list_models(
    client: &reqwest::Client,
    rec: &ProviderRecord,
) -> Result<Vec<AiModelOption>, String> {
    let base = rec.effective_base_url();
    match rec.provider_type {
        AiProviderType::Gemini => gemini::list_models(client, &base, &rec.api_key).await,
        AiProviderType::Cohere => cohere::list_models(client, &base, &rec.api_key).await,
        _ => openai::list_models(client, &base, &rec.api_key).await,
    }
}

pub async fn validate(
    client: &reqwest::Client,
    rec: &ProviderRecord,
) -> Result<(), String> {
    let base = rec.effective_base_url();
    match rec.provider_type {
        AiProviderType::Gemini => gemini::validate_minimal(client, &base, &rec.api_key, &rec.model).await,
        AiProviderType::Cohere => cohere::validate_minimal(client, &base, &rec.api_key, &rec.model).await,
        _ => openai::validate_minimal(client, &base, &rec.api_key, &rec.model).await,
    }
}
