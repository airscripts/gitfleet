use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct SiteApi;

impl SiteApi {
    pub async fn get_pages(
        client: &ProviderClient,
        project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pages");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get pages: {e}")))?;

        Ok(data)
    }

    pub async fn create_pages(
        _client: &ProviderClient,
        _project: &str,
        _source: &str,
        _build_type: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(GitfleetError::from(UnsupportedCapabilityError::new(
            ProviderId::GitLab,
            ProviderCapability::Site,
        )))
    }

    pub async fn remove_pages(client: &ProviderClient, project: &str) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pages");

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
    fn test_encode_path() {
        assert_eq!(encode_path("org/repo"), "org%2Frepo");
    }
}
