use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct AttestationsApi;

impl AttestationsApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
        _subject_digest: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/artifacts");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list attestations: {e}")))?;

        Ok(data)
    }

    pub async fn get(
        client: &ProviderClient,
        project: &str,
        attestation_id: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/artifacts/{attestation_id}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get attestation: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_attestations_encode_path() {
        assert_eq!(urlencoding::encode("org/repo").to_string(), "org%2Frepo");
    }
}
