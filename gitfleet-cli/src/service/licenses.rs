use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(provider: &dyn GitProvider, renderer: &Renderer) -> Result<(), GitfleetError> {
    let ops = provider.license_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Licenses,
        ))
    })?;

    let licenses = ops.list_licenses().await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&licenses)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize licenses: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = licenses
            .iter()
            .map(|l| {
                serde_json::json!({
                    "KEY": l.key,
                    "NAME": l.name,
                    "SPDX": l.spdx_id,
                })
            })
            .collect();

        renderer.render_table_titled(&rows, Some("No licenses found."), Some("Licenses"), None);
    }

    Ok(())
}

pub async fn view(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    key: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.license_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Licenses,
        ))
    })?;

    let license = ops.get_license(key).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&license)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize license: {e}")))?;

        renderer.write_result(&json);
    } else {
        renderer.render_summary(
            "License",
            &[
                ("Key", license.key.clone()),
                ("Name", license.name.clone()),
                ("SPDX", license.spdx_id.clone()),
                ("URL", license.url.clone()),
            ],
        );
    }

    Ok(())
}
