use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::RulesetInput;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct GovernanceApi;

impl GovernanceApi {
    pub async fn list_rulesets(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["rulesets"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list rulesets: {e}")))?;

        Ok(data)
    }

    pub async fn create_ruleset(
        client: &ProviderClient,
        repo: &str,
        input: &RulesetInput,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["rulesets"]);

        let body = serde_json::to_value(input)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize ruleset: {e}")))?;

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create ruleset: {e}")))?;

        Ok(data)
    }

    pub async fn update_ruleset(
        client: &ProviderClient,
        repo: &str,
        ruleset_id: u64,
        input: &RulesetInput,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["rulesets", &ruleset_id.to_string()]);

        let body = serde_json::to_value(input)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize ruleset: {e}")))?;

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update ruleset: {e}")))?;

        Ok(data)
    }

    pub async fn delete_ruleset(
        client: &ProviderClient,
        repo: &str,
        ruleset_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["rulesets", &ruleset_id.to_string()]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::RulesetInput;

    #[test]
    fn test_ruleset_input_type() {
        let input = RulesetInput {
            name: "main-protection".to_string(),
            target: Some("branch".to_string()),
            rules: None,
            enforcement: Some("active".to_string()),
            conditions: None,
        };

        assert_eq!(input.name, "main-protection");

        assert_eq!(input.enforcement, Some("active".to_string()));
    }
}
