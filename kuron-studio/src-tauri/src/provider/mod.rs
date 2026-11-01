pub mod cohere;
pub mod config;
pub mod gemini;
pub mod http;
pub mod openai;
pub mod repo;

use crate::error::AppError;

pub use config::{effective_model, is_vision_capable, AiModelOption, AiProviderType, ProviderRecord};
// ponytail: secret milik `secrets` (tanpa dep kripto di sini).
// ponytail: factory = match satu produk; trait object bila provider ke-4 butuh dispatch dinamis.

// OpenAI-compatible dispatch: OpenAi/MetaAi/Custom (Gemini + Cohere punya modul sendiri).
pub async fn translate_image(
    client: &reqwest::Client,
    rec: &ProviderRecord,
    prompt: &str,
    jpeg: &[u8],
) -> Result<String, AppError> {
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
) -> Result<Vec<AiModelOption>, AppError> {
    let base = rec.effective_base_url();
    match rec.provider_type {
        AiProviderType::Gemini => gemini::list_models(client, &base, &rec.api_key).await,
        AiProviderType::Cohere => cohere::list_models(client, &base, &rec.api_key).await,
        _ => openai::list_models(client, &base, &rec.api_key, rec.provider_type).await,
    }
}

pub async fn validate(
    client: &reqwest::Client,
    rec: &ProviderRecord,
) -> Result<(), AppError> {
    let base = rec.effective_base_url();
    match rec.provider_type {
        AiProviderType::Gemini => gemini::validate_minimal(client, &base, &rec.api_key, &rec.model).await,
        AiProviderType::Cohere => cohere::validate_minimal(client, &base, &rec.api_key, &rec.model).await,
        _ => openai::validate_minimal(client, &base, &rec.api_key, &rec.model).await,
    }
}
