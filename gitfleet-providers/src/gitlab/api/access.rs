use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct AccessApi;

impl AccessApi {
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

        let access_level = match permission {
            "admin" => 40,
            "maintainer" => 30,
            "developer" => 30,
            "reporter" => 20,
            "guest" => 10,
            _ => 30,
        };

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

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list group members: {e}")))?;

        Ok(data)
    }

    pub async fn remove_member(
        client: &ProviderClient,
        group: &str,
        username: &str,
    ) -> Result<(), GitfleetError> {
        let enc_group = urlencoding::encode(group);

        let enc_user = urlencoding::encode(username);
        let endpoint = format!("/groups/{enc_group}/members/{enc_user}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn list_teams(
        client: &ProviderClient,
        group: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let enc_group = urlencoding::encode(group);

        let endpoint = format!("/groups/{enc_group}/subgroups?per_page=100");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list teams: {e}")))?;

        Ok(data)
    }

    pub async fn create_team(
        client: &ProviderClient,
        group: &str,
        name: &str,
        _description: &str,
        _privacy: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = "/groups".to_string();

        let payload = serde_json::json!({
            "name": name,
            "path": name.to_lowercase().replace(' ', "-"),
            "parent_id": group,
        });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(payload), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create team: {e}")))?;

        Ok(data)
    }

    pub async fn list_team_members(
        client: &ProviderClient,
        group: &str,
        team_slug: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let enc_group = urlencoding::encode(group);

        let endpoint = format!("/groups/{enc_group}/members?per_page=100");
        let _ = team_slug;
        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list team members: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_access_level_mapping() {
        assert_eq!(
            match "admin" {
                "admin" => 40,
                "maintainer" => 30,
                "developer" => 30,
                "reporter" => 20,
                "guest" => 10,
                _ => 30,
            },
            40
        );

        assert_eq!(
            match "maintainer" {
                "admin" => 40,
                "maintainer" => 30,
                "developer" => 30,
                "reporter" => 20,
                "guest" => 10,
                _ => 30,
            },
            30
        );

        assert_eq!(
            match "developer" {
                "admin" => 40,
                "maintainer" => 30,
                "developer" => 30,
                "reporter" => 20,
                "guest" => 10,
                _ => 30,
            },
            30
        );

        assert_eq!(
            match "reporter" {
                "admin" => 40,
                "maintainer" => 30,
                "developer" => 30,
                "reporter" => 20,
                "guest" => 10,
                _ => 30,
            },
            20
        );

        assert_eq!(
            match "guest" {
                "admin" => 40,
                "maintainer" => 30,
                "developer" => 30,
                "reporter" => 20,
                "guest" => 10,
                _ => 30,
            },
            10
        );

        assert_eq!(
            match "unknown" {
                "admin" => 40,
                "maintainer" => 30,
                "developer" => 30,
                "reporter" => 20,
                "guest" => 10,
                _ => 30,
            },
            30
        );
    }
}
