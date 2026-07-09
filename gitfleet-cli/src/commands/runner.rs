use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum RunnerCommand {
    #[command(about = "List runners.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Remove a runner.")]
    Remove {
        runner_id: u64,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: RunnerCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        RunnerCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::runners::list(p, app.renderer(), &repo_str).await
        }

        RunnerCommand::Remove {
            runner_id,
            repo,
            yes,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "remove",
                        "target": format!("{repo_str} runner {runner_id}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would remove runner {runner_id} from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Remove runner {runner_id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            service::runners::remove(p, app.renderer(), &repo_str, runner_id).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_runner_list() {
        let app = test_helpers::make_app();

        run(
            RunnerCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_runner_list_json() {
        let app = test_helpers::make_app_json();

        run(
            RunnerCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_runner_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            RunnerCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runner_remove() {
        let app = test_helpers::make_app();

        run(
            RunnerCommand::Remove {
                runner_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_runner_remove_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            RunnerCommand::Remove {
                runner_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_runner_remove_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            RunnerCommand::Remove {
                runner_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runner_remove_dry_run_json() {
        let app = test_helpers::make_app_dry_run_json();

        run(
            RunnerCommand::Remove {
                runner_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }
}
