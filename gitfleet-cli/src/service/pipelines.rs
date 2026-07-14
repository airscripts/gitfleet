use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list_workflows(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    limit: u32,
    page: Option<u32>,
) -> Result<(), GitfleetError> {
    let ops = provider.pipeline_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Pipelines,
        ))
    })?;

    let data = ops.list_workflows(repo, limit, page).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        let workflows = data
            .get("workflows")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let rows: Vec<serde_json::Value> = workflows
            .iter()
            .map(|w| {
                serde_json::json!({
                    "ID": w.get("id"),
                    "NAME": w.get("name"),
                    "STATE": w.get("state"),
                    "PATH": w.get("path"),
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No pipeline definitions found."),
            Some("Pipeline Definitions"),
            Some(&["ID", "NAME", "STATE", "PATH"]),
        );
    }

    Ok(())
}

pub async fn view_workflow(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    workflow_id: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.pipeline_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Pipelines,
        ))
    })?;

    let data = ops.get_workflow(repo, workflow_id).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        renderer.render_summary(
            "Pipeline Definition",
            &[
                (
                    "ID",
                    data.get("id")
                        .and_then(|v| v.as_u64())
                        .map_or_else(String::new, |n| n.to_string()),
                ),
                (
                    "Name",
                    data.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "State",
                    data.get("state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "Path",
                    data.get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            ],
        );
    }

    Ok(())
}

pub async fn list_runs(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    filters: &str,
    limit: u32,
) -> Result<(), GitfleetError> {
    let ops = provider.pipeline_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Pipelines,
        ))
    })?;

    let data = ops.list_runs(repo, filters, limit).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        let runs = data
            .get("workflow_runs")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let rows: Vec<serde_json::Value> = runs
            .iter()
            .map(|r| {
                serde_json::json!({
                    "ID": r.get("id"),
                    "NAME": r.get("name"),
                    "STATUS": r.get("status"),
                    "CONCLUSION": r.get("conclusion"),
                    "BRANCH": r.get("head_branch"),
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No runs found."),
            Some("Runs"),
            Some(&["ID", "NAME", "STATUS", "CONCLUSION", "BRANCH"]),
        );
    }

    Ok(())
}

pub async fn view_run(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    run_id: u64,
) -> Result<(), GitfleetError> {
    let ops = provider.pipeline_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Pipelines,
        ))
    })?;

    let data = ops.get_run(repo, run_id).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        renderer.render_summary(
            "Pipeline Run",
            &[
                (
                    "ID",
                    data.get("id")
                        .and_then(|v| v.as_u64())
                        .map_or_else(String::new, |n| n.to_string()),
                ),
                (
                    "Name",
                    data.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "Status",
                    data.get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "Conclusion",
                    data.get("conclusion")
                        .and_then(|v| v.as_str())
                        .unwrap_or("in progress")
                        .to_string(),
                ),
                (
                    "Branch",
                    data.get("head_branch")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            ],
        );
    }

    Ok(())
}

pub async fn trigger_run(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    definition_id: Option<&str>,
    ref_name: &str,
    inputs: Option<serde_json::Value>,
) -> Result<(), GitfleetError> {
    let ops = provider.pipeline_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Pipelines,
        ))
    })?;

    ops.dispatch_pipeline(repo, definition_id, ref_name, inputs)
        .await?;

    let target = definition_id
        .map(|id| format!("{id} on {ref_name}"))
        .unwrap_or_else(|| ref_name.to_string());

    renderer.render_success_box("Pipeline dispatched", &target);

    Ok(())
}

pub async fn cancel_run(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    run_id: u64,
) -> Result<(), GitfleetError> {
    let ops = provider.pipeline_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Pipelines,
        ))
    })?;

    ops.cancel_run(repo, run_id).await?;

    renderer.render_success_box("Run cancelled", &run_id.to_string());

    Ok(())
}
