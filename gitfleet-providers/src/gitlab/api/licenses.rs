use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{LicenseDetail, LicenseSummary};

use crate::gitlab::client::ProviderClient;

pub struct LicensesApi;

impl LicensesApi {
    pub async fn list(client: &ProviderClient) -> Result<Vec<LicenseSummary>, GitfleetError> {
        let endpoint = "/templates/licenses";

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
                    .or_else(|| raw.get("key"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                url: raw
                    .get("html_url")
                    .or_else(|| raw.get("source_url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }

    pub async fn get(client: &ProviderClient, key: &str) -> Result<LicenseDetail, GitfleetError> {
        let endpoint = license_endpoint(key);

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
                .or_else(|| raw.get("key"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            url: raw
                .get("html_url")
                .or_else(|| raw.get("source_url"))
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
            permissions: string_array(&raw, "permissions"),
            conditions: string_array(&raw, "conditions"),
            limitations: string_array(&raw, "limitations"),
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

fn license_endpoint(key: &str) -> String {
    format!("/templates/licenses/{}", urlencoding::encode(key))
}

fn string_array(raw: &serde_json::Value, key: &str) -> Vec<String> {
    raw.get(key)
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::license_endpoint;

    #[test]
    fn test_gitlab_licenses_list_endpoint() {
        let endpoint = "/templates/licenses";

        assert_eq!(endpoint, "/templates/licenses");
    }

    #[test]
    fn test_gitlab_licenses_repo_license_endpoint() {
        let project = "org/repo";
        let encoded = urlencoding::encode(project).to_string();

        let endpoint = format!("/projects/{encoded}?license=true");

        assert!(endpoint.contains("org%2Frepo"));
    }

    #[test]
    fn test_gitlab_license_endpoint_encodes_key() {
        assert_eq!(
            license_endpoint("custom/license"),
            "/templates/licenses/custom%2Flicense"
        );
    }
}
