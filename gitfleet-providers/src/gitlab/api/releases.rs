use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct ReleasesApi;

impl ReleasesApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
        limit: u32,
        page: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let page = page.unwrap_or(1);
        let endpoint = format!("/projects/{encoded}/releases?per_page={limit}&page={page}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list releases: {e}")))?;

        Ok(data)
    }

    pub async fn fetch_by_tag(
        client: &ProviderClient,
        project: &str,
        tag: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let enc_tag = urlencoding::encode(tag);
        let endpoint = format!("/projects/{encoded}/releases/{enc_tag}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to fetch release by tag: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        project: &str,
        mut body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/releases");

        normalize_release_body(&mut body)?;

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create release: {e}")))?;

        Ok(data)
    }

    pub async fn update(
        client: &ProviderClient,
        project: &str,
        release: &str,
        mut body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);
        let release = urlencoding::encode(release);

        let endpoint = format!("/projects/{encoded}/releases/{release}");

        normalize_release_body(&mut body)?;

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update release: {e}")))?;

        Ok(data)
    }

    pub async fn delete(
        client: &ProviderClient,
        project: &str,
        release: &str,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);
        let release = urlencoding::encode(release);

        let endpoint = format!("/projects/{encoded}/releases/{release}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_release_body(body: &mut serde_json::Value) -> Result<(), GitfleetError> {
    if body.get("draft").and_then(serde_json::Value::as_bool) == Some(true)
        || body.get("prerelease").and_then(serde_json::Value::as_bool) == Some(true)
    {
        return Err(GitfleetError::from(
            gitfleet_core::errors::UnsupportedCapabilityError::new(
                gitfleet_core::provider::ProviderId::GitLab,
                gitfleet_core::provider::ProviderCapability::Releases,
            ),
        ));
    }

    let object = body
        .as_object_mut()
        .ok_or_else(|| GitfleetError::new("Release request body must be a JSON object."))?;

    if let Some(description) = object.remove("body") {
        object.insert("description".to_string(), description);
    }

    object.remove("draft");
    object.remove("prerelease");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_path_release() {
        assert_eq!(encode_path("org/repo"), "org%2Frepo");

        assert_eq!(encode_path("simple-proj"), "simple-proj");
    }

    #[test]
    fn test_release_list_endpoint() {
        let project = "org/repo";
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/releases?per_page=20");

        assert_eq!(endpoint, "/projects/org%2Frepo/releases?per_page=20");
    }

    #[test]
    fn test_normalize_release_body_maps_description() {
        let mut body = serde_json::json!({
            "tag_name": "v1.0.0",
            "body": "Release notes",
            "draft": false,
            "prerelease": false
        });

        normalize_release_body(&mut body).unwrap();

        assert_eq!(body["description"], "Release notes");
        assert!(body.get("body").is_none());
        assert!(body.get("draft").is_none());
        assert!(body.get("prerelease").is_none());
    }

    #[test]
    fn test_normalize_release_body_rejects_github_only_states() {
        let mut body = serde_json::json!({"tag_name": "v1.0.0", "draft": true});

        assert!(normalize_release_body(&mut body).is_err());
    }
}
