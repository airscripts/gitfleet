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
        let project_id = match repo {
            Some(project) => Some(resolve_project_id(client, project).await?),
            None => None,
        };
        let mut states = vec![None];

        if all {
            states = vec![Some("pending"), Some("done")];
        }

        let mut data = Vec::new();

        for state in states {
            data.extend(fetch_todos(client, state, participating, project_id).await?);
        }

        Ok(data.iter().map(normalize_notification).collect())
    }

    pub async fn mark_read(client: &ProviderClient) -> Result<(), GitfleetError> {
        client
            .request_token_required(
                reqwest::Method::POST,
                "/todos/mark_as_done",
                None,
                None,
                None,
            )
            .await?;

        Ok(())
    }
}

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
            .get("target")
            .and_then(|target| target.get("title"))
            .or_else(|| raw.get("body"))
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

async fn resolve_project_id(client: &ProviderClient, project: &str) -> Result<u64, GitfleetError> {
    let endpoint = format!("/projects/{}", urlencoding::encode(project));
    let response = client
        .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
        .await?;
    let project: serde_json::Value = crate::parse_json(response)
        .await
        .map_err(|e| GitfleetError::new(format!("Failed to resolve project: {e}")))?;

    project
        .get("id")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| GitfleetError::new("GitLab project response did not include an ID."))
}

async fn fetch_todos(
    client: &ProviderClient,
    state: Option<&str>,
    participating: bool,
    project_id: Option<u64>,
) -> Result<Vec<serde_json::Value>, GitfleetError> {
    let mut params = vec!["per_page=100".to_string()];

    if let Some(state) = state {
        params.push(format!("state={state}"));
    }

    if participating {
        params.push("action=mentioned".to_string());
    }

    if let Some(project_id) = project_id {
        params.push(format!("project_id={project_id}"));
    }

    let endpoint = format!("/todos?{}", params.join("&"));
    let response = client
        .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
        .await?;

    crate::parse_json(response)
        .await
        .map_err(|e| GitfleetError::new(format!("Failed to list todos: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_gitlab_notification_full() {
        let json = serde_json::json!({
            "id": 99,
            "project": { "path_with_namespace": "org/repo" },
            "body": "fallback body",
            "target": {"title": "Merge request approved"},
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
