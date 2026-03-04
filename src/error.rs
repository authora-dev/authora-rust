use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum AuthoraError {
    #[error("API error ({status_code}): {message}")]
    Api {
        status_code: u16,
        message: String,
        code: Option<String>,
    },

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limited")]
    RateLimit,

    #[error("Request timeout")]
    Timeout,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Crypto error: {0}")]
    Crypto(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiErrorBody {
    pub message: Option<String>,
    pub code: Option<String>,
    pub error: Option<String>,
}
