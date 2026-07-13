use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct StatusApi;

impl StatusApi {
    pub async fn get_combined_status(
        client: &ProviderClient,
        repo: &str,
        r#ref: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["commits", r#ref, "status"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get combined status: {e}")))?;

        Ok(data)
    }

    pub async fn list_statuses(
        client: &ProviderClient,
        repo: &str,
        r#ref: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["statuses", r#ref]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list statuses: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_combined_endpoint() {
        let repo = "owner/repo";
        let r#ref = "abc123";
        let endpoint = repo_path(repo, &["commits", r#ref, "status"]);

        assert_eq!(endpoint, "/repos/owner/repo/commits/abc123/status");
    }

    #[test]
    fn test_status_list_endpoint() {
        let repo = "owner/repo";
        let r#ref = "main";
        let endpoint = repo_path(repo, &["statuses", r#ref]);

        assert_eq!(endpoint, "/repos/owner/repo/statuses/main");
    }
}
