use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::RepoSummary;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct ProjectsApi;

impl ProjectsApi {
    pub async fn list_group(
        client: &ProviderClient,
        group: &str,
    ) -> Result<Vec<RepoSummary>, GitfleetError> {
        let enc_group = urlencoding::encode(group);

        let endpoint = format!("/groups/{enc_group}/projects?per_page=100");

        let data: Vec<serde_json::Value> = client.get_paginated(&endpoint, None, None).await?;

        Ok(data.iter().map(normalize_project).collect())
    }

    pub async fn list_user(client: &ProviderClient) -> Result<Vec<RepoSummary>, GitfleetError> {
        let endpoint = "/projects?membership=true&per_page=100&order_by=updated_at";

        let data: Vec<serde_json::Value> = client.get_paginated(endpoint, None, None).await?;

        Ok(data.iter().map(normalize_project).collect())
    }

    pub async fn list_user_named(
        client: &ProviderClient,
        username: &str,
    ) -> Result<Vec<RepoSummary>, GitfleetError> {
        let enc_user = urlencoding::encode(username);

        let endpoint = format!("/users/{enc_user}/projects?per_page=100&order_by=updated_at");

        let data: Vec<serde_json::Value> = client.get_paginated(&endpoint, None, None).await?;

        Ok(data.iter().map(normalize_project).collect())
    }

    pub async fn get(
        client: &ProviderClient,
        project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}");

        let response = client
            .request_optional_token(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let mut data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get project: {e}")))?;

        normalize_project_value(&mut data);

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        name: &str,
        visibility: &str,
        owner: Option<&str>,
        owner_type: Option<&str>,
        description: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut body = serde_json::json!({
            "name": name,
            "visibility": visibility,
        });

        if let Some(d) = description {
            body["description"] = serde_json::Value::String(d.to_string());
        }

        if matches!(owner_type, Some("org" | "group")) {
            let owner = owner.ok_or_else(|| {
                GitfleetError::new("A GitLab group is required for group-owned projects.")
            })?;
            let group_endpoint = format!("/groups/{}", urlencoding::encode(owner));

            let response = client
                .request_token_required(reqwest::Method::GET, &group_endpoint, None, None, None)
                .await?;

            let group: serde_json::Value = crate::parse_json(response)
                .await
                .map_err(|e| GitfleetError::new(format!("Failed to resolve group: {e}")))?;
            let namespace_id = group
                .get("id")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| {
                    GitfleetError::new("GitLab group response did not include an ID.")
                })?;

            body["namespace_id"] = serde_json::json!(namespace_id);
        }

        let endpoint = "/projects";

        let response = client
            .request_token_required(reqwest::Method::POST, endpoint, Some(body), None, None)
            .await?;

        let mut data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create project: {e}")))?;

        normalize_project_value(&mut data);

        Ok(data)
    }

    pub async fn update(
        client: &ProviderClient,
        project: &str,
        mut options: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}");

        if options.get("homepage").is_some() {
            return Err(GitfleetError::new(
                "GitLab does not support repository homepage metadata.",
            ));
        }

        if let Some(name) = options.get("name").cloned() {
            options["path"] = name;
        }

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(options), None, None)
            .await?;

        let mut data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update project: {e}")))?;

        normalize_project_value(&mut data);

        Ok(data)
    }

    pub async fn delete(client: &ProviderClient, project: &str) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn star(client: &ProviderClient, project: &str) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/star");

        client
            .request_token_required(
                reqwest::Method::POST,
                &endpoint,
                Some(serde_json::json!({})),
                None,
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn unstar(client: &ProviderClient, project: &str) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/unstar");

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn fork(
        client: &ProviderClient,
        project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/fork");

        let response = client
            .request_token_required(
                reqwest::Method::POST,
                &endpoint,
                Some(serde_json::json!({})),
                None,
                None,
            )
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to fork project: {e}")))?;

        Ok(data)
    }

    pub async fn archive(client: &ProviderClient, project: &str) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/archive");

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn unarchive(client: &ProviderClient, project: &str) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/unarchive");

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_project(raw: &serde_json::Value) -> RepoSummary {
    RepoSummary {
        id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        name: raw
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        fork: raw.get("forked_from_project").is_some(),
        full_name: raw
            .get("path_with_namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        private: raw
            .get("visibility")
            .and_then(|v| v.as_str())
            .map(|v| v == "private")
            .unwrap_or(false),
        archived: raw
            .get("archived")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        default_branch: raw
            .get("default_branch")
            .and_then(|v| v.as_str())
            .unwrap_or("main")
            .to_string(),
        pushed_at: raw
            .get("last_activity_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    }
}

fn normalize_project_value(raw: &mut serde_json::Value) {
    let Some(object) = raw.as_object_mut() else {
        return;
    };

    let full_name = object
        .get("path_with_namespace")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let html_url = object
        .get("web_url")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let private = object
        .get("visibility")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|visibility| visibility == "private");

    object.insert("full_name".to_string(), full_name);
    object.insert("html_url".to_string(), html_url);
    object.insert("private".to_string(), serde_json::Value::Bool(private));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_project_full() {
        let json = serde_json::json!({
            "id": 42,
            "name": "my-project",
            "path_with_namespace": "group/my-project",
            "visibility": "private",
            "archived": false,
            "default_branch": "main",
            "last_activity_at": "2024-06-01T00:00:00Z",
            "forked_from_project": { "id": 1, "name": "original" }
        });

        let result = normalize_project(&json);

        assert_eq!(result.id, 42);

        assert_eq!(result.name, "my-project");
        assert!(result.fork);

        assert_eq!(result.full_name, "group/my-project");
        assert!(result.private);

        assert!(!result.archived);
        assert_eq!(result.default_branch, "main");

        assert_eq!(result.pushed_at, Some("2024-06-01T00:00:00Z".to_string()));
    }

    #[test]
    fn test_normalize_project_minimal() {
        let json = serde_json::json!({
            "id": 1,
            "name": "minimal",
            "path_with_namespace": "user/minimal",
            "visibility": "public"
        });

        let result = normalize_project(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.name, "minimal");
        assert!(!result.fork);

        assert_eq!(result.full_name, "user/minimal");
        assert!(!result.private);

        assert!(!result.archived);
        assert_eq!(result.default_branch, "main");

        assert!(result.pushed_at.is_none());
    }
}
