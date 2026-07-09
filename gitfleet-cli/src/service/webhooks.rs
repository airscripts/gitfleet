use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.webhook_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Webhooks,
        ))
    })?;

    let webhooks = ops.list_webhooks(repo).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&webhooks)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize webhooks: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = webhooks
            .iter()
            .map(|w| {
                serde_json::json!({
                    "ID": w.id,
                    "NAME": w.name,
                    "URL": w.url,
                    "ACTIVE": w.active,
                    "EVENTS": w.events.join(", "),
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No webhooks found."),
            Some("Webhooks"),
            Some(&["ID", "NAME", "URL", "ACTIVE", "EVENTS"]),
        );
    }

    Ok(())
}

pub async fn create(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    input: serde_json::Value,
) -> Result<(), GitfleetError> {
    let ops = provider.webhook_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Webhooks,
        ))
    })?;

    let webhook = ops.create_webhook(repo, input).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&webhook)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize webhook: {e}")))?;

        renderer.write_result(&json);
    } else {
        renderer.render_success_box(
            "Webhook created",
            &format!("{} ({})", webhook.id, webhook.url),
        );
    }

    Ok(())
}

pub async fn delete(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    hook_id: u64,
) -> Result<(), GitfleetError> {
    let ops = provider.webhook_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Webhooks,
        ))
    })?;

    ops.remove_webhook(repo, hook_id).await?;

    renderer.render_success_box("Webhook deleted", &hook_id.to_string());

    Ok(())
}

pub async fn test(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    hook_id: u64,
) -> Result<(), GitfleetError> {
    let ops = provider.webhook_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Webhooks,
        ))
    })?;

    ops.test_webhook(repo, hook_id).await?;

    renderer.render_success_box("Test ping sent", &hook_id.to_string());

    Ok(())
}
