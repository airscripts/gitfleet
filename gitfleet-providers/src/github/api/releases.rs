use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct ReleasesApi;

impl ReleasesApi {
    pub async fn list(
        client: &ProviderClient,
        repo: &str,
        limit: u32,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!("{}?per_page={limit}", repo_path(repo, &["releases"]));

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list releases: {e}")))?;

        Ok(data)
    }

    pub async fn fetch_by_tag(
        client: &ProviderClient,
        repo: &str,
        tag: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["releases", "tags", tag]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to fetch release by tag: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        repo: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["releases"]);

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create release: {e}")))?;

        Ok(data)
    }

    pub async fn update(
        client: &ProviderClient,
        repo: &str,
        release_id: u64,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["releases", &release_id.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update release: {e}")))?;

        Ok(data)
    }

    pub async fn delete(
        client: &ProviderClient,
        repo: &str,
        release_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["releases", &release_id.to_string()]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn delete_asset(
        client: &ProviderClient,
        repo: &str,
        asset_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["releases", "assets", &asset_id.to_string()]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_releases_list_endpoint() {
        let repo = "owner/repo";
        let endpoint = format!("{}?per_page=30", repo_path(repo, &["releases"]));

        assert_eq!(endpoint, "/repos/owner/repo/releases?per_page=30");
    }

    #[test]
    fn test_releases_fetch_by_tag_endpoint() {
        let repo = "owner/repo";
        let tag = "v1.0.0";
        let endpoint = repo_path(repo, &["releases", "tags", tag]);

        assert_eq!(endpoint, "/repos/owner/repo/releases/tags/v1.0.0");
    }

    #[test]
    fn test_releases_delete_endpoint() {
        let repo = "owner/repo";
        let release_id: u64 = 42;
        let endpoint = repo_path(repo, &["releases", &release_id.to_string()]);

        assert_eq!(endpoint, "/repos/owner/repo/releases/42");
    }
}
