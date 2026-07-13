use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum PipelineCommand {
    #[command(about = "List workflow definitions.")]
    ListDef {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, default_value = "10")]
        limit: u32,
        #[arg(long)]
        page: Option<u32>,
    },

    #[command(about = "View a workflow definition.")]
    ViewDef {
        workflow_id: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "List workflow runs.")]
    ListRuns {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, default_value = "")]
        filter: String,
        #[arg(long, default_value = "10")]
        limit: u32,
    },

    #[command(about = "View a workflow run.")]
    ViewRun {
        run_id: u64,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Trigger a workflow run.")]
    Trigger {
        workflow_id: String,
        #[arg(long)]
        r#ref: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Cancel a workflow run.")]
    Cancel {
        run_id: u64,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },

    #[command(about = "Re-run a workflow.")]
    Rerun {
        run_id: u64,
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: PipelineCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        PipelineCommand::ListDef { repo, limit, page } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::pipelines::list_workflows(p, app.renderer(), &repo_str, limit, page).await
        }

        PipelineCommand::ViewDef { workflow_id, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::pipelines::view_workflow(p, app.renderer(), &repo_str, &workflow_id).await
        }

        PipelineCommand::ListRuns {
            repo,
            filter,
            limit,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::pipelines::list_runs(p, app.renderer(), &repo_str, &filter, limit).await
        }

        PipelineCommand::ViewRun { run_id, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::pipelines::view_run(p, app.renderer(), &repo_str, run_id).await
        }

        PipelineCommand::Trigger {
            workflow_id,
            r#ref,
            repo,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::pipelines::trigger_run(
                p,
                app.renderer(),
                &repo_str,
                &workflow_id,
                &r#ref,
                None,
            )
            .await
        }

        PipelineCommand::Cancel { run_id, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                let preview = serde_json::json!({
                    "dry_run": true,
                    "action": "cancel",
                    "target": format!("{repo_str} run {run_id}"),
                });

                if app.renderer().is_json() {
                    app.renderer().write_result(&preview);
                } else {
                    app.renderer()
                        .render_box(&format!("Would cancel workflow run {run_id}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Cancel workflow run {run_id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            service::pipelines::cancel_run(p, app.renderer(), &repo_str, run_id).await
        }

        PipelineCommand::Rerun { run_id, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.pipeline_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Pipelines,
                ))
            })?;

            ops.rerun(&repo_str, run_id).await?;

            app.renderer()
                .render_success_box("Run re-triggered", &run_id.to_string());

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_pipeline_list_def() {
        let app = test_helpers::make_app();

        run(
            PipelineCommand::ListDef {
                repo: Some("org/repo".into()),
                limit: 10,
                page: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_list_def_json() {
        let app = test_helpers::make_app_json();

        run(
            PipelineCommand::ListDef {
                repo: Some("org/repo".into()),
                limit: 10,
                page: Some(1),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_view_def() {
        let app = test_helpers::make_app();

        run(
            PipelineCommand::ViewDef {
                workflow_id: "ci.yml".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_view_def_json() {
        let app = test_helpers::make_app_json();

        run(
            PipelineCommand::ViewDef {
                workflow_id: "ci.yml".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_list_runs() {
        let app = test_helpers::make_app();

        run(
            PipelineCommand::ListRuns {
                repo: Some("org/repo".into()),
                filter: "".into(),
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_list_runs_json() {
        let app = test_helpers::make_app_json();

        run(
            PipelineCommand::ListRuns {
                repo: Some("org/repo".into()),
                filter: "status=completed".into(),
                limit: 5,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_view_run() {
        let app = test_helpers::make_app();

        run(
            PipelineCommand::ViewRun {
                run_id: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_view_run_json() {
        let app = test_helpers::make_app_json();

        run(
            PipelineCommand::ViewRun {
                run_id: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_trigger() {
        let app = test_helpers::make_app();

        run(
            PipelineCommand::Trigger {
                workflow_id: "ci.yml".into(),
                r#ref: "main".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_cancel() {
        let app = test_helpers::make_app();

        run(
            PipelineCommand::Cancel {
                run_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_rerun() {
        let app = test_helpers::make_app();

        run(
            PipelineCommand::Rerun {
                run_id: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pipeline_rerun_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            PipelineCommand::Rerun {
                run_id: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pipeline_list_def_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            PipelineCommand::ListDef {
                repo: Some("org/repo".into()),
                limit: 10,
                page: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
