use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct ChecksApi;

impl ChecksApi {
    pub async fn list_check_suites(
        client: &ProviderClient,
        repo: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut endpoint = repo_path(repo, &["commits", "check-suites"]);

        if let Some(r) = r#ref {
            endpoint = repo_path(repo, &["commits", r, "check-suites"]);
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list check suites: {e}")))?;

        Ok(data)
    }

    pub async fn list_check_runs(
        client: &ProviderClient,
        repo: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = match r#ref {
            Some(r) => repo_path(repo, &["commits", r, "check-runs"]),
            None => repo_path(repo, &["check-runs"]),
        };

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list check runs: {e}")))?;

        Ok(data)
    }

    pub async fn get_check_run(
        client: &ProviderClient,
        repo: &str,
        check_run_id: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["check-runs", &check_run_id.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get check run: {e}")))?;

        Ok(data)
    }

    pub async fn list_check_run_annotations(
        client: &ProviderClient,
        repo: &str,
        check_run_id: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(
            repo,
            &["check-runs", &check_run_id.to_string(), "annotations"],
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response.json().await.map_err(|e| {
            GitfleetError::new(format!("Failed to list check run annotations: {e}"))
        })?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_suites_endpoint_with_ref() {
        let repo = "owner/repo";
        let r#ref = "main";
        let endpoint = repo_path(repo, &["commits", r#ref, "check-suites"]);

        assert_eq!(endpoint, "/repos/owner/repo/commits/main/check-suites");
    }

    #[test]
    fn test_check_suites_endpoint_without_ref() {
        let repo = "owner/repo";
        let endpoint = repo_path(repo, &["commits", "check-suites"]);

        assert_eq!(endpoint, "/repos/owner/repo/commits/check-suites");
    }

    #[test]
    fn test_check_runs_endpoint() {
        let repo = "owner/repo";
        let endpoint = repo_path(repo, &["check-runs", "123"]);

        assert_eq!(endpoint, "/repos/owner/repo/check-runs/123");
    }
}
