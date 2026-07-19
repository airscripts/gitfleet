use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub struct ListOptions<'a> {
    pub state: &'a str,
    pub limit: u32,
    pub page: Option<u32>,
    pub base: Option<&'a str>,
    pub head: Option<&'a str>,
}

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    options: ListOptions<'_>,
) -> Result<(), GitfleetError> {
    let ops = provider.change_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Changes,
        ))
    })?;

    let prs = ops
        .list_changes(
            repo,
            options.state,
            options.limit,
            options.page,
            options.base,
            options.head,
        )
        .await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&prs)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize changes: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = prs
            .iter()
            .map(|pr| {
                serde_json::json!({
                    "NUMBER": pr.number,
                    "TITLE": pr.title,
                    "STATE": pr.state,
                    "AUTHOR": pr.user.as_ref().map(|u| u.login.clone()).unwrap_or_default(),
                    "URL": pr.html_url.as_deref().unwrap_or(""),
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No changes found."),
            Some("Change Requests"),
            Some(&["NUMBER", "TITLE", "STATE", "AUTHOR", "URL"]),
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
    let ops = provider.change_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Changes,
        ))
    })?;

    let pr = ops.get_change(repo, number).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&pr)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize change: {e}")))?;

        renderer.write_result(&json);
    } else {
        renderer.render_summary(
            "Change Request",
            &[
                ("Number", format!("#{}", pr.number)),
                ("Title", pr.title.clone()),
                ("State", pr.state.clone()),
                (
                    "Author",
                    pr.user
                        .as_ref()
                        .map(|u| u.login.clone())
                        .unwrap_or_default(),
                ),
                ("Base", pr.base.r#ref.clone()),
                ("Head", pr.head.r#ref.clone()),
                (
                    "Draft",
                    if pr.draft.unwrap_or(false) {
                        "yes".to_string()
                    } else {
                        "no".to_string()
                    },
                ),
                (
                    "Merged",
                    if pr.merged {
                        "yes".to_string()
                    } else {
                        "no".to_string()
                    },
                ),
                ("URL", pr.html_url.as_deref().unwrap_or("").to_string()),
            ],
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    title: &str,
    head: &str,
    base: &str,
    body: Option<&str>,
    draft: bool,
) -> Result<(), GitfleetError> {
    let ops = provider.change_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Changes,
        ))
    })?;

    let pr = ops
        .create_change(repo, title, head, base, body, draft)
        .await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&pr)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize change: {e}")))?;

        renderer.write_result(&json);
    } else {
        renderer.render_success_box(
            "Change request created",
            &format!("#{} {}", pr.number, pr.title),
        );
    }

    Ok(())
}

pub async fn merge(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    number: u64,
    method: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.change_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Changes,
        ))
    })?;

    let result = ops.merge_change(repo, number, method).await?;

    if renderer.is_json() {
        renderer.write_result(&result);
    } else {
        renderer.render_success_box("Merged", &format!("#{}", number));
    }

    Ok(())
}
