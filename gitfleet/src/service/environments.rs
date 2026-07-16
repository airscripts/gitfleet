use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.environment_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Environments,
        ))
    })?;

    let data = ops.list_environments(owner, repo).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&data)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize environments: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = data
            .environments
            .iter()
            .map(|env| {
                serde_json::json!({
                    "NAME": env.name,
                    "URL": env.url.as_deref().unwrap_or(""),
                    "CREATED": env.created_at,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No environments found."),
            Some("Environments"),
            Some(&["NAME", "URL", "CREATED"]),
        );
    }

    Ok(())
}

pub async fn create(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    repo: &str,
    name: &str,
    wait_timer: Option<u32>,
) -> Result<(), GitfleetError> {
    let ops = provider.environment_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Environments,
        ))
    })?;

    let data = ops
        .create_environment(owner, repo, name, wait_timer)
        .await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        renderer.render_success_box("Environment created", name);
    }

    Ok(())
}

pub async fn delete(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    repo: &str,
    name: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.environment_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Environments,
        ))
    })?;

    ops.delete_environment(owner, repo, name).await?;

    if renderer.is_json() {
        renderer.write_result(&serde_json::json!({
            "deleted": true,
            "name": name,
        }));
    } else {
        renderer.write_value(&format!("Environment '{name}' deleted."));
    }

    Ok(())
}
