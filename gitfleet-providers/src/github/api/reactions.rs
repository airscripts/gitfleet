use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::ReactionSummary;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct ReactionsApi;

impl ReactionsApi {
    pub async fn list_for_issue(
        client: &ProviderClient,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<ReactionSummary>, GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &issue_number.to_string(), "reactions"]);

        let response = client
            .request_token_required(
                reqwest::Method::GET,
                &endpoint,
                None,
                None,
                Some("application/vnd.github.squirrel-girl-preview+json"),
            )
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list reactions: {e}")))?;

        Ok(raw.iter().map(normalize_reaction).collect())
    }

    pub async fn create_for_issue(
        client: &ProviderClient,
        repo: &str,
        issue_number: u64,
        content: &str,
    ) -> Result<ReactionSummary, GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &issue_number.to_string(), "reactions"]);

        let payload = serde_json::json!({ "content": content });
        let response = client
            .request_token_required(
                reqwest::Method::POST,
                &endpoint,
                Some(payload),
                None,
                Some("application/vnd.github.squirrel-girl-preview+json"),
            )
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create reaction: {e}")))?;

        Ok(normalize_reaction(&raw))
    }

    pub async fn delete_for_issue(
        client: &ProviderClient,
        repo: &str,
        issue_number: u64,
        reaction_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(
            repo,
            &[
                "issues",
                &issue_number.to_string(),
                "reactions",
                &reaction_id.to_string(),
            ],
        );

        client
            .request_token_required(
                reqwest::Method::DELETE,
                &endpoint,
                None,
                None,
                Some("application/vnd.github.squirrel-girl-preview+json"),
            )
            .await?;

        Ok(())
    }

    pub async fn list_for_comment(
        client: &ProviderClient,
        repo: &str,
        comment_id: u64,
    ) -> Result<Vec<ReactionSummary>, GitfleetError> {
        let endpoint = repo_path(
            repo,
            &["issues", "comments", &comment_id.to_string(), "reactions"],
        );

        let response = client
            .request_token_required(
                reqwest::Method::GET,
                &endpoint,
                None,
                None,
                Some("application/vnd.github.squirrel-girl-preview+json"),
            )
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list comment reactions: {e}")))?;

        Ok(raw.iter().map(normalize_reaction).collect())
    }

    pub async fn create_for_comment(
        client: &ProviderClient,
        repo: &str,
        comment_id: u64,
        content: &str,
    ) -> Result<ReactionSummary, GitfleetError> {
        let endpoint = repo_path(
            repo,
            &["issues", "comments", &comment_id.to_string(), "reactions"],
        );

        let payload = serde_json::json!({ "content": content });

        let response = client
            .request_token_required(
                reqwest::Method::POST,
                &endpoint,
                Some(payload),
                None,
                Some("application/vnd.github.squirrel-girl-preview+json"),
            )
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create reaction: {e}")))?;

        Ok(normalize_reaction(&raw))
    }

    pub async fn delete_for_comment(
        client: &ProviderClient,
        repo: &str,
        comment_id: u64,
        reaction_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(
            repo,
            &[
                "issues",
                "comments",
                &comment_id.to_string(),
                "reactions",
                &reaction_id.to_string(),
            ],
        );

        client
            .request_token_required(
                reqwest::Method::DELETE,
                &endpoint,
                None,
                None,
                Some("application/vnd.github.squirrel-girl-preview+json"),
            )
            .await?;

        Ok(())
    }
}

fn normalize_reaction(raw: &serde_json::Value) -> ReactionSummary {
    ReactionSummary {
        id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        content: raw
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        user: raw
            .get("user")
            .and_then(|u| u.get("login"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        created_at: raw
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_reaction_full() {
        let json = serde_json::json!({
            "id": 42,
            "content": "+1",
            "user": { "login": "octocat" },
            "created_at": "2024-01-01T00:00:00Z"
        });

        let result = normalize_reaction(&json);

        assert_eq!(result.id, 42);

        assert_eq!(result.content, "+1");
        assert_eq!(result.user, Some("octocat".to_string()));

        assert_eq!(result.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_normalize_reaction_minimal() {
        let json = serde_json::json!({ "id": 1 });

        let result = normalize_reaction(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.content, "");
        assert!(result.user.is_none());

        assert_eq!(result.created_at, "");
    }
}
