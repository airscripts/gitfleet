use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::EnvironmentListResponse;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct EnvironmentsApi;

impl EnvironmentsApi {
    pub async fn list(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
    ) -> Result<EnvironmentListResponse, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let endpoint = format!("/projects/{encoded}/environments");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: EnvironmentListResponse = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list environments: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
        _wait_timer: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let endpoint = format!("/projects/{encoded}/environments");

        let body = serde_json::json!({ "name": name });
        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create environment: {e}")))?;

        Ok(data)
    }

    pub async fn delete(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let enc_name = urlencoding::encode(name);

        let endpoint = format!("/projects/{encoded}/environments/{enc_name}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_environments_create_body() {
        let name = "production";
        let body = serde_json::json!({ "name": name });

        assert_eq!(body["name"], "production");
    }

    #[test]
    fn test_environments_encode_path() {
        let full = "owner/repo";
        let encoded = urlencoding::encode(full).to_string();

        assert_eq!(encoded, "owner%2Frepo");
    }

    #[test]
    fn test_environments_delete_endpoint() {
        let full = "owner/repo";
        let encoded = urlencoding::encode(full).to_string();

        let name = "production";
        let enc_name = urlencoding::encode(name);

        let endpoint = format!("/projects/{encoded}/environments/{enc_name}");

        assert_eq!(endpoint, "/projects/owner%2Frepo/environments/production");
    }
}
