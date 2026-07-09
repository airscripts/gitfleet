use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.label_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Labels,
        ))
    })?;

    let labels = ops.list_labels(repo).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&labels)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize labels: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = labels
            .iter()
            .map(|l| {
                serde_json::json!({
                    "NAME": l.name,
                    "COLOR": l.color,
                    "DESCRIPTION": l.description,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No labels found."),
            Some("Labels"),
            Some(&["NAME", "COLOR", "DESCRIPTION"]),
        );
    }

    Ok(())
}
