use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::WebhookSummary;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct WebhooksApi;

impl WebhooksApi {
    pub async fn list(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<Vec<WebhookSummary>, GitfleetError> {
        let endpoint = repo_path(repo, &["hooks"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list webhooks: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| WebhookSummary {
                id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                name: raw
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                url: raw
                    .get("config")
                    .and_then(|config| config.get("url"))
                    .or_else(|| raw.get("url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                events: raw
                    .get("events")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                active: raw.get("active").and_then(|v| v.as_bool()).unwrap_or(false),
                created_at: raw
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                updated_at: raw
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }

    pub async fn create(
        client: &ProviderClient,
        repo: &str,
        input: serde_json::Value,
    ) -> Result<WebhookSummary, GitfleetError> {
        let endpoint = repo_path(repo, &["hooks"]);

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(input), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create webhook: {e}")))?;

        Ok(WebhookSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: raw
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            url: raw
                .get("config")
                .and_then(|config| config.get("url"))
                .or_else(|| raw.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            events: raw
                .get("events")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            active: raw.get("active").and_then(|v| v.as_bool()).unwrap_or(false),
            created_at: raw
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            updated_at: raw
                .get("updated_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    pub async fn remove(
        client: &ProviderClient,
        repo: &str,
        hook_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["hooks", &hook_id.to_string()]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn test(
        client: &ProviderClient,
        repo: &str,
        hook_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["hooks", &hook_id.to_string(), "tests"]);

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn deliveries(
        client: &ProviderClient,
        repo: &str,
        hook_id: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["hooks", &hook_id.to_string(), "deliveries"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list deliveries: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::WebhookSummary;

    fn normalize_webhook(raw: &serde_json::Value) -> WebhookSummary {
        WebhookSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: raw
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            url: raw
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            events: raw
                .get("events")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            active: raw.get("active").and_then(|v| v.as_bool()).unwrap_or(false),
            created_at: raw
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            updated_at: raw
                .get("updated_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    #[test]
    fn test_normalize_webhook_full() {
        let json = serde_json::json!({
            "id": 42,
            "name": "web",
            "url": "https://api.github.com/repos/o/r/hooks/42",
            "events": ["push", "pull_request"],
            "active": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        });

        let result = normalize_webhook(&json);

        assert_eq!(result.id, 42);

        assert_eq!(result.name, "web");
        assert_eq!(result.url, "https://api.github.com/repos/o/r/hooks/42");

        assert_eq!(result.events, vec!["push", "pull_request"]);
        assert!(result.active);

        assert_eq!(result.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_normalize_webhook_minimal() {
        let json = serde_json::json!({
            "id": 1
        });

        let result = normalize_webhook(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.name, "");
        assert_eq!(result.url, "");

        assert!(result.events.is_empty());
        assert!(!result.active);
    }
}
