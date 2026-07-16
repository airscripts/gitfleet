use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, SecretEncryptionError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum SecretCmdCommand {
    #[command(about = "List secrets.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Set a secret.")]
    Set {
        name: String,
        value: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Delete a secret.")]
    Delete {
        name: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },

    #[command(about = "Get the public key for encrypting secrets.")]
    PublicKey {
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: SecretCmdCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        SecretCmdCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let (owner, name) = crate::repo_util::split_repo(&repo_str)?;
            service::secrets::list(p, app.renderer(), owner, name).await
        }

        SecretCmdCommand::Set { name, value, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let (owner, repo_name) = crate::repo_util::split_repo(&repo_str)?;
            let ops = p.secret_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Secrets,
                ))
            })?;

            let key_data = ops.get_repo_public_key(owner, repo_name).await?;

            let encrypted =
                gitfleet_core::secrets::encrypt_secret(&value, &key_data.key).map_err(|e| {
                    GitfleetError::from(SecretEncryptionError::new(format!(
                        "Failed to encrypt secret: {e}"
                    )))
                })?;

            ops.set_repo_secret(owner, repo_name, &name, &encrypted, &key_data.key_id)
                .await?;

            app.renderer().render_success_box("Secret set", &name);

            Ok(())
        }

        SecretCmdCommand::Delete { name, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let (owner, repo_name) = crate::repo_util::split_repo(&repo_str)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("{repo_str} secret {name}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would delete secret '{name}' from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete secret '{name}'?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            service::secrets::delete(p, app.renderer(), owner, repo_name, &name).await
        }

        SecretCmdCommand::PublicKey { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let (owner, name) = crate::repo_util::split_repo(&repo_str)?;
            service::secrets::get_public_key(p, app.renderer(), owner, name).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_secret_list() {
        let app = test_helpers::make_app();

        run(
            SecretCmdCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_secret_list_json() {
        let app = test_helpers::make_app_json();

        run(
            SecretCmdCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_secret_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SecretCmdCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_secret_list_bad_repo_format() {
        let app = test_helpers::make_app();

        let result = run(
            SecretCmdCommand::List {
                repo: Some("badrepo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_secret_set_encrypt_error() {
        let app = test_helpers::make_app();

        let result = run(
            SecretCmdCommand::Set {
                name: "TOKEN".into(),
                value: "secret-value".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_secret_set_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SecretCmdCommand::Set {
                name: "TOKEN".into(),
                value: "secret-value".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_secret_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            SecretCmdCommand::Delete {
                name: "TOKEN".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_secret_delete_dry_run_json() {
        let app = test_helpers::make_app_dry_run_json();

        run(
            SecretCmdCommand::Delete {
                name: "TOKEN".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_secret_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            SecretCmdCommand::Delete {
                name: "TOKEN".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_secret_delete_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SecretCmdCommand::Delete {
                name: "TOKEN".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_secret_public_key() {
        let app = test_helpers::make_app();

        run(
            SecretCmdCommand::PublicKey {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_secret_public_key_json() {
        let app = test_helpers::make_app_json();

        run(
            SecretCmdCommand::PublicKey {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_secret_public_key_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SecretCmdCommand::PublicKey {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
