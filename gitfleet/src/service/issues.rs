use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    state: &str,
    limit: u32,
) -> Result<(), GitfleetError> {
    let ops = provider.issue_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Issues,
        ))
    })?;

    let data = ops.list_issues(repo, state, limit, &[], &[]).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        let items = data
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let rows: Vec<serde_json::Value> = items
            .iter()
            .map(|item| {
                let url = item
                    .get("html_url")
                    .or_else(|| item.get("web_url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                serde_json::json!({
                    "NUMBER": item.get("number"),
                    "TITLE": item.get("title"),
                    "STATE": item.get("state"),
                    "URL": url,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No issues found."),
            Some("Issues"),
            Some(&["NUMBER", "TITLE", "STATE", "URL"]),
        );
    }

    Ok(())
}

pub async fn view(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    number: u64,
) -> Result<(), GitfleetError> {
    let ops = provider.issue_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Issues,
        ))
    })?;

    let data = ops.get_issue(repo, number).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        renderer.render_summary(
            "Issue",
            &[
                (
                    "Number",
                    format!(
                        "#{}",
                        data.get("number").and_then(|v| v.as_u64()).unwrap_or(0)
                    ),
                ),
                (
                    "Title",
                    data.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "State",
                    data.get("state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "Author",
                    data.get("user")
                        .and_then(|u| u.get("login"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "URL",
                    data.get("html_url")
                        .or_else(|| data.get("web_url"))
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
    title: &str,
    body: Option<&str>,
    labels: &[String],
    assignees: &[String],
) -> Result<(), GitfleetError> {
    let ops = provider.issue_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Issues,
        ))
    })?;

    let data = ops
        .create_issue(repo, title, body, labels, assignees)
        .await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        let number = data.get("number").and_then(|v| v.as_u64()).unwrap_or(0);

        renderer.render_success_box("Issue created", &format!("#{}", number));
    }

    Ok(())
}
