use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::RepoSummary;

use crate::github::client::ProviderClient;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitHubRepoResponse {
    pub id: u64,
    pub name: String,
    pub fork: bool,
    pub private: bool,
    pub archived: bool,
    pub full_name: String,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub clone_url: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    pub default_branch: String,
    #[serde(default)]
    pub pushed_at: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub owner: Option<GitHubRepoOwner>,
    #[serde(default)]
    pub open_issues_count: Option<u64>,
    #[serde(default)]
    pub stargazers_count: Option<u64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent: Option<GitHubRepoParent>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitHubRepoOwner {
    pub login: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitHubRepoParent {
    pub full_name: String,
}

fn normalize_repo(repo: &GitHubRepoResponse) -> RepoSummary {
    RepoSummary {
        id: repo.id,
        name: repo.name.clone(),
        fork: repo.fork,
        private: repo.private,
        archived: repo.archived,
        full_name: repo.full_name.clone(),
        pushed_at: repo.pushed_at.clone(),
        default_branch: repo.default_branch.clone(),
    }
}

pub struct ReposApi;

impl ReposApi {
    pub async fn fetch_org(
        client: &ProviderClient,
        org: &str,
    ) -> Result<Vec<RepoSummary>, GitfleetError> {
        let endpoint = format!("/orgs/{org}/repos?per_page=100&type=all");

        let data: Vec<GitHubRepoResponse> = client.get_paginated(&endpoint, None, None).await?;
        Ok(data.iter().map(normalize_repo).collect())
    }

    pub async fn fetch_user_repos(
        client: &ProviderClient,
    ) -> Result<Vec<RepoSummary>, GitfleetError> {
        let endpoint = "/user/repos?per_page=100&sort=updated";

        let data: Vec<GitHubRepoResponse> = client.get_paginated(endpoint, None, None).await?;

        Ok(data.iter().map(normalize_repo).collect())
    }

    pub async fn fetch_user(
        client: &ProviderClient,
        username: &str,
    ) -> Result<Vec<RepoSummary>, GitfleetError> {
        let endpoint = format!("/users/{username}/repos?per_page=100&type=all");

        let data: Vec<GitHubRepoResponse> = client.get_paginated(&endpoint, None, None).await?;
        Ok(data.iter().map(normalize_repo).collect())
    }

    pub async fn get(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<GitHubRepoResponse, GitfleetError> {
        let endpoint = format!("/repos/{repo}");

        let response = client
            .request_optional_token(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: GitHubRepoResponse = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse repository: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        name: &str,
        visibility: &str,
        owner: Option<&str>,
        owner_type: Option<&str>,
        description: Option<&str>,
    ) -> Result<GitHubRepoResponse, GitfleetError> {
        let body = serde_json::json!({
            "name": name,
            "visibility": visibility,
            "description": description,
        });

        let endpoint = match owner_type {
            Some("org") => format!("/orgs/{}/repos", owner.unwrap_or("")),
            _ => "/user/repos".to_string(),
        };

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: GitHubRepoResponse = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create repository: {e}")))?;

        Ok(data)
    }

    pub async fn update(
        client: &ProviderClient,
        repo: &str,
        options: serde_json::Value,
    ) -> Result<GitHubRepoResponse, GitfleetError> {
        let endpoint = format!("/repos/{repo}");

        let response = client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(options), None, None)
            .await?;

        let data: GitHubRepoResponse = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update repository: {e}")))?;

        Ok(data)
    }

    pub async fn delete(client: &ProviderClient, repo: &str) -> Result<(), GitfleetError> {
        let endpoint = format!("/repos/{repo}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn star(client: &ProviderClient, repo: &str) -> Result<(), GitfleetError> {
        let endpoint = format!("/user/starred/{repo}");

        client
            .request_token_required(
                reqwest::Method::PUT,
                &endpoint,
                Some(serde_json::json!({})),
                None,
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn unstar(client: &ProviderClient, repo: &str) -> Result<(), GitfleetError> {
        let endpoint = format!("/user/starred/{repo}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn fork(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<GitHubRepoResponse, GitfleetError> {
        let endpoint = format!("/repos/{repo}/forks");

        let response = client
            .request_token_required(
                reqwest::Method::POST,
                &endpoint,
                Some(serde_json::json!({})),
                None,
                None,
            )
            .await?;

        let data: GitHubRepoResponse = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to fork repository: {e}")))?;

        Ok(data)
    }

    pub async fn archive(client: &ProviderClient, repo: &str) -> Result<(), GitfleetError> {
        let endpoint = format!("/repos/{repo}");

        let body = serde_json::json!({ "archived": true });

        client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn unarchive(client: &ProviderClient, repo: &str) -> Result<(), GitfleetError> {
        let endpoint = format!("/repos/{repo}");

        let body = serde_json::json!({ "archived": false });

        client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_repo_full() {
        let json = serde_json::json!({
            "id": 12345,
            "name": "my-repo",
            "fork": false,
            "private": true,
            "archived": false,
            "full_name": "testorg/my-repo",
            "html_url": "https://github.com/testorg/my-repo",
            "clone_url": "https://github.com/testorg/my-repo.git",
            "visibility": "private",
            "default_branch": "main",
            "pushed_at": "2024-06-01T00:00:00Z",
            "homepage": "https://example.com",
            "owner": { "login": "testorg" },
            "open_issues_count": 5,
            "stargazers_count": 10,
            "description": "A test repository",
            "parent": null
        });

        let repo: GitHubRepoResponse = serde_json::from_value(json).unwrap();

        let result = normalize_repo(&repo);

        assert_eq!(result.id, 12345);

        assert_eq!(result.name, "my-repo");
        assert!(!result.fork);

        assert!(result.private);
        assert!(!result.archived);

        assert_eq!(result.full_name, "testorg/my-repo");
        assert_eq!(result.pushed_at, Some("2024-06-01T00:00:00Z".to_string()));

        assert_eq!(result.default_branch, "main");
    }

    #[test]
    fn test_normalize_repo_minimal() {
        let json = serde_json::json!({
            "id": 1,
            "name": "minimal",
            "fork": false,
            "private": false,
            "archived": false,
            "full_name": "user/minimal",
            "default_branch": "master"
        });

        let repo: GitHubRepoResponse = serde_json::from_value(json).unwrap();

        let result = normalize_repo(&repo);

        assert_eq!(result.id, 1);

        assert_eq!(result.name, "minimal");
        assert!(!result.fork);

        assert!(!result.private);
        assert!(!result.archived);

        assert_eq!(result.full_name, "user/minimal");
        assert!(result.pushed_at.is_none());

        assert_eq!(result.default_branch, "master");
    }

    #[test]
    fn test_github_repo_owner_deserialize() {
        let json = serde_json::json!({ "login": "octocat" });

        let owner: GitHubRepoOwner = serde_json::from_value(json).unwrap();

        assert_eq!(owner.login, "octocat");
    }

    #[test]
    fn test_github_repo_parent_deserialize() {
        let json = serde_json::json!({ "full_name": "original/repo" });

        let parent: GitHubRepoParent = serde_json::from_value(json).unwrap();

        assert_eq!(parent.full_name, "original/repo");
    }
}
