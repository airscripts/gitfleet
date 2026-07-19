use gitfleet_core::errors::{GitfleetError, NotFoundError};
use gitfleet_core::types::PackageSummary;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct RegistryApi;

impl RegistryApi {
    pub async fn list(
        client: &ProviderClient,
        owner: &str,
        package_type: Option<&str>,
        limit: u32,
        page: Option<u32>,
    ) -> Result<Vec<PackageSummary>, GitfleetError> {
        let encoded = encode_path(owner);

        let page = page.unwrap_or(1);
        let mut endpoint = format!("/projects/{encoded}/packages?per_page={limit}&page={page}");

        if let Some(pt) = package_type {
            endpoint.push_str(&format!("&package_type={}", urlencoding::encode(pt)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list packages: {e}")))?;

        Ok(data.iter().map(normalize_package).collect())
    }

    pub async fn get(
        client: &ProviderClient,
        owner: &str,
        package_type: &str,
        package_name: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(owner);

        let endpoint = format!(
            "/projects/{encoded}/packages?package_type={}&package_name={}",
            urlencoding::encode(package_type),
            urlencoding::encode(package_name),
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get package: {e}")))?;

        data.into_iter()
            .find(|package| {
                package.get("name").and_then(serde_json::Value::as_str) == Some(package_name)
                    && package
                        .get("package_type")
                        .and_then(serde_json::Value::as_str)
                        == Some(package_type)
            })
            .ok_or_else(|| GitfleetError::from(NotFoundError::new("Package not found.")))
    }
}

fn normalize_package(raw: &serde_json::Value) -> PackageSummary {
    let id = raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    PackageSummary {
        id,
        name: raw
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        package_type: raw
            .get("package_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        visibility: raw
            .get("visibility")
            .and_then(|v| v.as_str())
            .unwrap_or("private")
            .to_string(),
        url: raw
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        html_url: raw
            .get("_links")
            .and_then(|l| l.get("web_path"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        created_at: raw
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        updated_at: raw
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        owner: String::new(),
        repository: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_package_full() {
        let json = serde_json::json!({
            "id": 1,
            "name": "my-package",
            "package_type": "npm",
            "visibility": "public",
            "url": "https://gitlab.com/api/v4/projects/1/packages/1",
            "_links": { "web_path": "/project1/-/packages/1" },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        });

        let result = normalize_package(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.name, "my-package");
        assert_eq!(result.package_type, "npm");

        assert_eq!(result.html_url, "/project1/-/packages/1");
    }

    #[test]
    fn test_normalize_package_minimal() {
        let json = serde_json::json!({});

        let result = normalize_package(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.name, "");
        assert_eq!(result.visibility, "private");
    }

    #[test]
    fn test_encode_path() {
        assert_eq!(encode_path("org/repo"), "org%2Frepo");
    }
}
