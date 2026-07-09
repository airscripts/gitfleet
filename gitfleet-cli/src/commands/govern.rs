use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum GovernCommand {
    #[command(about = "List rulesets.")]
    ListRulesets {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a ruleset.")]
    CreateRuleset {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        name: String,
        #[arg(long)]
        enforcement: Option<String>,
    },

    #[command(about = "Delete a ruleset.")]
    DeleteRuleset {
        ruleset_id: u64,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: GovernCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        GovernCommand::ListRulesets { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.governance_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Governance,
                ))
            })?;

            let data = ops.list_rulesets(&repo_str).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let items = data.as_array().cloned().unwrap_or_default();

                let rows: Vec<serde_json::Value> = items
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "ID": r.get("id"),
                            "NAME": r.get("name"),
                            "ENFORCED": r.get("enforced"),
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No rulesets found."),
                    Some("Rulesets"),
                    Some(&["ID", "NAME", "ENFORCED"]),
                );
            }

            Ok(())
        }

        GovernCommand::CreateRuleset {
            repo,
            name,
            enforcement,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let enforcement_str = enforcement.as_deref().unwrap_or("active");

            let input = gitfleet_core::types::RulesetInput {
                name,
                target: None,
                rules: None,
                enforcement: Some(enforcement_str.to_string()),
                conditions: None,
            };

            let ops = p.governance_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Governance,
                ))
            })?;

            let data = ops.create_ruleset(&repo_str, &input).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer().render_success_box(
                    "Ruleset created",
                    data.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                );
            }

            Ok(())
        }

        GovernCommand::DeleteRuleset {
            ruleset_id,
            repo,
            yes,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("{repo_str} ruleset {ruleset_id}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would delete ruleset {ruleset_id} from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete ruleset {ruleset_id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let ops = p.governance_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Governance,
                ))
            })?;

            ops.delete_ruleset(&repo_str, ruleset_id).await?;

            app.renderer()
                .render_success_box("Ruleset deleted", &ruleset_id.to_string());

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_list_rulesets() {
        let app = test_helpers::make_app();

        run(
            GovernCommand::ListRulesets {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_list_rulesets_json() {
        let app = test_helpers::make_app_json();

        run(
            GovernCommand::ListRulesets {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_list_rulesets_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            GovernCommand::ListRulesets {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_ruleset() {
        let app = test_helpers::make_app();

        run(
            GovernCommand::CreateRuleset {
                repo: Some("org/repo".into()),
                name: "ruleset1".into(),
                enforcement: Some("active".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_create_ruleset_default_enforcement() {
        let app = test_helpers::make_app();

        run(
            GovernCommand::CreateRuleset {
                repo: Some("org/repo".into()),
                name: "ruleset1".into(),
                enforcement: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_create_ruleset_json() {
        let app = test_helpers::make_app_json();

        run(
            GovernCommand::CreateRuleset {
                repo: Some("org/repo".into()),
                name: "ruleset1".into(),
                enforcement: Some("active".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_delete_ruleset_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            GovernCommand::DeleteRuleset {
                ruleset_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_delete_ruleset_with_yes() {
        let app = test_helpers::make_app();

        run(
            GovernCommand::DeleteRuleset {
                ruleset_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_delete_ruleset_json() {
        let app = test_helpers::make_app_json();

        run(
            GovernCommand::DeleteRuleset {
                ruleset_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_delete_ruleset_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            GovernCommand::DeleteRuleset {
                ruleset_id: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
