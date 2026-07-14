use std::collections::HashMap;

use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::SearchResult;

use crate::gitlab::client::ProviderClient;

pub struct SearchApi;

impl SearchApi {
    pub async fn search_issues(
        client: &ProviderClient,
        query: &str,
        sort: Option<&str>,
        order: Option<&str>,
        limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError> {
        let mut endpoint = format!(
            "/search?scope=issues&search={}&per_page={limit}",
            urlencoding::encode(query)
        );

        if let Some(s) = sort {
            endpoint.push_str(&format!("&order_by={}", urlencoding::encode(s)));
        }

        if let Some(o) = order {
            endpoint.push_str(&format!("&sort={}", urlencoding::encode(o)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to search issues: {e}")))?;

        let items = raw.iter().map(normalize_issue).collect();

        Ok(SearchResult {
            total_count: raw.len() as u64,
            incomplete_results: false,
            items,
        })
    }

    pub async fn search_projects(
        client: &ProviderClient,
        query: &str,
        sort: Option<&str>,
        order: Option<&str>,
        limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError> {
        let mut endpoint = format!(
            "/search?scope=projects&search={}&per_page={limit}",
            urlencoding::encode(query)
        );

        if let Some(s) = sort {
            endpoint.push_str(&format!("&order_by={}", urlencoding::encode(s)));
        }

        if let Some(o) = order {
            endpoint.push_str(&format!("&sort={}", urlencoding::encode(o)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to search projects: {e}")))?;

        let items = raw.iter().map(normalize_project).collect();

        Ok(SearchResult {
            total_count: raw.len() as u64,
            incomplete_results: false,
            items,
        })
    }

    pub async fn search_code(
        client: &ProviderClient,
        query: &str,
        limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError> {
        let endpoint = format!(
            "/search?scope=blobs&search={}&per_page={limit}",
            urlencoding::encode(query)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to search code: {e}")))?;

        let mut projects: HashMap<u64, String> = HashMap::new();
        let mut items = Vec::with_capacity(raw.len());

        for item in &raw {
            let project_id = item
                .get("project_id")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            let project = if let Some(project) = projects.get(&project_id) {
                project.clone()
            } else {
                let project = fetch_project_path(client, project_id).await?;
                projects.insert(project_id, project.clone());
                project
            };

            items.push(normalize_code(item, &project));
        }

        Ok(SearchResult {
            total_count: raw.len() as u64,
            incomplete_results: false,
            items,
        })
    }
}

fn normalize_issue(raw: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "number": raw.get("iid"),
        "title": raw.get("title"),
        "state": raw.get("state"),
        "html_url": raw.get("web_url"),
    })
}

fn normalize_project(raw: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "full_name": raw.get("path_with_namespace"),
        "private": raw
            .get("visibility")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|visibility| visibility == "private"),
        "stargazers_count": raw.get("star_count"),
        "language": serde_json::Value::Null,
        "html_url": raw.get("web_url"),
    })
}

async fn fetch_project_path(
    client: &ProviderClient,
    project_id: u64,
) -> Result<String, GitfleetError> {
    let endpoint = format!("/projects/{project_id}");
    let response = client
        .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
        .await?;
    let project: serde_json::Value = crate::parse_json(response)
        .await
        .map_err(|e| GitfleetError::new(format!("Failed to resolve code search project: {e}")))?;

    Ok(project
        .get("path_with_namespace")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string())
}

fn normalize_code(raw: &serde_json::Value, project: &str) -> serde_json::Value {
    serde_json::json!({
        "path": raw.get("path"),
        "repository": {
            "full_name": project,
        },
        "ref": raw.get("ref"),
    })
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::SearchResult;

    fn normalize_gitlab_search_result(
        raw: &[serde_json::Value],
    ) -> SearchResult<serde_json::Value> {
        SearchResult {
            total_count: raw.len() as u64,
            incomplete_results: false,
            items: raw.to_vec(),
        }
    }

    #[test]
    fn test_normalize_gitlab_search_result_with_items() {
        let items = vec![
            serde_json::json!({ "id": 1, "title": "Issue 1" }),
            serde_json::json!({ "id": 2, "title": "Issue 2" }),
        ];
        let result = normalize_gitlab_search_result(&items);

        assert_eq!(result.total_count, 2);

        assert!(!result.incomplete_results);
        assert_eq!(result.items.len(), 2);
    }

    #[test]
    fn test_normalize_gitlab_search_result_empty() {
        let items: Vec<serde_json::Value> = vec![];
        let result = normalize_gitlab_search_result(&items);

        assert_eq!(result.total_count, 0);

        assert!(result.items.is_empty());
    }
}
