use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct AccessApi;

impl AccessApi {
    pub async fn invite_collaborator(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        username: &str,
        permission: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["collaborators", username]);

        let body = serde_json::json!({ "permission": permission });

        client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn list_teams(
        client: &ProviderClient,
        org: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!("/orgs/{org}/teams?per_page=100");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list teams: {e}")))?;

        Ok(data)
    }

    pub async fn add_team_repo(
        client: &ProviderClient,
        org: &str,
        team_slug: &str,
        owner: &str,
        repo: &str,
        permission: &str,
    ) -> Result<(), GitfleetError> {
        let endpoint = format!("/orgs/{org}/teams/{team_slug}/repos/{owner}/{repo}");

        let body = serde_json::json!({ "permission": permission });

        client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn list_org_members(
        client: &ProviderClient,
        org: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!("/orgs/{org}/members?per_page=100");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list org members: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_access_invite_body() {
        let permission = "admin";
        let body = serde_json::json!({ "permission": permission });

        assert_eq!(body["permission"], "admin");
    }

    #[test]
    fn test_access_team_repo_body() {
        let permission = "push";
        let body = serde_json::json!({ "permission": permission });

        assert_eq!(body["permission"], "push");
    }
}
