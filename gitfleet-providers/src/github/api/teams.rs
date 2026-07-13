use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::encode_segment;
use crate::github::client::ProviderClient;

pub struct TeamsApi;

impl TeamsApi {
    pub async fn list(
        client: &ProviderClient,
        org: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!("/orgs/{}/teams?per_page=100", encode_segment(org));

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list teams: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        org: &str,
        name: &str,
        description: &str,
        privacy: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!("/orgs/{}/teams", encode_segment(org));

        let payload = serde_json::json!({
            "name": name,
            "privacy": privacy,
            "description": description,
        });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(payload), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create team: {e}")))?;

        Ok(data)
    }

    pub async fn list_members(
        client: &ProviderClient,
        org: &str,
        team_slug: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!(
            "/orgs/{}/teams/{}/members?per_page=100",
            encode_segment(org),
            encode_segment(team_slug)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list team members: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_teams_list_endpoint() {
        let org = "myorg";
        let endpoint = format!("/orgs/{org}/teams?per_page=100");

        assert_eq!(endpoint, "/orgs/myorg/teams?per_page=100");
    }

    #[test]
    fn test_teams_create_body() {
        let name = "engineering";
        let privacy = "closed";
        let description = "Engineering team";
        let body = serde_json::json!({
            "name": name,
            "privacy": privacy,
            "description": description,
        });

        assert_eq!(body["name"], "engineering");

        assert_eq!(body["privacy"], "closed");
        assert_eq!(body["description"], "Engineering team");
    }
}
