use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{LicenseDetail, LicenseSummary};

use crate::gitlab::client::ProviderClient;

pub struct LicensesApi;

impl LicensesApi {
    pub async fn list(client: &ProviderClient) -> Result<Vec<LicenseSummary>, GitfleetError> {
        let endpoint = "/licenses";

        let response = client
            .request_token_required(reqwest::Method::GET, endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list licenses: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| LicenseSummary {
                key: raw
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: raw
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                spdx_id: raw
                    .get("spdx_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                url: raw
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }

    pub async fn get(client: &ProviderClient, key: &str) -> Result<LicenseDetail, GitfleetError> {
        let endpoint = format!("/licenses/{key}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get license: {e}")))?;

        Ok(LicenseDetail {
            key: raw
                .get("key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            name: raw
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            spdx_id: raw
                .get("spdx_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            url: raw
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: raw
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            implementation: raw
                .get("implementation")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            permissions: Vec::new(),
            conditions: Vec::new(),
            limitations: Vec::new(),
            body: raw
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    pub async fn repo_license(
        client: &ProviderClient,
        project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = urlencoding::encode(project).to_string();

        let endpoint = format!("/projects/{encoded}?license=true");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get repo license: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_licenses_list_endpoint() {
        let endpoint = "/licenses";

        assert_eq!(endpoint, "/licenses");
    }

    #[test]
    fn test_gitlab_licenses_repo_license_endpoint() {
        let project = "org/repo";
        let encoded = urlencoding::encode(project).to_string();

        let endpoint = format!("/projects/{encoded}?license=true");

        assert!(endpoint.contains("org%2Frepo"));
    }
}
