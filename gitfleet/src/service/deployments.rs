use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    environment: Option<&str>,
    limit: u32,
    page: Option<u32>,
) -> Result<(), GitfleetError> {
    let ops = provider.deploy_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Deployments,
        ))
    })?;

    let deployments = ops.list_deployments(repo, environment, limit, page).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&deployments)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize deployments: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = deployments
            .iter()
            .map(|d| {
                serde_json::json!({
                    "ID": d.id,
                    "ENVIRONMENT": d.environment,
                    "REF": d.r#ref,
                    "TASK": d.task,
                    "CREATED": d.created_at,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No deployments found."),
            Some("Deployments"),
            Some(&["ID", "ENVIRONMENT", "REF", "TASK", "CREATED"]),
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
    let ops = provider.deploy_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Deployments,
        ))
    })?;

    let deployment = ops.create_deployment(repo, input).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&deployment)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize deployment: {e}")))?;

        renderer.write_result(&json);
    } else {
        renderer.render_success_box(
            "Deployment created",
            &format!("{}: {}", deployment.id, deployment.environment),
        );
    }

    Ok(())
}
