use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct ReleasesApi;

impl ReleasesApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
        limit: u32,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/releases?per_page={limit}");

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
        project: &str,
        tag: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let enc_tag = urlencoding::encode(tag);
        let endpoint = format!("/projects/{encoded}/releases/{enc_tag}");

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
        project: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/releases");

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
        project: &str,
        release_id: u64,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/releases/{release_id}");

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update release: {e}")))?;

        Ok(data)
    }

    pub async fn delete(
        client: &ProviderClient,
        project: &str,
        release_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/releases/{release_id}");

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
    fn test_encode_path_release() {
        assert_eq!(encode_path("org/repo"), "org%2Frepo");

        assert_eq!(encode_path("simple-proj"), "simple-proj");
    }

    #[test]
    fn test_release_list_endpoint() {
        let project = "org/repo";
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/releases?per_page=20");

        assert_eq!(endpoint, "/projects/org%2Frepo/releases?per_page=20");
    }
}
