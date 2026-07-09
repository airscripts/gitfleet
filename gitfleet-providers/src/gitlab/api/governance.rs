use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct GovernanceApi;

impl GovernanceApi {
    pub async fn list_rulesets(
        client: &ProviderClient,
        project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/push_rules");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list push rules: {e}")))?;

        Ok(data)
    }

    pub async fn create_ruleset(
        client: &ProviderClient,
        project: &str,
        input: &gitfleet_core::types::RulesetInput,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/push_rules");

        let body = serde_json::to_value(input)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize push rule: {e}")))?;

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create push rule: {e}")))?;

        Ok(data)
    }

    pub async fn delete_ruleset(
        client: &ProviderClient,
        project: &str,
        ruleset_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/push_rules/{ruleset_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_governance_encode_path() {
        assert_eq!(urlencoding::encode("org/repo").to_string(), "org%2Frepo");
    }
}
