use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::PackageSummary;

use crate::github::api::path::{encode_segment, repo_path};
use crate::github::client::ProviderClient;

pub struct PackagesApi;

impl PackagesApi {
    pub async fn list_for_org(
        client: &ProviderClient,
        org: &str,
        package_type: Option<&str>,
        limit: u32,
    ) -> Result<Vec<PackageSummary>, GitfleetError> {
        let mut endpoint = format!("/orgs/{}/packages?per_page={limit}", encode_segment(org));

        if let Some(pt) = package_type {
            endpoint.push_str(&format!("&package_type={}", encode_segment(pt)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list packages: {e}")))?;

        Ok(raw.iter().map(normalize_package).collect())
    }

    pub async fn list_for_repo(
        client: &ProviderClient,
        repo: &str,
        package_type: Option<&str>,
    ) -> Result<Vec<PackageSummary>, GitfleetError> {
        let mut endpoint = repo_path(repo, &["packages"]);
        endpoint.push_str("?per_page=100");

        if let Some(pt) = package_type {
            endpoint.push_str(&format!("&package_type={}", encode_segment(pt)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list packages: {e}")))?;

        Ok(raw.iter().map(normalize_package).collect())
    }

    pub async fn get(
        client: &ProviderClient,
        repo: &str,
        package_type: &str,
        package_name: &str,
    ) -> Result<PackageSummary, GitfleetError> {
        let endpoint = repo_path(repo, &["packages", package_type, package_name]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get package: {e}")))?;

        Ok(normalize_package(&raw))
    }

    pub async fn list_for_user(
        client: &ProviderClient,
        owner: &str,
        package_type: Option<&str>,
        limit: u32,
    ) -> Result<Vec<PackageSummary>, GitfleetError> {
        let mut endpoint = format!("/users/{}/packages?per_page={limit}", encode_segment(owner));

        if let Some(pt) = package_type {
            endpoint.push_str(&format!("&package_type={}", encode_segment(pt)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list packages: {e}")))?;

        Ok(raw.iter().map(normalize_package).collect())
    }

    pub async fn get_json(
        client: &ProviderClient,
        owner: &str,
        package_type: &str,
        package_name: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!(
            "/orgs/{}/packages/{}/{}",
            encode_segment(owner),
            encode_segment(package_type),
            encode_segment(package_name)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get package: {e}")))?;

        Ok(data)
    }

    pub async fn get_json_for_user(
        client: &ProviderClient,
        owner: &str,
        package_type: &str,
        package_name: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!(
            "/users/{}/packages/{}/{}",
            encode_segment(owner),
            encode_segment(package_type),
            encode_segment(package_name)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get package: {e}")))
    }
}

fn normalize_package(raw: &serde_json::Value) -> PackageSummary {
    PackageSummary {
        id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
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
            .unwrap_or("")
            .to_string(),
        url: raw
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        html_url: raw
            .get("html_url")
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
        owner: raw
            .get("owner")
            .and_then(|o| o.get("login"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        repository: raw
            .get("repository")
            .and_then(|r| r.get("full_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_package_full() {
        let json = serde_json::json!({
            "id": 1,
            "name": "my-pkg",
            "package_type": "npm",
            "visibility": "public",
            "url": "https://api.github.com/orgs/org/packages/npm/my-pkg",
            "html_url": "https://github.com/org/repo/pkgs/npm/my-pkg",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-06-01T00:00:00Z",
            "owner": { "login": "org" },
            "repository": { "full_name": "org/repo" }
        });

        let result = normalize_package(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.name, "my-pkg");
        assert_eq!(result.package_type, "npm");

        assert_eq!(result.visibility, "public");
        assert_eq!(result.owner, "org");

        assert_eq!(result.repository, "org/repo");
    }

    #[test]
    fn test_normalize_package_minimal() {
        let json = serde_json::json!({});

        let result = normalize_package(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.name, "");
        assert_eq!(result.owner, "");

        assert_eq!(result.repository, "");
    }
}
