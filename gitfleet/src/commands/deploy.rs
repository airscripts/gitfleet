use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum DeployCommand {
    #[command(about = "List deployments.")]
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        environment: Option<String>,
        #[arg(long, default_value = "10")]
        limit: u32,
        #[arg(long)]
        page: Option<u32>,
    },

    #[command(about = "Create a deployment.")]
    Create {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        r#ref: String,
        #[arg(long)]
        environment: String,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
}

pub async fn run(cmd: DeployCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        DeployCommand::List {
            repo,
            environment,
            limit,
            page,
        } => {
            crate::commands::validate_page(page)?;
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::deployments::list(
                p,
                app.renderer(),
                &repo_str,
                environment.as_deref(),
                limit,
                page,
            )
            .await
        }

        DeployCommand::Create {
            repo,
            r#ref,
            environment,
            task,
            description,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let mut input = serde_json::json!({
                "ref": r#ref,
                "environment": environment,
            });

            if let Some(t) = task {
                input["task"] = serde_json::Value::String(t);
            }

            if let Some(d) = description {
                input["description"] = serde_json::Value::String(d);
            }

            service::deployments::create(p, app.renderer(), &repo_str, input).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_deploy_list() {
        let app = test_helpers::make_app();

        run(
            DeployCommand::List {
                repo: Some("org/repo".into()),
                environment: None,
                limit: 10,
                page: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_deploy_list_with_environment() {
        let app = test_helpers::make_app();

        run(
            DeployCommand::List {
                repo: Some("org/repo".into()),
                environment: Some("production".into()),
                limit: 5,
                page: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_deploy_list_json() {
        let app = test_helpers::make_app_json();

        run(
            DeployCommand::List {
                repo: Some("org/repo".into()),
                environment: None,
                limit: 10,
                page: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_deploy_create() {
        let app = test_helpers::make_app();

        run(
            DeployCommand::Create {
                repo: Some("org/repo".into()),
                r#ref: "main".into(),
                environment: "staging".into(),
                task: None,
                description: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_deploy_create_with_task_and_description() {
        let app = test_helpers::make_app();

        run(
            DeployCommand::Create {
                repo: Some("org/repo".into()),
                r#ref: "main".into(),
                environment: "staging".into(),
                task: Some("deploy".into()),
                description: Some("Production deploy".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_deploy_create_json() {
        let app = test_helpers::make_app_json();

        run(
            DeployCommand::Create {
                repo: Some("org/repo".into()),
                r#ref: "main".into(),
                environment: "staging".into(),
                task: Some("deploy".into()),
                description: None,
            },
            &app,
        )
        .await
        .unwrap();
    }
}
