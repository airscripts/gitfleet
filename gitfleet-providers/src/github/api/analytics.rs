use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct AnalyticsApi;

impl AnalyticsApi {
    pub async fn get_traffic_views(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["traffic", "views"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get traffic views: {e}")))?;

        Ok(data)
    }

    pub async fn get_traffic_clones(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["traffic", "clones"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get traffic clones: {e}")))?;

        Ok(data)
    }

    pub async fn get_referrers(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["traffic", "popular", "referrers"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get referrers: {e}")))?;

        Ok(data)
    }

    pub async fn get_popular_paths(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["traffic", "popular", "paths"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get popular paths: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_traffic_views_endpoint() {
        let repo = "owner/repo";
        let endpoint = repo_path(repo, &["traffic", "views"]);

        assert_eq!(endpoint, "/repos/owner/repo/traffic/views");
    }

    #[test]
    fn test_analytics_traffic_clones_endpoint() {
        let repo = "owner/repo";
        let endpoint = repo_path(repo, &["traffic", "clones"]);

        assert_eq!(endpoint, "/repos/owner/repo/traffic/clones");
    }
}
