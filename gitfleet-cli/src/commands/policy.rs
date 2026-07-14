use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum PolicyCommand {
    #[command(about = "Branch protection commands.")]
    BranchProtection {
        #[command(subcommand)]
        subcommand: BranchProtectionSubcommand,
    },

    #[command(about = "Tag protection commands.")]
    TagProtection {
        #[command(subcommand)]
        subcommand: TagProtectionSubcommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum BranchProtectionSubcommand {
    #[command(about = "Get branch protection.")]
    Get {
        branch: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Set branch protection.")]
    Set {
        branch: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Delete branch protection.")]
    Delete {
        branch: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum TagProtectionSubcommand {
    #[command(about = "List tag protection rules.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a tag protection rule.")]
    Create {
        pattern: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Delete a tag protection rule.")]
    Delete {
        identifier: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: PolicyCommand, app: &App) -> Result<(), GitfleetError> {
    match cmd {
        PolicyCommand::BranchProtection { subcommand } => {
            run_branch_protection(subcommand, app).await
        }

        PolicyCommand::TagProtection { subcommand } => run_tag_protection(subcommand, app).await,
    }
}

async fn run_branch_protection(
    cmd: BranchProtectionSubcommand,
    app: &App,
) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        BranchProtectionSubcommand::Get { branch, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.policy_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::RepositoryPolicies,
                ))
            })?;

            let data = ops.get_branch_protection(&repo_str, &branch).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer().render_summary(
                    "Branch Protection",
                    &[("Branch", branch), ("Enabled", "true".to_string())],
                );
            }

            Ok(())
        }

        BranchProtectionSubcommand::Set { branch, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.policy_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::RepositoryPolicies,
                ))
            })?;

            let input = serde_json::json!({
                "required_status_checks": null,
                "enforce_admins": true,
                "required_pull_request_reviews": null,
                "restrictions": null,
            });

            let data = ops.protect_branch(&repo_str, &branch, input).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer().render_success_box(
                    "Branch protection set",
                    &format!("Protection enabled for branch '{branch}'"),
                );
            }

            Ok(())
        }

        BranchProtectionSubcommand::Delete { branch, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("{repo_str} branch protection {branch}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would delete branch protection for '{branch}' from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete branch protection for '{branch}'?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let ops = p.policy_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::RepositoryPolicies,
                ))
            })?;

            ops.unprotect_branch(&repo_str, &branch).await?;

            app.renderer().render_success_box(
                "Branch protection removed",
                &format!("Protection removed for branch '{branch}'"),
            );

            Ok(())
        }
    }
}

async fn run_tag_protection(cmd: TagProtectionSubcommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        TagProtectionSubcommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.policy_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::RepositoryPolicies,
                ))
            })?;

            let rules = ops.list_tag_protection(&repo_str).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&rules)
                    .map_err(|e| GitfleetError::new(format!("Failed to serialize: {e}")))?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = rules
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "IDENTIFIER": r.identifier,
                            "PATTERN": r.pattern,
                            "CREATED": r.created_at,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No tag protection rules found."),
                    Some("Tag Protection Rules"),
                    Some(&["IDENTIFIER", "PATTERN", "CREATED"]),
                );
            }

            Ok(())
        }

        TagProtectionSubcommand::Create { pattern, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.policy_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::RepositoryPolicies,
                ))
            })?;

            let result = ops.create_tag_protection(&repo_str, &pattern).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result)
                    .map_err(|e| GitfleetError::new(format!("Failed to serialize: {e}")))?;

                app.renderer().write_result(&json);
            } else {
                app.renderer().render_success_box(
                    "Tag protection rule created",
                    &format!(
                        "Pattern '{pattern}' protected (identifier: {})",
                        result.identifier
                    ),
                );
            }

            Ok(())
        }

        TagProtectionSubcommand::Delete {
            identifier,
            repo,
            yes,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("{repo_str} tag protection rule {identifier}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would delete tag protection rule {identifier} from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete tag protection rule {identifier}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let ops = p.policy_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::RepositoryPolicies,
                ))
            })?;

            ops.delete_tag_protection(&repo_str, &identifier).await?;

            app.renderer()
                .render_success_box("Tag protection rule deleted", &identifier);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_branch_protection_get() {
        let app = test_helpers::make_app();

        run(
            PolicyCommand::BranchProtection {
                subcommand: BranchProtectionSubcommand::Get {
                    branch: "main".into(),
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_branch_protection_get_json() {
        let app = test_helpers::make_app_json();

        run(
            PolicyCommand::BranchProtection {
                subcommand: BranchProtectionSubcommand::Get {
                    branch: "main".into(),
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_branch_protection_get_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            PolicyCommand::BranchProtection {
                subcommand: BranchProtectionSubcommand::Get {
                    branch: "main".into(),
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_branch_protection_set() {
        let app = test_helpers::make_app();

        run(
            PolicyCommand::BranchProtection {
                subcommand: BranchProtectionSubcommand::Set {
                    branch: "main".into(),
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_branch_protection_set_json() {
        let app = test_helpers::make_app_json();

        run(
            PolicyCommand::BranchProtection {
                subcommand: BranchProtectionSubcommand::Set {
                    branch: "main".into(),
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_branch_protection_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            PolicyCommand::BranchProtection {
                subcommand: BranchProtectionSubcommand::Delete {
                    branch: "main".into(),
                    repo: Some("org/repo".into()),
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_branch_protection_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            PolicyCommand::BranchProtection {
                subcommand: BranchProtectionSubcommand::Delete {
                    branch: "main".into(),
                    repo: Some("org/repo".into()),
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_branch_protection_delete_json() {
        let app = test_helpers::make_app_json();

        run(
            PolicyCommand::BranchProtection {
                subcommand: BranchProtectionSubcommand::Delete {
                    branch: "main".into(),
                    repo: Some("org/repo".into()),
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tag_protection_list() {
        let app = test_helpers::make_app();

        run(
            PolicyCommand::TagProtection {
                subcommand: TagProtectionSubcommand::List {
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tag_protection_list_json() {
        let app = test_helpers::make_app_json();

        run(
            PolicyCommand::TagProtection {
                subcommand: TagProtectionSubcommand::List {
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tag_protection_create() {
        let app = test_helpers::make_app();

        run(
            PolicyCommand::TagProtection {
                subcommand: TagProtectionSubcommand::Create {
                    pattern: "v*".into(),
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tag_protection_create_json() {
        let app = test_helpers::make_app_json();

        run(
            PolicyCommand::TagProtection {
                subcommand: TagProtectionSubcommand::Create {
                    pattern: "v*".into(),
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tag_protection_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            PolicyCommand::TagProtection {
                subcommand: TagProtectionSubcommand::Delete {
                    identifier: "1".into(),
                    repo: Some("org/repo".into()),
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tag_protection_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            PolicyCommand::TagProtection {
                subcommand: TagProtectionSubcommand::Delete {
                    identifier: "1".into(),
                    repo: Some("org/repo".into()),
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tag_protection_delete_json() {
        let app = test_helpers::make_app_json();

        run(
            PolicyCommand::TagProtection {
                subcommand: TagProtectionSubcommand::Delete {
                    identifier: "1".into(),
                    repo: Some("org/repo".into()),
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tag_protection_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            PolicyCommand::TagProtection {
                subcommand: TagProtectionSubcommand::List {
                    repo: Some("org/repo".into()),
                },
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
