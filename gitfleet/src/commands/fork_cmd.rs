use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum ForkCmdCommand {
    #[command(about = "List forks of a repository.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a fork of a repository.")]
    Create {
        repository: String,
        #[arg(long)]
        owner: Option<String>,
    },
}

pub async fn run(cmd: ForkCmdCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        ForkCmdCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.repo_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Repositories,
                ))
            })?;

            let forks = ops.list_forks(&repo_str).await?;

            if app.renderer().is_json() {
                let data = serde_json::to_value(&forks).map_err(|error| {
                    GitfleetError::new(format!("Failed to serialize forks: {error}"))
                })?;

                app.renderer().write_result(&data);
            } else {
                let rows: Vec<serde_json::Value> = forks
                    .iter()
                    .map(|fork| {
                        serde_json::json!({
                            "NAME": fork.full_name,
                            "VISIBILITY": if fork.private { "private" } else { "public" },
                            "DEFAULT BRANCH": fork.default_branch,
                            "ARCHIVED": fork.archived,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No forks found."),
                    Some("Forks"),
                    Some(&["NAME", "VISIBILITY", "DEFAULT BRANCH", "ARCHIVED"]),
                );
            }

            Ok(())
        }

        ForkCmdCommand::Create { repository, owner } => {
            let ops = p.repo_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Repositories,
                ))
            })?;

            let data = ops.fork_repo(&repository, owner.as_deref()).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let full_name = data
                    .get("full_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&repository);

                let html_url = data.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
                app.renderer()
                    .render_success_box("Fork created", &format!("{full_name}\n{html_url}"));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_fork_list() {
        let app = test_helpers::make_app();

        run(
            ForkCmdCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_fork_list_json() {
        let app = test_helpers::make_app_json();

        run(
            ForkCmdCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_fork_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ForkCmdCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fork_create() {
        let app = test_helpers::make_app();

        run(
            ForkCmdCommand::Create {
                repository: "org/repo".into(),
                owner: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_fork_create_with_owner() {
        let app = test_helpers::make_app();

        run(
            ForkCmdCommand::Create {
                repository: "org/repo".into(),
                owner: Some("my-org".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_fork_create_json() {
        let app = test_helpers::make_app_json();

        run(
            ForkCmdCommand::Create {
                repository: "org/repo".into(),
                owner: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_fork_create_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ForkCmdCommand::Create {
                repository: "org/repo".into(),
                owner: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
