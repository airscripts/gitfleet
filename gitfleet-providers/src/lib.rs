use base64::Engine;

use gitfleet_core::constants::MAX_HTTP_RESPONSE_BYTES;
use gitfleet_core::errors::GitfleetError;

pub mod github;
pub mod gitlab;
pub mod registry;

mod retry;

pub use github::GitHubProvider;
pub use gitlab::GitLabProvider;
pub use registry::ProviderRegistry;

pub(crate) fn decode_file_content(
    mut data: serde_json::Value,
) -> Result<serde_json::Value, GitfleetError> {
    let is_base64 = data
        .get("encoding")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|encoding| encoding.eq_ignore_ascii_case("base64"));

    if !is_base64 {
        return Ok(data);
    }

    let Some(content) = data.get("content").and_then(serde_json::Value::as_str) else {
        return Ok(data);
    };

    let compact = content
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(compact)
        .map_err(|error| GitfleetError::new(format!("Failed to decode file content: {error}")))?;
    let decoded = String::from_utf8(decoded)
        .map_err(|error| GitfleetError::new(format!("File content is not valid UTF-8: {error}")))?;

    if let Some(object) = data.as_object_mut() {
        object.insert("content".to_string(), serde_json::Value::String(decoded));
        object.insert(
            "encoding".to_string(),
            serde_json::Value::String("utf-8".to_string()),
        );
    }

    Ok(data)
}

#[cfg(test)]
mod content_tests {
    use super::*;

    #[test]
    fn decodes_base64_file_content() {
        let data = serde_json::json!({"content": "aGVsbG8=", "encoding": "base64"});
        let decoded = decode_file_content(data).unwrap();

        assert_eq!(decoded["content"], "hello");
        assert_eq!(decoded["encoding"], "utf-8");
    }

    #[test]
    fn preserves_plain_file_content() {
        let data = serde_json::json!({"content": "hello", "encoding": "text"});
        let decoded = decode_file_content(data.clone()).unwrap();

        assert_eq!(decoded, data);
    }
}

pub(crate) fn validate_relative_endpoint(endpoint: &str) -> Result<(), GitfleetError> {
    if !endpoint.starts_with('/')
        || endpoint.contains("://")
        || endpoint.contains('#')
        || endpoint.chars().any(char::is_control)
    {
        return Err(GitfleetError::new(
            "Provider endpoint must be a relative path beginning with '/'.",
        ));
    }

    let path = endpoint.split('?').next().unwrap_or(endpoint);

    if path.split('/').any(|segment| {
        segment == "." || segment == ".." || segment.to_ascii_lowercase().contains("%2e")
    }) {
        return Err(GitfleetError::new(
            "Provider endpoint must not contain path traversal segments.",
        ));
    }

    Ok(())
}

pub(crate) async fn parse_json<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, GitfleetError> {
    let body = read_response_bytes(response).await?;

    serde_json::from_slice(&body)
        .map_err(|error| GitfleetError::new(format!("Failed to parse provider response: {error}")))
}

pub(crate) async fn parse_graphql(
    response: reqwest::Response,
    operation: &str,
) -> Result<serde_json::Value, GitfleetError> {
    let value: serde_json::Value = parse_json(response).await?;

    validate_graphql_response(value, operation)
}

fn validate_graphql_response(
    value: serde_json::Value,
    operation: &str,
) -> Result<serde_json::Value, GitfleetError> {
    let Some(errors) = value.get("errors") else {
        return Ok(value);
    };

    let messages = errors.as_array().ok_or_else(|| {
        GitfleetError::new(format!(
            "GitHub GraphQL {operation} returned an invalid errors payload."
        ))
    })?;

    if messages.is_empty() {
        return Ok(value);
    }

    let details = messages
        .iter()
        .take(5)
        .filter_map(|error| error.get("message").and_then(serde_json::Value::as_str))
        .map(|message| message.chars().take(500).collect::<String>())
        .collect::<Vec<_>>()
        .join("; ");

    let details = if details.is_empty() {
        "The provider did not include an error message.".to_string()
    } else {
        details
    };

    Err(GitfleetError::new(format!(
        "GitHub GraphQL {operation} failed: {details}"
    )))
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

#[cfg(test)]
mod tests {
    use super::{validate_graphql_response, validate_relative_endpoint};

    #[test]
    fn test_validate_relative_endpoint_accepts_provider_paths() {
        assert!(validate_relative_endpoint("/repos/org/repo?per_page=100").is_ok());
        assert!(validate_relative_endpoint("//mark_todos_as_done").is_ok());
    }

    #[test]
    fn test_validate_relative_endpoint_rejects_path_traversal() {
        assert!(validate_relative_endpoint("/../admin").is_err());
        assert!(validate_relative_endpoint("/api/%2e%2e/admin").is_err());
    }

    #[test]
    fn test_validate_graphql_response_accepts_data() {
        let response = serde_json::json!({"data": {"viewer": {"login": "octocat"}}});

        assert!(validate_graphql_response(response, "viewer query").is_ok());
    }

    #[test]
    fn test_validate_graphql_response_rejects_errors() {
        let response = serde_json::json!({
            "data": null,
            "errors": [{"message": "Resource not accessible"}]
        });

        let error = validate_graphql_response(response, "project deletion").unwrap_err();

        assert!(error.to_string().contains("project deletion failed"));
        assert!(error.to_string().contains("Resource not accessible"));
    }

    #[test]
    fn test_validate_graphql_response_rejects_malformed_errors() {
        let response = serde_json::json!({"errors": "invalid"});

        let error = validate_graphql_response(response, "query").unwrap_err();

        assert!(error.to_string().contains("invalid errors payload"));
    }
}
