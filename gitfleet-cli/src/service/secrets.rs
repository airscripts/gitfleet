use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.secret_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Secrets,
        ))
    })?;

    let result = ops.list_repo_secrets(owner, repo).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&result)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize secrets: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = result
            .secrets
            .iter()
            .map(|s| {
                serde_json::json!({
                    "NAME": s.name,
                    "CREATED": s.created_at,
                    "UPDATED": s.updated_at,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No secrets found."),
            Some("Secrets"),
            Some(&["NAME", "CREATED", "UPDATED"]),
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
    encrypted_value: &str,
    key_id: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.secret_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Secrets,
        ))
    })?;

    ops.set_repo_secret(owner, repo, name, encrypted_value, key_id)
        .await?;

    renderer.render_success_box("Secret set", name);

    Ok(())
}

pub async fn delete(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    repo: &str,
    name: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.secret_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Secrets,
        ))
    })?;

    ops.delete_repo_secret(owner, repo, name).await?;

    renderer.render_success_box("Secret deleted", name);

    Ok(())
}

pub async fn get_public_key(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.secret_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Secrets,
        ))
    })?;

    let key = ops.get_repo_public_key(owner, repo).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&key)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize public key: {e}")))?;

        renderer.write_result(&json);
    } else {
        renderer.render_summary(
            "Public Key",
            &[("Key ID", key.key_id), ("Key", key.key.clone())],
        );
    }

    Ok(())
}
