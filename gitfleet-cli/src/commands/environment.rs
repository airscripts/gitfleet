use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum EnvironmentCommand {
    #[command(about = "List environments.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create an environment.")]
    Create {
        name: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        wait_timer: Option<u32>,
    },

    #[command(about = "Delete an environment.")]
    Delete {
        name: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: EnvironmentCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        EnvironmentCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let (owner, name) = crate::repo_util::split_repo(&repo_str)?;
            service::environments::list(p, app.renderer(), owner, name).await
        }

        EnvironmentCommand::Create {
            name,
            repo,
            wait_timer,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let (owner, repo_name) = crate::repo_util::split_repo(&repo_str)?;
            service::environments::create(p, app.renderer(), owner, repo_name, &name, wait_timer)
                .await
        }

        EnvironmentCommand::Delete { name, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("{repo_str} environment {name}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would delete environment '{name}' from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete environment '{name}'?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            app.renderer()
                .write_value(&format!("Environment '{name}' deleted."));

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_env_list() {
        let app = test_helpers::make_app();

        run(
            EnvironmentCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_env_list_json() {
        let app = test_helpers::make_app_json();

        run(
            EnvironmentCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_env_list_bad_repo_format() {
        let app = test_helpers::make_app();

        let result = run(
            EnvironmentCommand::List {
                repo: Some("invalidrepo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_env_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            EnvironmentCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_env_create() {
        let app = test_helpers::make_app();

        run(
            EnvironmentCommand::Create {
                name: "staging".into(),
                repo: Some("org/repo".into()),
                wait_timer: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_env_create_with_wait_timer() {
        let app = test_helpers::make_app();

        run(
            EnvironmentCommand::Create {
                name: "staging".into(),
                repo: Some("org/repo".into()),
                wait_timer: Some(30),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_env_create_json() {
        let app = test_helpers::make_app_json();

        run(
            EnvironmentCommand::Create {
                name: "staging".into(),
                repo: Some("org/repo".into()),
                wait_timer: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_env_create_bad_repo_format() {
        let app = test_helpers::make_app();

        let result = run(
            EnvironmentCommand::Create {
                name: "staging".into(),
                repo: Some("invalidrepo".into()),
                wait_timer: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_env_create_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            EnvironmentCommand::Create {
                name: "staging".into(),
                repo: Some("org/repo".into()),
                wait_timer: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_env_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            EnvironmentCommand::Delete {
                name: "staging".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_env_delete_dry_run_json() {
        let app = test_helpers::make_app_dry_run_json();

        run(
            EnvironmentCommand::Delete {
                name: "staging".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_env_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            EnvironmentCommand::Delete {
                name: "staging".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_env_delete_silent_mode() {
        let app = test_helpers::make_app();

        run(
            EnvironmentCommand::Delete {
                name: "staging".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }
}
