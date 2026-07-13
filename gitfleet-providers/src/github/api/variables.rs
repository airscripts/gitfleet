use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{RepoVariable, VariableListResponse};

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct VariablesApi;

impl VariablesApi {
    pub async fn list_repo(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
    ) -> Result<VariableListResponse<RepoVariable>, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["actions", "variables"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: VariableListResponse<RepoVariable> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list repo variables: {e}")))?;

        Ok(data)
    }

    pub async fn set_repo(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
        value: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["actions", "variables"]);

        let body = serde_json::json!({
            "name": name,
            "value": value,
        });

        let result = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(GitfleetError::Other(msg)) if msg.contains(": 409") => {
                Self::update_repo(client, owner, repo, name, value).await
            }

            Err(e) => Err(e),
        }
    }

    pub async fn update_repo(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
        value: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["actions", "variables", name]);

        let body = serde_json::json!({
            "name": name,
            "value": value,
        });

        client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn delete_repo(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["actions", "variables", name]);

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
    fn test_variables_set_body() {
        let name = "MY_VAR";
        let value = "hello";
        let body = serde_json::json!({
            "name": name,
            "value": value,
        });

        assert_eq!(body["name"], "MY_VAR");

        assert_eq!(body["value"], "hello");
    }

    #[test]
    fn test_variables_repo_path() {
        let full = "owner/repo";
        let endpoint = repo_path(full, &["actions", "variables"]);

        assert_eq!(endpoint, "/repos/owner/repo/actions/variables");
    }
}
