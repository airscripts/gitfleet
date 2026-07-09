use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::WebhookSummary;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct WebhooksApi;

impl WebhooksApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
    ) -> Result<Vec<WebhookSummary>, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/hooks");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list webhooks: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| WebhookSummary {
                id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                name: raw
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                url: raw
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                events: raw
                    .get("push_events")
                    .and_then(|v| v.as_bool())
                    .map(|b| if b { vec!["push".to_string()] } else { vec![] })
                    .unwrap_or_default(),
                active: raw
                    .get("push_events")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
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
        project: &str,
        input: serde_json::Value,
    ) -> Result<WebhookSummary, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/hooks");

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(input), None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create webhook: {e}")))?;

        Ok(WebhookSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: raw
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            url: raw
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            events: vec![],
            active: raw
                .get("push_events")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
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
        project: &str,
        hook_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/hooks/{hook_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn test(
        client: &ProviderClient,
        project: &str,
        hook_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/hooks/{hook_id}/test");

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::WebhookSummary;

    fn normalize_gitlab_webhook(raw: &serde_json::Value) -> WebhookSummary {
        WebhookSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: raw
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            url: raw
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            events: raw
                .get("push_events")
                .and_then(|v| v.as_bool())
                .map(|b| if b { vec!["push".to_string()] } else { vec![] })
                .unwrap_or_default(),
            active: raw
                .get("push_events")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
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
    fn test_normalize_gitlab_webhook_full() {
        let json = serde_json::json!({
            "id": 42,
            "url": "https://example.com/hook",
            "push_events": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        });

        let result = normalize_gitlab_webhook(&json);

        assert_eq!(result.id, 42);

        assert_eq!(result.name, "https://example.com/hook");
        assert_eq!(result.url, "https://example.com/hook");

        assert_eq!(result.events, vec!["push"]);
        assert!(result.active);
    }

    #[test]
    fn test_normalize_gitlab_webhook_inactive() {
        let json = serde_json::json!({
            "id": 1,
            "url": "https://example.com/hook2",
            "push_events": false
        });

        let result = normalize_gitlab_webhook(&json);

        assert!(!result.active);

        assert!(result.events.is_empty());
    }

    #[test]
    fn test_normalize_gitlab_webhook_minimal() {
        let json = serde_json::json!({});

        let result = normalize_gitlab_webhook(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.name, "");
        assert!(!result.active);
    }
}
