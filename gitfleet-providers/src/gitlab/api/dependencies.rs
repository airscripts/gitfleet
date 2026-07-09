use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct DependenciesApi;

impl DependenciesApi {
    pub async fn sbom(
        client: &ProviderClient,
        project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/packages");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get SBOM: {e}")))?;

        Ok(data)
    }

    pub async fn review(
        client: &ProviderClient,
        project: &str,
        base: &str,
        head: &str,
    ) -> Result<Vec<gitfleet_core::types::DependencyReviewChange>, GitfleetError> {
        let encoded = encode_path(project);

        let enc_base = urlencoding::encode(base);
        let enc_head = urlencoding::encode(head);

        let endpoint =
            format!("/projects/{encoded}/repository/compare?from={enc_base}&to={enc_head}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to compare dependencies: {e}")))?;

        let changes = data
            .get("diffs")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(changes
            .iter()
            .map(|raw| gitfleet_core::types::DependencyReviewChange {
                change_type: raw
                    .get("new_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                package: String::new(),
                ecosystem: String::new(),
                version: String::new(),
                severity: String::new(),
                vulnerabilities: 0,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_dependencies_encode_path() {
        assert_eq!(urlencoding::encode("org/repo").to_string(), "org%2Frepo");
    }
}
