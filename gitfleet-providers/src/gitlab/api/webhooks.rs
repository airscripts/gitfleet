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

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list webhooks: {e}")))?;

        Ok(data.iter().map(normalize_webhook).collect())
    }

    pub async fn create(
        client: &ProviderClient,
        project: &str,
        input: serde_json::Value,
    ) -> Result<WebhookSummary, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/hooks");

        let body = normalize_create_input(&input)?;

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create webhook: {e}")))?;

        Ok(normalize_webhook(&raw))
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

        let endpoint = format!("/projects/{encoded}/hooks/{hook_id}/test/push_events");

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_create_input(input: &serde_json::Value) -> Result<serde_json::Value, GitfleetError> {
    if input.get("active").and_then(serde_json::Value::as_bool) == Some(false) {
        return Err(GitfleetError::new(
            "GitLab does not support creating a disabled project webhook; pass --active.",
        ));
    }

    let config = input
        .get("config")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| GitfleetError::new("Webhook configuration must be a JSON object."))?;
    let url = config
        .get("url")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| GitfleetError::new("Webhook URL is required."))?;
    let mut body = serde_json::json!({ "url": url });

    if let Some(secret) = config.get("secret").and_then(serde_json::Value::as_str) {
        body["token"] = serde_json::Value::String(secret.to_string());
    }

    if let Some(events) = input.get("events").and_then(serde_json::Value::as_array) {
        for event in events.iter().filter_map(serde_json::Value::as_str) {
            let field = match event {
                "push" => Some("push_events"),
                "pull_request" => Some("merge_requests_events"),
                "issues" => Some("issues_events"),
                "issue_comment" => Some("note_events"),
                "release" => Some("releases_events"),
                "deployment" => Some("deployment_events"),
                "workflow_job" => Some("job_events"),
                "workflow_run" => Some("pipeline_events"),
                "wiki" => Some("wiki_page_events"),
                _ => None,
            };

            if let Some(field) = field {
                body[field] = serde_json::Value::Bool(true);
            }
        }
    }

    Ok(body)
}

fn normalize_webhook(raw: &serde_json::Value) -> WebhookSummary {
    const EVENT_FIELDS: [(&str, &str); 10] = [
        ("push_events", "push"),
        ("tag_push_events", "tag_push"),
        ("issues_events", "issues"),
        ("merge_requests_events", "pull_request"),
        ("note_events", "issue_comment"),
        ("releases_events", "release"),
        ("deployment_events", "deployment"),
        ("job_events", "workflow_job"),
        ("pipeline_events", "workflow_run"),
        ("wiki_page_events", "wiki"),
    ];

    let events = EVENT_FIELDS
        .iter()
        .filter(|(field, _)| raw.get(field).and_then(serde_json::Value::as_bool) == Some(true))
        .map(|(_, event)| (*event).to_string())
        .collect::<Vec<_>>();
    let disabled = raw
        .get("disabled_until")
        .is_some_and(|value| !value.is_null());

    WebhookSummary {
        id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        name: raw
            .get("name")
            .or_else(|| raw.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        url: raw
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        active: !events.is_empty() && !disabled,
        events,
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

#[cfg(test)]
mod tests {
    use super::normalize_webhook;

    #[test]
    fn test_normalize_gitlab_webhook_full() {
        let json = serde_json::json!({
            "id": 42,
            "url": "https://example.com/hook",
            "push_events": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        });

        let result = normalize_webhook(&json);

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
            "push_events": true,
            "disabled_until": "2026-08-01T00:00:00Z"
        });

        let result = normalize_webhook(&json);

        assert!(!result.active);

        assert_eq!(result.events, vec!["push"]);
    }

    #[test]
    fn test_normalize_gitlab_webhook_minimal() {
        let json = serde_json::json!({});

        let result = normalize_webhook(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.name, "");
        assert!(!result.active);
    }
}
