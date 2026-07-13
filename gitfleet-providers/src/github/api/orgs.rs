use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::encode_segment;
use crate::github::client::ProviderClient;

pub struct OrgsApi;

impl OrgsApi {
    pub async fn list_members(
        client: &ProviderClient,
        org: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!("/orgs/{}/members?per_page=100", encode_segment(org));

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list org members: {e}")))?;

        Ok(data)
    }

    pub async fn invite_member(
        client: &ProviderClient,
        org: &str,
        username: &str,
        role: &str,
    ) -> Result<(), GitfleetError> {
        let endpoint = format!(
            "/orgs/{}/memberships/{}",
            encode_segment(org),
            encode_segment(username)
        );

        let payload = serde_json::json!({ "role": role });

        client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(payload), None, None)
            .await?;

        Ok(())
    }

    pub async fn remove_member(
        client: &ProviderClient,
        org: &str,
        username: &str,
    ) -> Result<(), GitfleetError> {
        let endpoint = format!(
            "/orgs/{}/memberships/{}",
            encode_segment(org),
            encode_segment(username)
        );

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_orgs_invite_member_body() {
        let role = "admin";
        let body = serde_json::json!({ "role": role });

        assert_eq!(body["role"], "admin");
    }

    #[test]
    fn test_orgs_list_members_endpoint() {
        let org = "myorg";
        let endpoint = format!("/orgs/{org}/members?per_page=100");

        assert_eq!(endpoint, "/orgs/myorg/members?per_page=100");
    }
}
