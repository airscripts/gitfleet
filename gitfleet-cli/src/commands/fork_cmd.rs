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
        org: Option<String>,
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

            let data = ops.get_repo(&repo_str).await?;

            if app.renderer().is_json() {
                let forks_count = data
                    .get("forks_count")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                app.renderer()
                    .write_result(&serde_json::json!({ "forks_count": forks_count }));
            } else {
                let forks_count = data
                    .get("forks_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                app.renderer().write_value(&format!("Forks: {forks_count}"));
            }

            Ok(())
        }

        ForkCmdCommand::Create { repository, org } => {
            let ops = p.repo_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Repositories,
                ))
            })?;

            let data = ops.fork_repo(&repository).await?;

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

            let _ = org;

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
                org: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_fork_create_with_org() {
        let app = test_helpers::make_app();

        run(
            ForkCmdCommand::Create {
                repository: "org/repo".into(),
                org: Some("my-org".into()),
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
                org: None,
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
                org: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
