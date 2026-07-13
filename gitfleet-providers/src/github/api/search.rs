use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::SearchResult;

use crate::github::client::ProviderClient;

pub struct SearchApi;

impl SearchApi {
    pub async fn issues(
        client: &ProviderClient,
        query: &str,
        sort: Option<&str>,
        order: Option<&str>,
        limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError> {
        let mut endpoint = format!(
            "/search/issues?q={}&per_page={limit}",
            urlencoding::encode(query)
        );

        if let Some(s) = sort {
            endpoint.push_str(&format!("&sort={}", urlencoding::encode(s)));
        }

        if let Some(o) = order {
            endpoint.push_str(&format!("&order={}", urlencoding::encode(o)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to search issues: {e}")))?;

        Ok(SearchResult {
            total_count: raw.get("total_count").and_then(|v| v.as_u64()).unwrap_or(0),
            incomplete_results: raw
                .get("incomplete_results")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            items: raw
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
        })
    }

    pub async fn repos(
        client: &ProviderClient,
        query: &str,
        sort: Option<&str>,
        order: Option<&str>,
        limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError> {
        let mut endpoint = format!(
            "/search/repositories?q={}&per_page={limit}",
            urlencoding::encode(query)
        );

        if let Some(s) = sort {
            endpoint.push_str(&format!("&sort={}", urlencoding::encode(s)));
        }

        if let Some(o) = order {
            endpoint.push_str(&format!("&order={}", urlencoding::encode(o)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to search repos: {e}")))?;

        Ok(SearchResult {
            total_count: raw.get("total_count").and_then(|v| v.as_u64()).unwrap_or(0),
            incomplete_results: raw
                .get("incomplete_results")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            items: raw
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
        })
    }

    pub async fn code(
        client: &ProviderClient,
        query: &str,
        limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError> {
        let endpoint = format!(
            "/search/code?q={}&per_page={limit}",
            urlencoding::encode(query)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to search code: {e}")))?;

        Ok(SearchResult {
            total_count: raw.get("total_count").and_then(|v| v.as_u64()).unwrap_or(0),
            incomplete_results: raw
                .get("incomplete_results")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            items: raw
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::SearchResult;

    fn normalize_search_result(raw: &serde_json::Value) -> SearchResult<serde_json::Value> {
        SearchResult {
            total_count: raw.get("total_count").and_then(|v| v.as_u64()).unwrap_or(0),
            incomplete_results: raw
                .get("incomplete_results")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            items: raw
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
        }
    }

    #[test]
    fn test_normalize_issue_search_result() {
        let json = serde_json::json!({
            "total_count": 1,
            "incomplete_results": false,
            "items": [
                {
                    "id": 100,
                    "title": "Bug in login",
                    "state": "open",
                    "number": 7
                }
            ]
        });

        let result = normalize_search_result(&json);

        assert_eq!(result.total_count, 1);

        assert!(!result.incomplete_results);
        assert_eq!(result.items.len(), 1);

        assert_eq!(result.items[0]["id"], 100);
    }

    #[test]
    fn test_normalize_repo_search_result() {
        let json = serde_json::json!({
            "total_count": 2,
            "incomplete_results": true,
            "items": [
                { "id": 1, "name": "repo1" },
                { "id": 2, "name": "repo2" }
            ]
        });

        let result = normalize_search_result(&json);

        assert_eq!(result.total_count, 2);

        assert!(result.incomplete_results);
        assert_eq!(result.items.len(), 2);
    }

    #[test]
    fn test_normalize_search_result_empty() {
        let json = serde_json::json!({});

        let result = normalize_search_result(&json);

        assert_eq!(result.total_count, 0);

        assert!(!result.incomplete_results);
        assert!(result.items.is_empty());
    }
}
