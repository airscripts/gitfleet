use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    limit: u32,
    page: Option<u32>,
) -> Result<(), GitfleetError> {
    let ops = provider.release_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Releases,
        ))
    })?;

    let data = ops.list_releases(repo, limit, page).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        let items = data.as_array().cloned().unwrap_or_default();

        let rows: Vec<serde_json::Value> = items
            .iter()
            .map(|r| {
                serde_json::json!({
                    "TAG": r.get("tag_name"),
                    "TITLE": r.get("name"),
                    "DRAFT": r.get("draft"),
                    "PRERELEASE": r.get("prerelease"),
                    "PUBLISHED": r.get("published_at"),
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No releases found."),
            Some("Releases"),
            Some(&["TAG", "TITLE", "DRAFT", "PRERELEASE", "PUBLISHED"]),
        );
    }

    Ok(())
}

pub async fn view(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    tag: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.release_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Releases,
        ))
    })?;

    let data = ops.fetch_release_by_tag(repo, tag).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        renderer.render_summary(
            "Release",
            &[
                (
                    "Tag",
                    data.get("tag_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "Title",
                    data.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "Draft",
                    if data.get("draft").and_then(|v| v.as_bool()).unwrap_or(false) {
                        "yes".to_string()
                    } else {
                        "no".to_string()
                    },
                ),
                (
                    "Prerelease",
                    if data
                        .get("prerelease")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        "yes".to_string()
                    } else {
                        "no".to_string()
                    },
                ),
                (
                    "URL",
                    data.get("html_url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            ],
        );
    }

    Ok(())
}

pub async fn create(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    body: serde_json::Value,
) -> Result<(), GitfleetError> {
    let ops = provider.release_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Releases,
        ))
    })?;

    let data = ops.create_release(repo, body).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        let tag = data.get("tag_name").and_then(|v| v.as_str()).unwrap_or("");

        let url = data.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
        renderer.render_success_box("Release created", &format!("{tag}\n{url}"));
    }

    Ok(())
}

pub async fn delete(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    release: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.release_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Releases,
        ))
    })?;

    ops.delete_release(repo, release).await?;

    renderer.render_success_box("Release deleted", release);

    Ok(())
}
