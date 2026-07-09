use gitfleet_core::errors::{GitfleetError, NotFoundError};
use gitfleet_core::types::Discussion;

use crate::github::client::ProviderClient;

pub struct DiscussionsApi;

fn normalize_discussion(raw: &serde_json::Value) -> Discussion {
    Discussion {
        id: raw
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        url: raw
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        body: raw
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        title: raw
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        number: raw.get("number").and_then(|v| v.as_u64()).unwrap_or(0),
        author: raw
            .get("author")
            .and_then(|a| a.get("login"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        closed: raw.get("closed").and_then(|v| v.as_bool()).unwrap_or(false),
        category: raw
            .get("category")
            .and_then(|c| c.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        created_at: raw
            .get("createdAt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        updated_at: raw
            .get("updatedAt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        comments_count: raw
            .get("comments")
            .and_then(|c| c.get("totalCount"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
    }
}

impl DiscussionsApi {
    pub async fn list(
        client: &ProviderClient,
        owner: &str,
        name: &str,
        category_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Discussion>, GitfleetError> {
        let query = r#"
            query ListDiscussions($owner: String!, $name: String!, $first: Int!, $categoryId: ID) {
                repository(owner: $owner, name: $name) {
                    discussions(first: $first, categoryId: $categoryId) {
                        nodes { id title number url body author { login } closed createdAt updatedAt category { name } comments { totalCount } }
                    }
                }
            }
        "#;
        let payload = serde_json::json!({
            "query": query,
            "variables": {
                "owner": owner,
                "name": name,
                "first": limit,
                "categoryId": category_id,
            }
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/graphql", Some(payload), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list discussions: {e}")))?;

        let nodes = data
            .get("data")
            .and_then(|d| d.get("repository"))
            .and_then(|r| r.get("discussions"))
            .and_then(|d| d.get("nodes"))
            .and_then(|n| n.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(nodes.iter().map(normalize_discussion).collect())
    }

    pub async fn get(
        client: &ProviderClient,
        owner: &str,
        name: &str,
        discussion_number: u64,
    ) -> Result<Discussion, GitfleetError> {
        let query = r#"
            query GetDiscussion($owner: String!, $name: String!, $number: Int!) {
                repository(owner: $owner, name: $name) {
                    discussion(number: $number) {
                        id title number url body author { login } closed createdAt updatedAt category { name } comments { totalCount }
                    }
                }
            }
        "#;
        let payload = serde_json::json!({
            "query": query,
            "variables": { "owner": owner, "name": name, "number": discussion_number }
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/graphql", Some(payload), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get discussion: {e}")))?;

        let raw = data
            .get("data")
            .and_then(|d| d.get("repository"))
            .and_then(|r| r.get("discussion"))
            .unwrap_or(&serde_json::Value::Null);

        Ok(normalize_discussion(raw))
    }

    pub async fn create(
        client: &ProviderClient,
        owner: &str,
        name: &str,
        title: &str,
        body: &str,
        category_id: Option<&str>,
    ) -> Result<Discussion, GitfleetError> {
        let id_query = r#"
            query GetRepoId($owner: String!, $name: String!) {
                repository(owner: $owner, name: $name) { id }
            }
        "#;
        let id_payload = serde_json::json!({
            "query": id_query,
            "variables": { "owner": owner, "name": name }
        });

        let id_response = client
            .request_token_required(
                reqwest::Method::POST,
                "/graphql",
                Some(id_payload),
                None,
                None,
            )
            .await?;

        let id_data: serde_json::Value = id_response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to fetch repository id: {e}")))?;

        let repo_node_id = id_data
            .get("data")
            .and_then(|d| d.get("repository"))
            .and_then(|r| r.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                GitfleetError::from(NotFoundError::new(
                    "Failed to resolve repository node id for discussion creation",
                ))
            })?;

        let mutation = r#"
            mutation CreateDiscussion($input: CreateDiscussionInput!) {
                createDiscussion(input: $input) {
                    discussion {
                        id title number url body author { login } closed createdAt updatedAt category { name } comments { totalCount }
                    }
                }
            }
        "#;
        let mut input = serde_json::json!({
            "repositoryId": repo_node_id,
            "title": title,
            "body": body,
        });

        if let Some(cid) = category_id {
            input["categoryId"] = serde_json::Value::String(cid.to_string());
        }

        let payload = serde_json::json!({
            "query": mutation,
            "variables": { "input": input }
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/graphql", Some(payload), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create discussion: {e}")))?;

        let raw = data
            .get("data")
            .and_then(|d| d.get("createDiscussion"))
            .and_then(|c| c.get("discussion"))
            .unwrap_or(&serde_json::Value::Null);

        Ok(normalize_discussion(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_discussion_full() {
        let json = serde_json::json!({
            "id": "DI_abc123",
            "url": "https://github.com/org/repo/discussions/1",
            "body": "Discussion body text",
            "title": "Feature request",
            "number": 1,
            "author": { "login": "octocat" },
            "closed": false,
            "category": { "name": "Ideas" },
            "createdAt": "2024-01-01T00:00:00Z",
            "updatedAt": "2024-01-02T00:00:00Z",
            "comments": { "totalCount": 5 }
        });

        let result = normalize_discussion(&json);

        assert_eq!(result.id, "DI_abc123");

        assert_eq!(result.url, "https://github.com/org/repo/discussions/1");
        assert_eq!(result.body, "Discussion body text");

        assert_eq!(result.title, "Feature request");
        assert_eq!(result.number, 1);

        assert_eq!(result.author, "octocat");
        assert!(!result.closed);

        assert_eq!(result.category, "Ideas");
        assert_eq!(result.created_at, "2024-01-01T00:00:00Z");

        assert_eq!(result.updated_at, "2024-01-02T00:00:00Z");
        assert_eq!(result.comments_count, 5);
    }

    #[test]
    fn test_normalize_discussion_minimal() {
        let json = serde_json::json!({});

        let result = normalize_discussion(&json);

        assert_eq!(result.id, "");

        assert_eq!(result.url, "");
        assert_eq!(result.body, "");

        assert_eq!(result.title, "");
        assert_eq!(result.number, 0);

        assert_eq!(result.author, "");
        assert!(!result.closed);

        assert_eq!(result.category, "");
        assert_eq!(result.comments_count, 0);
    }
}
