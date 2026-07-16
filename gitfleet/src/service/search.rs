use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn search_issues(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    query: &str,
    sort: Option<&str>,
    order: Option<&str>,
    limit: u32,
) -> Result<(), GitfleetError> {
    let ops = provider.search_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Search,
        ))
    })?;

    let result = ops.search_issues(query, sort, order, limit).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&result)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize search results: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = result
            .items
            .iter()
            .map(|item| {
                serde_json::json!({
                    "NUMBER": item.get("number"),
                    "TITLE": item.get("title"),
                    "STATE": item.get("state"),
                    "URL": item.get("html_url"),
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No issues found."),
            Some("Issues"),
            Some(&["NUMBER", "TITLE", "STATE", "URL"]),
        );

        renderer.write_value("");

        renderer.render_summary(
            "Search Results",
            &[
                ("Total", result.total_count.to_string()),
                ("Showing", result.items.len().to_string()),
            ],
        );
    }

    Ok(())
}

pub async fn search_repos(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    query: &str,
    sort: Option<&str>,
    order: Option<&str>,
    limit: u32,
) -> Result<(), GitfleetError> {
    let ops = provider.search_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Search,
        ))
    })?;

    let result = ops.search_repos(query, sort, order, limit).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&result)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize search results: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = result
            .items
            .iter()
            .map(|item| {
                serde_json::json!({
                    "NAME": item.get("full_name"),
                    "VISIBILITY": if item.get("private").and_then(|v| v.as_bool()).unwrap_or(false) { "private" } else { "public" },
                    "STARS": item.get("stargazers_count"),
                    "LANGUAGE": item.get("language"),
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No repositories found."),
            Some("Repositories"),
            Some(&["NAME", "VISIBILITY", "STARS", "LANGUAGE"]),
        );

        renderer.write_value("");

        renderer.render_summary(
            "Search Results",
            &[
                ("Total", result.total_count.to_string()),
                ("Showing", result.items.len().to_string()),
            ],
        );
    }

    Ok(())
}

pub async fn search_code(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    query: &str,
    limit: u32,
) -> Result<(), GitfleetError> {
    let ops = provider.search_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Search,
        ))
    })?;

    let result = ops.search_code(query, limit).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&result)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize search results: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = result
            .items
            .iter()
            .map(|item| {
                let repo = item
                    .get("repository")
                    .and_then(|r| r.get("full_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let path = item.get("path").and_then(|v| v.as_str()).unwrap_or("");
                serde_json::json!({
                    "FILE": path,
                    "REPO": repo,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No code results found."),
            Some("Code"),
            Some(&["FILE", "REPO"]),
        );

        renderer.write_value("");

        renderer.render_summary(
            "Search Results",
            &[
                ("Total", result.total_count.to_string()),
                ("Showing", result.items.len().to_string()),
            ],
        );
    }

    Ok(())
}
