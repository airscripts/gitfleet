use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.variable_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Variables,
        ))
    })?;

    let result = ops.list_repo_variables(owner, repo).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&result)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize variables: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = result
            .variables
            .iter()
            .map(|v| {
                serde_json::json!({
                    "NAME": v.name,
                    "VALUE": v.value.as_deref().unwrap_or("(hidden)"),
                    "CREATED": v.created_at,
                    "UPDATED": v.updated_at,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No variables found."),
            Some("Variables"),
            Some(&["NAME", "VALUE", "CREATED", "UPDATED"]),
        );
    }

    Ok(())
}

pub async fn set(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    repo: &str,
    name: &str,
    value: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.variable_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Variables,
        ))
    })?;

    ops.set_repo_variable(owner, repo, name, value).await?;

    renderer.render_success_box("Variable set", name);

    Ok(())
}

pub async fn delete(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    repo: &str,
    name: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.variable_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Variables,
        ))
    })?;

    ops.delete_repo_variable(owner, repo, name).await?;

    renderer.render_success_box("Variable deleted", name);

    Ok(())
}
