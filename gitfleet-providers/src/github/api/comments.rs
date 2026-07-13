use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::CommentSummary;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct CommentsApi;

impl CommentsApi {
    pub async fn list_issue_comments(
        client: &ProviderClient,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<CommentSummary>, GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &issue_number.to_string(), "comments"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse issue comments: {e}")))?;

        Ok(raw
            .iter()
            .map(|c| CommentSummary {
                id: c.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                body: c
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                author: c
                    .get("user")
                    .and_then(|u| u.get("login"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                created_at: c
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                updated_at: c
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }

    pub async fn create_issue_comment(
        client: &ProviderClient,
        repo: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<CommentSummary, GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &issue_number.to_string(), "comments"]);

        let payload = serde_json::json!({ "body": body });
        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(payload), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse created comment: {e}")))?;

        Ok(CommentSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            body: raw
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            author: raw
                .get("user")
                .and_then(|u| u.get("login"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
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

    pub async fn list_pr_comments(
        client: &ProviderClient,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<CommentSummary>, GitfleetError> {
        let endpoint = repo_path(repo, &["pulls", &pr_number.to_string(), "comments"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse PR comments: {e}")))?;

        Ok(raw
            .iter()
            .map(|c| CommentSummary {
                id: c.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                body: c
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                author: c
                    .get("user")
                    .and_then(|u| u.get("login"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                created_at: c
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                updated_at: c
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::CommentSummary;

    fn normalize_comment(raw: &serde_json::Value) -> CommentSummary {
        CommentSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            body: raw
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            author: raw
                .get("user")
                .and_then(|u| u.get("login"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
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
    fn test_normalize_comment_full() {
        let json = serde_json::json!({
            "id": 42,
            "body": "Great PR!",
            "user": { "login": "reviewer" },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        });

        let result = normalize_comment(&json);

        assert_eq!(result.id, 42);

        assert_eq!(result.body, "Great PR!");
        assert_eq!(result.author, Some("reviewer".to_string()));

        assert_eq!(result.created_at, "2024-01-01T00:00:00Z");
        assert_eq!(result.updated_at, "2024-01-02T00:00:00Z");
    }

    #[test]
    fn test_normalize_comment_minimal() {
        let json = serde_json::json!({ "id": 1 });

        let result = normalize_comment(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.body, "");
        assert!(result.author.is_none());

        assert_eq!(result.created_at, "");
    }
}
