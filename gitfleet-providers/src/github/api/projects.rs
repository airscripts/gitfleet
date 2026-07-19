use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::ProjectSummary;

use crate::github::client::ProviderClient;

pub struct ProjectsApi;

impl ProjectsApi {
    pub async fn list(
        client: &ProviderClient,
        owner: &str,
        limit: u32,
        page: Option<u32>,
    ) -> Result<Vec<ProjectSummary>, GitfleetError> {
        let page = page.unwrap_or(1);
        let fetch_limit = limit.saturating_mul(page);
        let query = r#"
            query Projects($owner: String!, $limit: Int!) {
                repositoryOwner(login: $owner) {
                    ... on Organization {
                        projectsV2(first: $limit, orderBy: {field: UPDATED_AT, direction: DESC}) {
                            nodes { id number title shortDescription closed url updatedAt }
                        }
                    }
                    ... on User {
                        projectsV2(first: $limit, orderBy: {field: UPDATED_AT, direction: DESC}) {
                            nodes { id number title shortDescription closed url updatedAt }
                        }
                    }
                }
            }
        "#;
        let payload = serde_json::json!({
            "query": query,
            "variables": { "owner": owner, "limit": fetch_limit }
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/graphql", Some(payload), None, None)
            .await?;

        let data = crate::parse_graphql(response, "project listing")
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list projects: {e}")))?;

        let nodes = data
            .get("data")
            .and_then(|d| d.get("repositoryOwner"))
            .and_then(|o| o.get("projectsV2"))
            .and_then(|p| p.get("nodes"))
            .and_then(|n| n.as_array());

        let mut results = Vec::new();

        if let Some(nodes) = nodes {
            for n in nodes {
                results.push(normalize_project_summary(n));
            }
        }

        let start = ((page - 1) * limit) as usize;
        Ok(results
            .into_iter()
            .skip(start)
            .take(limit as usize)
            .collect())
    }

    pub async fn get(
        client: &ProviderClient,
        owner: &str,
        number: u64,
        limit: u32,
    ) -> Result<serde_json::Value, GitfleetError> {
        let query = r#"
            query Project($owner: String!, $number: Int!, $limit: Int!) {
                organization(login: $owner) {
                    projectV2(number: $number) {
                        id number title shortDescription closed url updatedAt
                        items(first: $limit) { nodes { id type content { ... on Issue { id number title url state repository { nameWithOwner } } ... on PullRequest { id number title url state repository { nameWithOwner } } ... on DraftIssue { id title body } } fieldValueByName(name: "Status") { ... on ProjectV2ItemFieldSingleSelectValue { name } } } }
                        fields(first: 100) { nodes { ... on ProjectV2Field { id name dataType } ... on ProjectV2SingleSelectField { id name dataType options { id name } } ... on ProjectV2IterationField { id name dataType } } }
                    }
                }

                user(login: $owner) {
                    projectV2(number: $number) {
                        id number title shortDescription closed url updatedAt
                        items(first: $limit) { nodes { id type content { ... on Issue { id number title url state repository { nameWithOwner } } ... on PullRequest { id number title url state repository { nameWithOwner } } ... on DraftIssue { id title body } } fieldValueByName(name: "Status") { ... on ProjectV2ItemFieldSingleSelectValue { name } } } }
                        fields(first: 100) { nodes { ... on ProjectV2Field { id name dataType } ... on ProjectV2SingleSelectField { id name dataType options { id name } } ... on ProjectV2IterationField { id name dataType } } }
                    }
                }
            }
        "#;
        let payload = serde_json::json!({
            "query": query,
            "variables": { "owner": owner, "number": number, "limit": limit }
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/graphql", Some(payload), None, None)
            .await?;

        let data = crate::parse_graphql(response, "project lookup")
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get project: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        owner: &str,
        title: &str,
    ) -> Result<ProjectSummary, GitfleetError> {
        let owner_query = r#"
            query ProjectOwner($owner: String!) {
                repositoryOwner(login: $owner) {
                    ... on Organization { id }
                    ... on User { id }
                }
            }
        "#;
        let owner_payload = serde_json::json!({
            "query": owner_query,
            "variables": { "owner": owner }
        });

        let owner_response = client
            .request_token_required(
                reqwest::Method::POST,
                "/graphql",
                Some(owner_payload),
                None,
                None,
            )
            .await?;

        let owner_data = crate::parse_graphql(owner_response, "project owner lookup")
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to resolve project owner: {e}")))?;
        let owner_id = owner_data
            .get("data")
            .and_then(|data| data.get("repositoryOwner"))
            .and_then(|owner| owner.get("id"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| GitfleetError::new(format!("Project owner '{owner}' was not found.")))?;

        let mutation = r#"
            mutation CreateProject($input: CreateProjectV2Input!) {
                createProjectV2(input: $input) { projectV2 { id number title url } }
            }
        "#;
        let payload = serde_json::json!({
            "query": mutation,
            "variables": { "input": { "ownerId": owner_id, "title": title } }
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/graphql", Some(payload), None, None)
            .await?;

        let data = crate::parse_graphql(response, "project creation")
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create project: {e}")))?;

        let project = data
            .get("data")
            .and_then(|d| d.get("createProjectV2"))
            .and_then(|c| c.get("projectV2"))
            .ok_or_else(|| GitfleetError::new("GitHub did not return the created project."))?;

        Ok(normalize_project_summary(project))
    }

    pub async fn get_by_id(
        client: &ProviderClient,
        project_id: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let query = r#"
            query GetProject($id: ID!) {
                node(id: $id) {
                    ... on ProjectV2 {
                        id number title shortDescription closed url updatedAt
                    }
                }
            }
        "#;
        let payload = serde_json::json!({
            "query": query,
            "variables": { "id": project_id }
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/graphql", Some(payload), None, None)
            .await?;

        let data = crate::parse_graphql(response, "project lookup")
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get project: {e}")))?;

        Ok(data)
    }

    pub async fn delete(client: &ProviderClient, project_id: &str) -> Result<(), GitfleetError> {
        let mutation = r#"
            mutation DeleteProject($input: DeleteProjectV2Input!) {
                deleteProjectV2(input: $input) { clientMutationId }
            }
        "#;
        let payload = serde_json::json!({
            "query": mutation,
            "variables": { "input": { "projectId": project_id } }
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/graphql", Some(payload), None, None)
            .await?;

        crate::parse_graphql(response, "project deletion").await?;

        Ok(())
    }
}

fn normalize_project_summary(raw: &serde_json::Value) -> ProjectSummary {
    ProjectSummary {
        id: raw
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        number: raw.get("number").and_then(|v| v.as_u64()).unwrap_or(0),
        title: raw
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        description: raw
            .get("shortDescription")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        closed: raw.get("closed").and_then(|v| v.as_bool()).unwrap_or(false),
        url: raw
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        updated_at: raw
            .get("updatedAt")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_project_summary_full() {
        let json = serde_json::json!({
            "id": "PVT_abc123",
            "number": 1,
            "title": "My Project",
            "shortDescription": "A project",
            "closed": false,
            "url": "https://github.com/orgs/example/projects/1",
            "updatedAt": "2024-06-01T00:00:00Z"
        });

        let result = normalize_project_summary(&json);

        assert_eq!(result.id, "PVT_abc123");

        assert_eq!(result.number, 1);
        assert_eq!(result.title, "My Project");

        assert_eq!(result.description, "A project");
        assert!(!result.closed);

        assert_eq!(result.url, "https://github.com/orgs/example/projects/1");
        assert_eq!(result.updated_at, Some("2024-06-01T00:00:00Z".to_string()));
    }

    #[test]
    fn test_normalize_project_summary_minimal() {
        let json = serde_json::json!({});

        let result = normalize_project_summary(&json);

        assert_eq!(result.id, "");

        assert_eq!(result.number, 0);
        assert_eq!(result.title, "");

        assert_eq!(result.description, "");
        assert!(!result.closed);

        assert!(result.updated_at.is_none());
    }
}
