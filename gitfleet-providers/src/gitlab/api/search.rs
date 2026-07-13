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

        Ok(SearchResult {
            total_count: raw.len() as u64,
            incomplete_results: false,
            items: raw,
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

        Ok(SearchResult {
            total_count: raw.len() as u64,
            incomplete_results: false,
            items: raw,
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

        Ok(SearchResult {
            total_count: raw.len() as u64,
            incomplete_results: false,
            items: raw,
        })
    }
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
