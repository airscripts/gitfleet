use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::Notification;

use crate::gitlab::client::ProviderClient;

pub struct NotificationsApi;

impl NotificationsApi {
    pub async fn list(
        client: &ProviderClient,
        all: bool,
        participating: bool,
        repo: Option<&str>,
    ) -> Result<Vec<Notification>, GitfleetError> {
        let mut params = String::new();

        if all {
            params.push_str("state=all&");
        }

        if participating {
            params.push_str("action=mentioned&");
        }

        params.push_str("per_page=100");

        let endpoint = match repo {
            Some(r) => {
                let encoded = urlencoding::encode(r).to_string();
                format!("/projects/{encoded}/todos?{params}")
            }

            None => format!("/todos?{params}"),
        };

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list todos: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| {
                let project = raw.get("project").and_then(|v| v.as_object());
                Notification {
                    id: raw
                        .get("id")
                        .and_then(|v| v.as_u64())
                        .map(|i| i.to_string())
                        .unwrap_or_default(),
                    repository: project
                        .and_then(|o| o.get("path_with_namespace"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    subject_title: raw
                        .get("body")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    subject_type: raw
                        .get("target_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    reason: raw
                        .get("action_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    unread: !raw
                        .get("state")
                        .and_then(|v| v.as_str())
                        .map(|s| s == "done")
                        .unwrap_or(false),
                    updated_at: raw
                        .get("updated_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                }
            })
            .collect())
    }

    pub async fn mark_read(client: &ProviderClient) -> Result<(), GitfleetError> {
        client
            .request_token_required(
                reqwest::Method::POST,
                "//mark_todos_as_done",
                Some(serde_json::json!({})),
                None,
                None,
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::Notification;

    fn normalize_notification(raw: &serde_json::Value) -> Notification {
        let project = raw.get("project").and_then(|v| v.as_object());
        Notification {
            id: raw
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|i| i.to_string())
                .unwrap_or_default(),
            repository: project
                .and_then(|o| o.get("path_with_namespace"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            subject_title: raw
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            subject_type: raw
                .get("target_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            reason: raw
                .get("action_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            unread: !raw
                .get("state")
                .and_then(|v| v.as_str())
                .map(|s| s == "done")
                .unwrap_or(false),
            updated_at: raw
                .get("updated_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    #[test]
    fn test_normalize_gitlab_notification_full() {
        let json = serde_json::json!({
            "id": 99,
            "project": { "path_with_namespace": "org/repo" },
            "body": "Merge request approved",
            "target_type": "MergeRequest",
            "action_name": "approved",
            "state": "pending",
            "updated_at": "2024-06-01T00:00:00Z"
        });

        let result = normalize_notification(&json);

        assert_eq!(result.id, "99");

        assert_eq!(result.repository, "org/repo");
        assert_eq!(result.subject_title, "Merge request approved");

        assert_eq!(result.subject_type, "MergeRequest");
        assert_eq!(result.reason, "approved");

        assert!(result.unread);
        assert_eq!(result.updated_at, "2024-06-01T00:00:00Z");
    }

    #[test]
    fn test_normalize_gitlab_notification_done() {
        let json = serde_json::json!({
            "id": 50,
            "state": "done"
        });

        let result = normalize_notification(&json);

        assert!(!result.unread);
    }

    #[test]
    fn test_normalize_gitlab_notification_minimal() {
        let json = serde_json::json!({});

        let result = normalize_notification(&json);

        assert_eq!(result.id, "");

        assert_eq!(result.repository, "");
        assert!(result.unread);
    }
}
