use gitfleet_core::constants::MAX_HTTP_RESPONSE_BYTES;
use gitfleet_core::errors::GitfleetError;

pub mod github;
pub mod gitlab;
pub mod registry;

mod retry;

pub use github::GitHubProvider;
pub use gitlab::GitLabProvider;
pub use registry::ProviderRegistry;

pub(crate) async fn parse_json<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, GitfleetError> {
    let body = read_response_bytes(response).await?;

    serde_json::from_slice(&body)
        .map_err(|error| GitfleetError::new(format!("Failed to parse provider response: {error}")))
}

pub(crate) async fn read_response_text(
    response: reqwest::Response,
) -> Result<String, GitfleetError> {
    let body = read_response_bytes(response).await?;

    String::from_utf8(body).map_err(|error| {
        GitfleetError::new(format!("Provider response was not valid UTF-8: {error}"))
    })
}

async fn read_response_bytes(mut response: reqwest::Response) -> Result<Vec<u8>, GitfleetError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_HTTP_RESPONSE_BYTES as u64)
    {
        return Err(GitfleetError::new(format!(
            "Provider response exceeds the {} MiB limit.",
            MAX_HTTP_RESPONSE_BYTES / (1024 * 1024)
        )));
    }

    let mut body = Vec::with_capacity(
        response
            .content_length()
            .map(|length| length as usize)
            .unwrap_or_default()
            .min(MAX_HTTP_RESPONSE_BYTES),
    );

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| GitfleetError::new(format!("Failed to read provider response: {error}")))?
    {
        if chunk.len() > MAX_HTTP_RESPONSE_BYTES.saturating_sub(body.len()) {
            return Err(GitfleetError::new(format!(
                "Provider response exceeds the {} MiB limit.",
                MAX_HTTP_RESPONSE_BYTES / (1024 * 1024)
            )));
        }

        body.extend_from_slice(&chunk);
    }

    Ok(body)
}
