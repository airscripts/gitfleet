use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct ForksApi;

impl ForksApi {
    pub async fn list(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["forks"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list forks: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forks_list_endpoint() {
        let repo = "owner/repo";
        let endpoint = repo_path(repo, &["forks"]);

        assert_eq!(endpoint, "/repos/owner/repo/forks");
    }
}
