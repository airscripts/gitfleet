use gitfleet_core::errors::GitfleetError;

use crate::github::client::ProviderClient;

pub struct ProtectionApi;

impl ProtectionApi {
    pub async fn get_branch_protection(
        client: &ProviderClient,
        repo: &str,
        branch: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!(
            "/repos/{repo}/branches/{}/protection",
            urlencoding::encode(branch)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get branch protection: {e}")))?;

        Ok(data)
    }

    pub async fn protect_branch(
        client: &ProviderClient,
        repo: &str,
        branch: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!(
            "/repos/{repo}/branches/{}/protection",
            urlencoding::encode(branch)
        );

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(input), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to set branch protection: {e}")))?;

        Ok(data)
    }

    pub async fn unprotect_branch(
        client: &ProviderClient,
        repo: &str,
        branch: &str,
    ) -> Result<(), GitfleetError> {
        let endpoint = format!(
            "/repos/{repo}/branches/{}/protection",
            urlencoding::encode(branch)
        );

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}
