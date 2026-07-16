use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum IdentityCommand {
    #[command(about = "SSH key commands.")]
    SshKey {
        #[command(subcommand)]
        subcommand: SshKeySubcommand,
    },

    #[command(about = "GPG key commands.")]
    GpgKey {
        #[command(subcommand)]
        subcommand: GpgKeySubcommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum SshKeySubcommand {
    #[command(about = "List SSH keys.")]
    List,

    #[command(about = "Add an SSH key.")]
    Add { title: String, key: String },

    #[command(about = "Delete an SSH key.")]
    Delete {
        key_id: u64,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum GpgKeySubcommand {
    #[command(about = "List GPG keys.")]
    List,

    #[command(about = "Add a GPG key.")]
    Add { armored_public_key: String },

    #[command(about = "Delete a GPG key.")]
    Delete {
        key_id: u64,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: IdentityCommand, app: &App) -> Result<(), GitfleetError> {
    match cmd {
        IdentityCommand::SshKey { subcommand } => run_ssh_key(subcommand, app).await,
        IdentityCommand::GpgKey { subcommand } => run_gpg_key(subcommand, app).await,
    }
}

async fn run_ssh_key(cmd: SshKeySubcommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        SshKeySubcommand::List => {
            let ops = p.identity_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Identity,
                ))
            })?;

            let keys = ops.list_ssh_keys().await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&keys).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize SSH keys: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = keys
                    .iter()
                    .map(|k| {
                        serde_json::json!({
                            "ID": k.id,
                            "TITLE": k.title,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No SSH keys found."),
                    Some("SSH Keys"),
                    None,
                );
            }

            Ok(())
        }

        SshKeySubcommand::Add { title, key } => {
            let ops = p.identity_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Identity,
                ))
            })?;

            let result: gitfleet_core::types::SshKeySummary = ops.add_ssh_key(&title, &key).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result)
                    .map_err(|e| GitfleetError::new(format!("Failed to serialize SSH key: {e}")))?;

                app.renderer().write_result(&json);
            } else {
                app.renderer().render_success_box("SSH key added", &title);
            }

            Ok(())
        }

        SshKeySubcommand::Delete { key_id, yes } => {
            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("SSH key {key_id}"),
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would delete SSH key {key_id}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete SSH key {key_id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let ops = p.identity_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Identity,
                ))
            })?;

            ops.delete_ssh_key(key_id).await?;

            app.renderer()
                .render_success_box("SSH key deleted", &key_id.to_string());

            Ok(())
        }
    }
}

async fn run_gpg_key(cmd: GpgKeySubcommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        GpgKeySubcommand::List => {
            let ops = p.identity_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Identity,
                ))
            })?;

            let keys = ops.list_gpg_keys().await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&keys).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize GPG keys: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = keys
                    .iter()
                    .map(|k| {
                        serde_json::json!({
                            "ID": k.id,
                            "NAME": k.name,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No GPG keys found."),
                    Some("GPG Keys"),
                    None,
                );
            }

            Ok(())
        }

        GpgKeySubcommand::Add { armored_public_key } => {
            let ops = p.identity_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Identity,
                ))
            })?;

            let result = ops.add_gpg_key(&armored_public_key).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result)
                    .map_err(|e| GitfleetError::new(format!("Failed to serialize GPG key: {e}")))?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("GPG key added", &result.name);
            }

            Ok(())
        }

        GpgKeySubcommand::Delete { key_id, yes } => {
            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("GPG key {key_id}"),
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would delete GPG key {key_id}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete GPG key {key_id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let ops = p.identity_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Identity,
                ))
            })?;

            ops.delete_gpg_key(key_id).await?;

            app.renderer()
                .render_success_box("GPG key deleted", &key_id.to_string());

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_ssh_key_list() {
        let app = test_helpers::make_app();

        run(
            IdentityCommand::SshKey {
                subcommand: SshKeySubcommand::List,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_ssh_key_list_json() {
        let app = test_helpers::make_app_json();

        run(
            IdentityCommand::SshKey {
                subcommand: SshKeySubcommand::List,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_ssh_key_add() {
        let app = test_helpers::make_app();

        run(
            IdentityCommand::SshKey {
                subcommand: SshKeySubcommand::Add {
                    title: "new-key".into(),
                    key: "ssh-rsa AAAAB3...".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_ssh_key_add_json() {
        let app = test_helpers::make_app_json();

        run(
            IdentityCommand::SshKey {
                subcommand: SshKeySubcommand::Add {
                    title: "new-key".into(),
                    key: "ssh-rsa AAAAB3...".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_ssh_key_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            IdentityCommand::SshKey {
                subcommand: SshKeySubcommand::Delete {
                    key_id: 1,
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_ssh_key_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            IdentityCommand::SshKey {
                subcommand: SshKeySubcommand::Delete {
                    key_id: 1,
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_ssh_key_delete_json() {
        let app = test_helpers::make_app_json();

        run(
            IdentityCommand::SshKey {
                subcommand: SshKeySubcommand::Delete {
                    key_id: 1,
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_ssh_key_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            IdentityCommand::SshKey {
                subcommand: SshKeySubcommand::List,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gpg_key_list() {
        let app = test_helpers::make_app();

        run(
            IdentityCommand::GpgKey {
                subcommand: GpgKeySubcommand::List,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_gpg_key_list_json() {
        let app = test_helpers::make_app_json();

        run(
            IdentityCommand::GpgKey {
                subcommand: GpgKeySubcommand::List,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_gpg_key_add() {
        let app = test_helpers::make_app();

        run(
            IdentityCommand::GpgKey {
                subcommand: GpgKeySubcommand::Add {
                    armored_public_key: "-----BEGIN PGP PUBLIC KEY-----...".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_gpg_key_add_json() {
        let app = test_helpers::make_app_json();

        run(
            IdentityCommand::GpgKey {
                subcommand: GpgKeySubcommand::Add {
                    armored_public_key: "-----BEGIN PGP PUBLIC KEY-----...".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_gpg_key_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            IdentityCommand::GpgKey {
                subcommand: GpgKeySubcommand::Delete {
                    key_id: 1,
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_gpg_key_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            IdentityCommand::GpgKey {
                subcommand: GpgKeySubcommand::Delete {
                    key_id: 1,
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_gpg_key_delete_json() {
        let app = test_helpers::make_app_json();

        run(
            IdentityCommand::GpgKey {
                subcommand: GpgKeySubcommand::Delete {
                    key_id: 1,
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_gpg_key_add_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            IdentityCommand::GpgKey {
                subcommand: GpgKeySubcommand::Add {
                    armored_public_key: "-----BEGIN PGP PUBLIC KEY-----...".into(),
                },
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
