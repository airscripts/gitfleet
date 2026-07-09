use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::Notification;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct NotificationsApi;

impl NotificationsApi {
    pub async fn fetch(
        client: &ProviderClient,
        all: bool,
        participating: bool,
        repo: Option<&str>,
    ) -> Result<Vec<Notification>, GitfleetError> {
        let mut params = String::new();

        if all {
            params.push_str("all=true&");
        }

        if participating {
            params.push_str("participating=true&");
        }

        params.push_str("per_page=100");

        let endpoint = match repo {
            Some(r) => format!("{}?{params}", repo_path(r, &["notifications"])),
            None => format!("/notifications?{params}"),
        };

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to fetch notifications: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| {
                let repo_obj = raw.get("repository").and_then(|v| v.as_object());

                let subject = raw.get("subject").and_then(|v| v.as_object());
                Notification {
                    id: raw
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    repository: repo_obj
                        .and_then(|o| o.get("full_name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    subject_title: subject
                        .and_then(|o| o.get("title"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    subject_type: subject
                        .and_then(|o| o.get("type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    reason: raw
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    unread: raw.get("unread").and_then(|v| v.as_bool()).unwrap_or(false),
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
                reqwest::Method::PUT,
                "/notifications",
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
        let repo_obj = raw.get("repository").and_then(|v| v.as_object());

        let subject = raw.get("subject").and_then(|v| v.as_object());
        Notification {
            id: raw
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            repository: repo_obj
                .and_then(|o| o.get("full_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            subject_title: subject
                .and_then(|o| o.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            subject_type: subject
                .and_then(|o| o.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            reason: raw
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            unread: raw.get("unread").and_then(|v| v.as_bool()).unwrap_or(false),
            updated_at: raw
                .get("updated_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    #[test]
    fn test_normalize_notification_full() {
        let json = serde_json::json!({
            "id": "1234567",
            "repository": {
                "id": 12345,
                "full_name": "testorg/my-repo",
                "html_url": "https://github.com/testorg/my-repo"
            },
            "subject": {
                "title": "Fix bug in auth",
                "type": "PullRequest",
                "url": "https://api.github.com/repos/testorg/my-repo/pulls/42"
            },
            "reason": "subscribed",
            "unread": true,
            "updated_at": "2024-06-15T12:00:00Z"
        });

        let result = normalize_notification(&json);

        assert_eq!(result.id, "1234567");

        assert_eq!(result.repository, "testorg/my-repo");
        assert_eq!(result.subject_title, "Fix bug in auth");

        assert_eq!(result.subject_type, "PullRequest");
        assert_eq!(result.reason, "subscribed");

        assert!(result.unread);
        assert_eq!(result.updated_at, "2024-06-15T12:00:00Z");
    }

    #[test]
    fn test_normalize_notification_minimal() {
        let json = serde_json::json!({
            "id": "99"
        });

        let result = normalize_notification(&json);

        assert_eq!(result.id, "99");

        assert_eq!(result.repository, "");
        assert_eq!(result.subject_title, "");

        assert_eq!(result.subject_type, "");
        assert_eq!(result.reason, "");

        assert!(!result.unread);
        assert_eq!(result.updated_at, "");
    }
}
