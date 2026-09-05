use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};
use gitfleet_core::types::{Label, LabelTemplate};

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

pub fn list_templates(renderer: &Renderer) -> Result<(), GitfleetError> {
    let templates = gitfleet_core::labels::builtins()?;

    if renderer.is_json() {
        let json = serde_json::to_value(&templates)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize templates: {e}")))?;
        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = templates
            .iter()
            .map(|template| {
                serde_json::json!({
                    "NAME": template.name,
                    "LABELS": template.labels.len(),
                    "DESCRIPTION": template.description,
                })
            })
            .collect();
        renderer.render_table_titled(
            &rows,
            Some("No built-in templates found."),
            Some("Label Templates"),
            Some(&["NAME", "LABELS", "DESCRIPTION"]),
        );
    }

    Ok(())
}

pub fn show_template(renderer: &Renderer, name: &str) -> Result<(), GitfleetError> {
    let template = gitfleet_core::labels::builtin(name)?;

    if renderer.is_json() {
        let json = serde_json::to_value(&template)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize template: {e}")))?;
        renderer.write_result(&json);
    } else {
        renderer.render_box(
            &format!("{}\n{}", template.name, template.description),
            "info",
        );
        let rows: Vec<serde_json::Value> = template
            .labels
            .iter()
            .map(|label| {
                serde_json::json!({
                    "NAME": label.name,
                    "RENAMES": label.rename_from.join(", "),
                    "COLOR": label.color,
                    "DESCRIPTION": label.description,
                })
            })
            .collect();
        renderer.render_table_titled(
            &rows,
            Some("No labels in template."),
            Some("Labels"),
            Some(&["NAME", "RENAMES", "COLOR", "DESCRIPTION"]),
        );
    }

    Ok(())
}

pub fn load_template(
    template_name: Option<&str>,
    file: Option<&str>,
) -> Result<LabelTemplate, GitfleetError> {
    match (template_name, file) {
        (Some(name), None) => gitfleet_core::labels::builtin(name),
        (None, Some(path)) => {
            let content = std::fs::read_to_string(path).map_err(|e| {
                GitfleetError::new(format!("Failed to read label template '{path}': {e}"))
            })?;
            gitfleet_core::labels::parse_template(&content)
        }
        _ => Err(GitfleetError::new(
            "Exactly one of --template or --file is required.",
        )),
    }
}

pub async fn sync(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    template: &LabelTemplate,
    replace: bool,
    dry_run: bool,
) -> Result<(), GitfleetError> {
    let ops = provider.label_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Labels,
        ))
    })?;

    let current = ops.list_labels(repo).await?;
    let plan = gitfleet_core::labels::plan(template, &current, replace)?;

    if dry_run {
        let counts = gitfleet_core::labels::plan_counts(&plan);
        let json = serde_json::json!({
            "dry_run": true,
            "repo": repo,
            "template": template.name,
            "replace": replace,
            "actions": plan_actions_json(&plan),
            "preserved": plan.preserved,
            "counts": counts,
        });
        render_sync(renderer, &json)?;
        return Ok(());
    }

    if plan.actions.is_empty() {
        let counts = gitfleet_core::labels::plan_counts(&plan);
        let json = serde_json::json!({
            "dry_run": false,
            "repo": repo,
            "template": template.name,
            "replace": replace,
            "actions": [],
            "preserved": plan.preserved,
            "counts": counts,
            "status": "unchanged",
        });
        render_sync(renderer, &json)?;
        return Ok(());
    }

    gitfleet_core::prompt::confirm_destructive(
        &format!("Apply {} label changes to {repo}?", plan.actions.len()),
        renderer.mode(),
        renderer.yes(),
    )?;

    let report = gitfleet_core::labels::execute(&plan, ops, repo).await;
    let mut json = serde_json::to_value(&report)
        .map_err(|e| GitfleetError::new(format!("Failed to serialize sync report: {e}")))?;
    json["dry_run"] = serde_json::json!(false);
    json["repo"] = serde_json::json!(repo);
    json["template"] = serde_json::json!(template.name);
    json["replace"] = serde_json::json!(replace);
    render_sync(renderer, &json)?;

    gitfleet_core::labels::ensure_success(&report)
}

pub async fn rename(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    old_name: &str,
    new_name: &str,
    dry_run: bool,
) -> Result<(), GitfleetError> {
    if old_name == new_name {
        return Err(GitfleetError::new("Old and new label names must differ."));
    }

    let ops = provider.label_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Labels,
        ))
    })?;
    let labels = ops.list_labels(repo).await?;
    let current_index = labels
        .iter()
        .position(|label| label.name.eq_ignore_ascii_case(old_name))
        .ok_or_else(|| GitfleetError::new(format!("Label '{old_name}' was not found.")))?;
    let current = &labels[current_index];

    if labels
        .iter()
        .enumerate()
        .any(|(index, label)| index != current_index && label.name.eq_ignore_ascii_case(new_name))
    {
        return Err(GitfleetError::new(format!(
            "Cannot rename '{old_name}' to '{new_name}' because the destination already exists."
        )));
    }

    let desired = Label {
        name: new_name.to_string(),
        color: current.color.clone(),
        new_name: None,
        description: current.description.clone(),
    };

    if dry_run {
        let json = serde_json::json!({
            "dry_run": true,
            "repo": repo,
            "action": "rename",
            "from": old_name,
            "name": new_name,
        });
        render_sync(renderer, &json)?;
        return Ok(());
    }

    gitfleet_core::prompt::confirm_destructive(
        &format!("Rename label '{old_name}' to '{new_name}'?"),
        renderer.mode(),
        renderer.yes(),
    )?;

    let result = ops.update_label(&current.name, &desired, repo).await?;
    if renderer.is_json() {
        renderer.write_result(&serde_json::json!({
            "action": "rename",
            "from": old_name,
            "name": new_name,
            "result": result,
        }));
    } else {
        renderer.render_success_box("Label renamed", &format!("{old_name} → {new_name}"));
    }

    Ok(())
}

fn render_sync(renderer: &Renderer, json: &serde_json::Value) -> Result<(), GitfleetError> {
    if renderer.is_json() {
        renderer.write_result(json);
        return Ok(());
    }

    let rows: Vec<serde_json::Value> = json
        .get("actions")
        .and_then(serde_json::Value::as_array)
        .map(|actions| {
            actions
                .iter()
                .map(|action| {
                    serde_json::json!({
                        "KIND": action.get("kind").cloned().unwrap_or(serde_json::Value::Null),
                        "NAME": action.get("name").cloned().unwrap_or(serde_json::Value::Null),
                        "STATUS": action.get("status").cloned().unwrap_or(serde_json::Value::Null),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    renderer.render_table_titled(
        &rows,
        Some("No label changes planned."),
        Some("Label Sync"),
        Some(&["KIND", "NAME", "STATUS"]),
    );
    if let Some(preserved) = json.get("preserved").and_then(serde_json::Value::as_array)
        && !preserved.is_empty()
    {
        renderer.write_value(&format!("Preserved labels: {}", preserved.len()));
    }

    Ok(())
}

fn plan_actions_json(plan: &gitfleet_core::labels::LabelSyncPlan) -> Vec<serde_json::Value> {
    plan.actions
        .iter()
        .map(|action| {
            serde_json::json!({
                "kind": action.kind,
                "name": action.name,
                "from": action.from,
                "status": "would_apply",
            })
        })
        .collect()
}
