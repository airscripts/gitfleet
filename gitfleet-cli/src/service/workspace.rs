use gitfleet_core::errors::{GitfleetError, PartialFailureError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

pub async fn archive(name: &str, app: &App) -> Result<(), GitfleetError> {
    let workspace = gitfleet_core::workspace::get(name)?;
    let provider = app.provider()?;

    let repo_ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::Repositories,
        ))
    })?;

    let mut results = Vec::with_capacity(workspace.repositories.len());
    let mut has_partial_failure = false;

    for repository in workspace.repositories {
        let target = format!("{}/{}", repository.namespace, repository.name);

        if repository.provider != app.provider_id() || repository.host != app.provider_host() {
            has_partial_failure = true;

            results.push(serde_json::json!({
                "repository": target,
                "provider": repository.provider.to_string(),
                "host": repository.host,
                "status": "skipped",
                "reason": "Repository does not match the active provider profile.",
            }));

            continue;
        }

        results.push(serde_json::json!({
            "repository": target,
            "provider": repository.provider.to_string(),
            "host": repository.host,
            "status": if app.dry_run() { "would_archive" } else { "pending" },
        }));
    }

    let has_pending_targets = results.iter().any(|result| result["status"] == "pending");

    if !app.dry_run() && has_pending_targets {
        gitfleet_core::prompt::confirm_destructive(
            &format!("Archive compatible repositories in workspace '{name}'?"),
            app.renderer().mode(),
            app.renderer().yes(),
        )?;
    }

    if !app.dry_run() && has_pending_targets {
        for result in &mut results {
            if result["status"] != "pending" {
                continue;
            }

            let target = result["repository"].as_str().unwrap_or_default();

            match repo_ops.archive_repo(target).await {
                Ok(()) => result["status"] = serde_json::json!("archived"),
                Err(error) => {
                    has_partial_failure = true;
                    result["status"] = serde_json::json!("failed");
                    result["reason"] = serde_json::json!(error.to_string());
                }
            }
        }
    }

    let archived = results
        .iter()
        .filter(|result| result["status"] == "archived")
        .count();

    let would_archive = results
        .iter()
        .filter(|result| result["status"] == "would_archive")
        .count();

    let skipped = results
        .iter()
        .filter(|result| result["status"] == "skipped")
        .count();

    let failed = results
        .iter()
        .filter(|result| result["status"] == "failed")
        .count();

    let report = serde_json::json!({
        "operation": "archive",
        "workspace": name,
        "provider": app.provider_id().to_string(),
        "host": app.provider_host(),
        "dry_run": app.dry_run(),
        "results": results,
        "summary": {
            "total": archived + would_archive + skipped + failed,
            "archived": archived,
            "would_archive": would_archive,
            "skipped": skipped,
            "failed": failed,
        },
    });

    if app.renderer().is_json() {
        app.renderer().write_result(&report);
    } else {
        let rows = report["results"].as_array().cloned().unwrap_or_default();

        app.renderer().render_table_titled(
            &rows,
            Some("Workspace has no repositories."),
            Some(&format!("Workspace '{name}' archive")),
            None,
        );
    }

    if has_partial_failure {
        return Err(GitfleetError::from(PartialFailureError::new(
            "Workspace archive completed with skipped or failed repositories.",
        )));
    }

    Ok(())
}
