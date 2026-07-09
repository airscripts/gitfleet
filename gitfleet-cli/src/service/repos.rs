use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    org: Option<&str>,
    username: Option<&str>,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    let repos = if let Some(org) = org {
        ops.list_org_repos(org).await?
    } else if let Some(username) = username {
        ops.list_user_named_repos(username).await?
    } else {
        ops.list_user_repos().await?
    };

    if renderer.is_json() {
        let json = serde_json::to_value(&repos)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize repos: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = repos
            .iter()
            .map(|r| {
                serde_json::json!({
                    "NAME": r.full_name,
                    "VISIBILITY": if r.private { "private" } else { "public" },
                    "DEFAULT BRANCH": r.default_branch,
                    "ARCHIVED": r.archived,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No repositories found."),
            Some("Repositories"),
            Some(&["NAME", "VISIBILITY", "DEFAULT BRANCH", "ARCHIVED"]),
        );
    }

    Ok(())
}

pub async fn view(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    let data = ops.get_repo(repo).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        renderer.render_summary(
            "Repository",
            &[
                (
                    "Name",
                    data.get("full_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "Description",
                    data.get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A")
                        .to_string(),
                ),
                (
                    "Visibility",
                    if data
                        .get("private")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        "private".to_string()
                    } else {
                        "public".to_string()
                    },
                ),
                (
                    "Default Branch",
                    data.get("default_branch")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
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
    name: &str,
    owner: Option<&str>,
    owner_type: Option<&str>,
    visibility: &str,
    description: Option<&str>,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    let data = ops
        .create_repo(name, visibility, owner, owner_type, description)
        .await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        let full_name = data
            .get("full_name")
            .and_then(|v| v.as_str())
            .unwrap_or(name);

        let html_url = data.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
        renderer.render_success_box("Repository created", &format!("{full_name}\n{html_url}"));
    }

    Ok(())
}

pub async fn delete(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    ops.delete_repo(repo).await?;

    renderer.render_success_box("Repository deleted", repo);

    Ok(())
}

pub async fn star(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    ops.star_repo(repo).await?;

    renderer.render_success_box("Starred", repo);

    Ok(())
}

pub async fn unstar(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    ops.unstar_repo(repo).await?;

    renderer.render_success_box("Unstarred", repo);

    Ok(())
}
