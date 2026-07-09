use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct BrowsingApi;

impl BrowsingApi {
    pub async fn list_contents(
        client: &ProviderClient,
        project: &str,
        path: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = match path {
            Some(p) => format!(
                "/projects/{encoded}/repository/tree?path={}",
                urlencoding::encode(p)
            ),
            None => format!("/projects/{encoded}/repository/tree"),
        };

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list contents: {e}")))?;

        Ok(data)
    }

    pub async fn file_contents(
        client: &ProviderClient,
        project: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint = format!(
            "/projects/{encoded}/repository/files/{}?raw=false",
            urlencoding::encode(path)
        );

        if let Some(r) = r#ref {
            endpoint.push_str(&format!("&ref={}", urlencoding::encode(r)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get file contents: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_browsing_encode_path() {
        let encoded = urlencoding::encode("org/repo").to_string();

        assert_eq!(encoded, "org%2Frepo");
    }

    #[test]
    fn test_gitlab_browsing_list_contents_no_path() {
        let project = "org/repo";
        let encoded = urlencoding::encode(project).to_string();

        let endpoint = format!("/projects/{encoded}/repository/tree");

        assert_eq!(endpoint, "/projects/org%2Frepo/repository/tree");
    }

    #[test]
    fn test_gitlab_browsing_list_contents_with_path() {
        let project = "org/repo";
        let encoded = urlencoding::encode(project).to_string();

        let path = "src";
        let endpoint = format!(
            "/projects/{encoded}/repository/tree?path={}",
            urlencoding::encode(path)
        );

        assert_eq!(endpoint, "/projects/org%2Frepo/repository/tree?path=src");
    }
}
