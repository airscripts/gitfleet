use gitfleet_core::errors::{GitfleetError, NotFoundError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct AccessApi;

impl AccessApi {
    pub async fn invite_group_member(
        client: &ProviderClient,
        group: &str,
        username: &str,
        role: &str,
    ) -> Result<(), GitfleetError> {
        let encoded = urlencoding::encode(group);
        let endpoint = format!("/groups/{encoded}/members");
        let access_level = access_level(role, true);
        let body = serde_json::json!({
            "username": username,
            "access_level": access_level,
        });

        client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn invite_member(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        username: &str,
        permission: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let endpoint = format!("/projects/{encoded}/members");

        let access_level = access_level(permission, false);

        let body = serde_json::json!({
            "username": username,
            "access_level": access_level,
        });

        client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn list_group_members(
        client: &ProviderClient,
        group: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let enc_group = urlencoding::encode(group);

        let endpoint = format!("/groups/{enc_group}/members");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let mut data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list group members: {e}")))?;

        if let Some(members) = data.as_array_mut() {
            for member in members {
                normalize_member(member);
            }
        }

        Ok(data)
    }

    pub async fn remove_member(
        client: &ProviderClient,
        group: &str,
        username: &str,
    ) -> Result<(), GitfleetError> {
        let user_id = resolve_user_id(client, username).await?;
        let enc_group = urlencoding::encode(group);
        let endpoint = format!("/groups/{enc_group}/members/{user_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn list_teams(
        _client: &ProviderClient,
        _group: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(teams_unsupported())
    }

    pub async fn create_team(
        _client: &ProviderClient,
        _group: &str,
        _name: &str,
        _description: &str,
        _privacy: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(teams_unsupported())
    }

    pub async fn list_team_members(
        _client: &ProviderClient,
        _group: &str,
        _team_slug: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(teams_unsupported())
    }
}

fn access_level(role: &str, group: bool) -> u64 {
    match role {
        "admin" | "owner" if group => 50,
        "admin" | "maintainer" => 40,
        "member" | "developer" => 30,
        "reporter" => 20,
        "guest" => 10,
        _ => 30,
    }
}

async fn resolve_user_id(client: &ProviderClient, username: &str) -> Result<u64, GitfleetError> {
    let endpoint = format!("/users?username={}", urlencoding::encode(username));
    let response = client
        .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
        .await?;
    let users: Vec<serde_json::Value> = crate::parse_json(response)
        .await
        .map_err(|e| GitfleetError::new(format!("Failed to resolve user: {e}")))?;

    users
        .iter()
        .find(|user| user.get("username").and_then(serde_json::Value::as_str) == Some(username))
        .and_then(|user| user.get("id"))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            GitfleetError::from(NotFoundError::new(format!(
                "User '{username}' was not found."
            )))
        })
}

fn normalize_member(member: &mut serde_json::Value) {
    let Some(object) = member.as_object_mut() else {
        return;
    };
    let login = object
        .get("username")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    object.insert("login".to_string(), login);
    object.insert(
        "type".to_string(),
        serde_json::Value::String("User".to_string()),
    );
}

fn teams_unsupported() -> GitfleetError {
    GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitLab,
        ProviderCapability::Access,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_level_mapping() {
        assert_eq!(access_level("admin", false), 40);
        assert_eq!(access_level("maintainer", false), 40);
        assert_eq!(access_level("developer", false), 30);
        assert_eq!(access_level("reporter", false), 20);
        assert_eq!(access_level("guest", false), 10);
        assert_eq!(access_level("unknown", false), 30);
        assert_eq!(access_level("owner", true), 50);
    }
}
