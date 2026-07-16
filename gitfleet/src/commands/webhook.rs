use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum WebhookCommand {
    #[command(about = "List webhooks.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a webhook.")]
    Create {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        url: String,
        #[arg(long)]
        events: Vec<String>,
        #[arg(long)]
        secret: Option<String>,
        #[arg(long)]
        active: bool,
    },

    #[command(about = "Delete a webhook.")]
    Delete {
        hook_id: u64,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },

    #[command(about = "Test a webhook.")]
    Test {
        hook_id: u64,
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: WebhookCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        WebhookCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::webhooks::list(p, app.renderer(), &repo_str).await
        }

        WebhookCommand::Create {
            repo,
            url,
            events,
            secret,
            active,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let mut config = serde_json::json!({
                "url": url,
                "content_type": "json",
            });

            if let Some(ref s) = secret {
                config["secret"] = serde_json::Value::String(s.clone());
            }

            let input = serde_json::json!({
                "name": "web",
                "config": config,
                "events": events,
                "active": active,
            });

            service::webhooks::create(p, app.renderer(), &repo_str, input).await
        }

        WebhookCommand::Delete { hook_id, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("{repo_str} webhook {hook_id}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would delete webhook {hook_id} from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete webhook {hook_id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            service::webhooks::delete(p, app.renderer(), &repo_str, hook_id).await
        }

        WebhookCommand::Test { hook_id, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::webhooks::test(p, app.renderer(), &repo_str, hook_id).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_webhook_list() {
        let app = test_helpers::make_app();

        run(
            WebhookCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_webhook_list_json() {
        let app = test_helpers::make_app_json();

        run(
            WebhookCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_webhook_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            WebhookCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_webhook_create() {
        let app = test_helpers::make_app();

        run(
            WebhookCommand::Create {
                repo: Some("org/repo".into()),
                url: "https://example.com/hook".into(),
                events: vec!["push".into()],
                secret: None,
                active: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_webhook_create_with_secret() {
        let app = test_helpers::make_app();

        run(
            WebhookCommand::Create {
                repo: Some("org/repo".into()),
                url: "https://example.com/hook".into(),
                events: vec!["push".into(), "pull_request".into()],
                secret: Some("s3cr3t".into()),
                active: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_webhook_create_json() {
        let app = test_helpers::make_app_json();

        run(
            WebhookCommand::Create {
                repo: Some("org/repo".into()),
                url: "https://example.com/hook".into(),
                events: vec!["push".into()],
                secret: None,
                active: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_webhook_create_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            WebhookCommand::Create {
                repo: Some("org/repo".into()),
                url: "https://example.com/hook".into(),
                events: vec!["push".into()],
                secret: None,
                active: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_webhook_delete() {
        let app = test_helpers::make_app();

        run(
            WebhookCommand::Delete {
                hook_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_webhook_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            WebhookCommand::Delete {
                hook_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_webhook_delete_dry_run_json() {
        let app = test_helpers::make_app_dry_run_json();

        run(
            WebhookCommand::Delete {
                hook_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_webhook_delete_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            WebhookCommand::Delete {
                hook_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_webhook_test() {
        let app = test_helpers::make_app();

        run(
            WebhookCommand::Test {
                hook_id: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_webhook_test_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            WebhookCommand::Test {
                hook_id: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
