use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum VariableCommand {
    #[command(about = "List variables.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Set a variable.")]
    Set {
        name: String,
        value: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Delete a variable.")]
    Delete {
        name: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: VariableCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        VariableCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let (owner, name) = crate::repo_util::split_repo(&repo_str)?;
            service::variables::list(p, app.renderer(), owner, name).await
        }

        VariableCommand::Set { name, value, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let (owner, repo_name) = crate::repo_util::split_repo(&repo_str)?;
            service::variables::set(p, app.renderer(), owner, repo_name, &name, &value).await
        }

        VariableCommand::Delete { name, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let (owner, repo_name) = crate::repo_util::split_repo(&repo_str)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("{repo_str} variable {name}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would delete variable '{name}' from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete variable '{name}'?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            service::variables::delete(p, app.renderer(), owner, repo_name, &name).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_variable_list() {
        let app = test_helpers::make_app();

        run(
            VariableCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_variable_list_json() {
        let app = test_helpers::make_app_json();

        run(
            VariableCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_variable_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            VariableCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_variable_list_bad_repo_format() {
        let app = test_helpers::make_app();

        let result = run(
            VariableCommand::List {
                repo: Some("badrepo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_variable_set() {
        let app = test_helpers::make_app();

        run(
            VariableCommand::Set {
                name: "ENV".into(),
                value: "prod".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_variable_set_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            VariableCommand::Set {
                name: "ENV".into(),
                value: "prod".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_variable_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            VariableCommand::Delete {
                name: "ENV".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_variable_delete_dry_run_json() {
        let app = test_helpers::make_app_dry_run_json();

        run(
            VariableCommand::Delete {
                name: "ENV".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_variable_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            VariableCommand::Delete {
                name: "ENV".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_variable_delete_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            VariableCommand::Delete {
                name: "ENV".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
