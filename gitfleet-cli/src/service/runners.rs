use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.runner_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Runners,
        ))
    })?;

    let runners = ops.list_runners(repo).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&runners)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize runners: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = runners
            .iter()
            .map(|r| {
                serde_json::json!({
                    "ID": r.id,
                    "NAME": r.name,
                    "OS": r.os,
                    "STATUS": r.status,
                    "BUSY": r.busy,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No runners found."),
            Some("Runners"),
            Some(&["ID", "NAME", "OS", "STATUS", "BUSY"]),
        );
    }

    Ok(())
}

pub async fn remove(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    runner_id: u64,
) -> Result<(), GitfleetError> {
    let ops = provider.runner_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Runners,
        ))
    })?;

    ops.remove_runner(repo, runner_id).await?;

    renderer.render_success_box("Runner removed", &runner_id.to_string());

    Ok(())
}
