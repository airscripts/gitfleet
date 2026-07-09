use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum AccessCommand {
    #[command(about = "Organization commands.")]
    Org {
        #[command(subcommand)]
        subcommand: OrgSubcommand,
    },

    #[command(about = "Team commands.")]
    Team {
        #[command(subcommand)]
        subcommand: TeamSubcommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum OrgSubcommand {
    #[command(about = "List organization members.")]
    ListMembers { org: String },

    #[command(about = "Invite a member to an organization.")]
    Invite {
        org: String,
        username: String,
        #[arg(long, default_value = "member")]
        role: String,
    },

    #[command(about = "Remove a member from an organization.")]
    Remove {
        org: String,
        username: String,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum TeamSubcommand {
    #[command(about = "List teams.")]
    List { org: String },

    #[command(about = "Create a team.")]
    Create { org: String, name: String },

    #[command(about = "List team members.")]
    ListMembers { org: String, team_slug: String },
}

pub async fn run(cmd: AccessCommand, app: &App) -> Result<(), GitfleetError> {
    match cmd {
        AccessCommand::Org { subcommand } => run_org(subcommand, app).await,
        AccessCommand::Team { subcommand } => run_team(subcommand, app).await,
    }
}

async fn run_org(cmd: OrgSubcommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        OrgSubcommand::ListMembers { org } => {
            let ops = p.access_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Access,
                ))
            })?;

            let data = ops.list_org_members(&org).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let items = data.as_array().cloned().unwrap_or_default();

                let rows: Vec<serde_json::Value> = items
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "LOGIN": m.get("login"),
                            "ID": m.get("id"),
                            "TYPE": m.get("type"),
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No members found."),
                    Some("Members"),
                    Some(&["LOGIN", "ID", "TYPE"]),
                );
            }

            Ok(())
        }

        OrgSubcommand::Invite {
            org,
            username,
            role,
        } => {
            let ops = p.access_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Access,
                ))
            })?;

            ops.invite_collaborator(&org, &org, &username, &role)
                .await?;

            app.renderer()
                .render_success_box("Invitation sent", &format!("{username} to {org}"));

            Ok(())
        }

        OrgSubcommand::Remove { org, username, yes } => {
            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "remove",
                        "target": format!("{username} from {org}"),
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would remove {username} from {org}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Remove {username} from {org}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let ops = p.access_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Access,
                ))
            })?;

            ops.remove_org_member(&org, &username).await?;

            app.renderer()
                .render_success_box("Member removed", &format!("{username} from {org}"));

            Ok(())
        }
    }
}

async fn run_team(cmd: TeamSubcommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.access_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            p.id(),
            ProviderCapability::Access,
        ))
    })?;

    match cmd {
        TeamSubcommand::List { org } => {
            let data = ops.list_teams(&org).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let items = data.as_array().cloned().unwrap_or_default();

                let rows: Vec<serde_json::Value> = items
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "NAME": t.get("name"),
                            "SLUG": t.get("slug"),
                            "ID": t.get("id"),
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No teams found."),
                    Some("Teams"),
                    Some(&["NAME", "SLUG", "ID"]),
                );
            }

            Ok(())
        }

        TeamSubcommand::Create { org, name } => {
            let data = ops.create_team(&org, &name).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer().render_success_box("Team created", &name);
            }

            Ok(())
        }

        TeamSubcommand::ListMembers { org, team_slug } => {
            let data = ops.list_team_members(&org, &team_slug).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let items = data.as_array().cloned().unwrap_or_default();

                let rows: Vec<serde_json::Value> = items
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "LOGIN": m.get("login"),
                            "ID": m.get("id"),
                            "ROLE": m.get("role"),
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No team members found."),
                    Some("Team Members"),
                    Some(&["LOGIN", "ID", "ROLE"]),
                );
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
    async fn test_org_list_members() {
        let app = test_helpers::make_app();

        run(
            AccessCommand::Org {
                subcommand: OrgSubcommand::ListMembers {
                    org: "myorg".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_org_list_members_json() {
        let app = test_helpers::make_app_json();

        run(
            AccessCommand::Org {
                subcommand: OrgSubcommand::ListMembers {
                    org: "myorg".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_org_invite() {
        let app = test_helpers::make_app();

        run(
            AccessCommand::Org {
                subcommand: OrgSubcommand::Invite {
                    org: "myorg".into(),
                    username: "dev".into(),
                    role: "member".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_org_invite_custom_role() {
        let app = test_helpers::make_app();

        run(
            AccessCommand::Org {
                subcommand: OrgSubcommand::Invite {
                    org: "myorg".into(),
                    username: "admin".into(),
                    role: "admin".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_org_remove_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            AccessCommand::Org {
                subcommand: OrgSubcommand::Remove {
                    org: "myorg".into(),
                    username: "dev".into(),
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_org_remove_with_yes() {
        let app = test_helpers::make_app();

        run(
            AccessCommand::Org {
                subcommand: OrgSubcommand::Remove {
                    org: "myorg".into(),
                    username: "dev".into(),
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_org_remove_json() {
        let app = test_helpers::make_app_json();

        run(
            AccessCommand::Org {
                subcommand: OrgSubcommand::Remove {
                    org: "myorg".into(),
                    username: "dev".into(),
                    yes: true,
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_org_list_members_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            AccessCommand::Org {
                subcommand: OrgSubcommand::ListMembers {
                    org: "myorg".into(),
                },
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_team_list() {
        let app = test_helpers::make_app();

        run(
            AccessCommand::Team {
                subcommand: TeamSubcommand::List {
                    org: "myorg".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_team_list_json() {
        let app = test_helpers::make_app_json();

        run(
            AccessCommand::Team {
                subcommand: TeamSubcommand::List {
                    org: "myorg".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_team_create() {
        let app = test_helpers::make_app();

        run(
            AccessCommand::Team {
                subcommand: TeamSubcommand::Create {
                    org: "myorg".into(),
                    name: "team1".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_team_create_json() {
        let app = test_helpers::make_app_json();

        run(
            AccessCommand::Team {
                subcommand: TeamSubcommand::Create {
                    org: "myorg".into(),
                    name: "team1".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_team_list_members() {
        let app = test_helpers::make_app();

        run(
            AccessCommand::Team {
                subcommand: TeamSubcommand::ListMembers {
                    org: "myorg".into(),
                    team_slug: "team1".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_team_list_members_json() {
        let app = test_helpers::make_app_json();

        run(
            AccessCommand::Team {
                subcommand: TeamSubcommand::ListMembers {
                    org: "myorg".into(),
                    team_slug: "team1".into(),
                },
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_team_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            AccessCommand::Team {
                subcommand: TeamSubcommand::List {
                    org: "myorg".into(),
                },
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
