use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct AdvisoriesApi;

impl AdvisoriesApi {
    pub async fn list_vulnerabilities(
        client: &ProviderClient,
        project: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint = format!("/projects/{encoded}/vulnerability_findings");

        if let Some(s) = state {
            endpoint.push_str(&format!("?state={s}"));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list vulnerabilities: {e}")))?;

        Ok(data)
    }

    pub async fn list_dependabot_alerts(
        client: &ProviderClient,
        project: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Self::list_vulnerabilities(client, project, state).await
    }

    pub async fn list_codeql_alerts(
        client: &ProviderClient,
        project: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint = format!("/projects/{encoded}/vulnerability_findings?report_type=sast");

        if let Some(s) = state {
            endpoint.push_str(&format!("&state={s}"));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list SAST alerts: {e}")))?;

        Ok(data)
    }

    pub async fn list_secret_scanning_alerts(
        client: &ProviderClient,
        project: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint =
            format!("/projects/{encoded}/vulnerability_findings?report_type=secret_detection");

        if let Some(s) = state {
            endpoint.push_str(&format!("&state={s}"));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response.json().await.map_err(|e| {
            GitfleetError::new(format!("Failed to list secret scanning alerts: {e}"))
        })?;

        Ok(data)
    }

    pub async fn get_alert(
        client: &ProviderClient,
        project: &str,
        number: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/vulnerabilities/{number}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get vulnerability: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_advisories_encode_path() {
        assert_eq!(urlencoding::encode("org/repo").to_string(), "org%2Frepo");
    }
}
