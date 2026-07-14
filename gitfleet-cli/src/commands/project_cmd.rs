use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnprocessableError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum ProjectCmdCommand {
    #[command(about = "List projects.")]
    List {
        #[arg(long)]
        owner: Option<String>,
        #[arg(long, default_value = "10")]
        limit: u32,
    },

    #[command(about = "View a project.")]
    View { id: u64 },

    #[command(about = "Create a project.")]
    Create {
        #[arg(long)]
        owner: Option<String>,
        title: String,
        #[arg(long)]
        body: Option<String>,
    },

    #[command(about = "Delete a project.")]
    Delete {
        id: u64,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: ProjectCmdCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.planning_ops().ok_or_else(|| {
        GitfleetError::UnsupportedCapability(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::Projects,
        ))
    })?;

    match cmd {
        ProjectCmdCommand::List { owner, limit } => {
            let owner_str = owner.as_deref().ok_or_else(|| {
                GitfleetError::from(UnprocessableError::new(
                    "Project owner is required. Use --owner OWNER.",
                ))
            })?;

            let data = ops.list_projects(owner_str, limit).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&data).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize projects: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = data
                    .iter()
                    .map(|proj| {
                        serde_json::json!({
                            "ID": proj.id,
                            "NUMBER": proj.number,
                            "TITLE": proj.title,
                            "CLOSED": proj.closed,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No projects found."),
                    Some("Projects"),
                    Some(&["ID", "NUMBER", "TITLE", "CLOSED"]),
                );
            }

            Ok(())
        }

        ProjectCmdCommand::View { id } => {
            let data = ops.get_project(&id.to_string()).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer()
                    .render_success_box("Project", &id.to_string());
            }

            Ok(())
        }

        ProjectCmdCommand::Create { owner, title, body } => {
            let owner_str = owner.as_deref().ok_or_else(|| {
                GitfleetError::from(UnprocessableError::new(
                    "Project owner is required. Use --owner OWNER.",
                ))
            })?;

            let result = ops
                .create_project(owner_str, &title, body.as_deref())
                .await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result)
                    .map_err(|e| GitfleetError::new(format!("Failed to serialize project: {e}")))?;

                app.renderer().write_result(&json);
            } else {
                app.renderer().render_success_box("Project created", &title);
            }

            Ok(())
        }

        ProjectCmdCommand::Delete { id, yes } => {
            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("project {id}"),
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would delete project {id}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete project {id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            ops.delete_project(&id.to_string()).await?;

            app.renderer()
                .render_success_box("Project deleted", &id.to_string());

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_project_list() {
        let app = test_helpers::make_app();

        run(
            ProjectCmdCommand::List {
                owner: Some("org".into()),
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_project_list_default_owner() {
        let app = test_helpers::make_app();

        let result = run(
            ProjectCmdCommand::List {
                owner: None,
                limit: 5,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_project_list_json() {
        let app = test_helpers::make_app_json();

        run(
            ProjectCmdCommand::List {
                owner: Some("org".into()),
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_project_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ProjectCmdCommand::List {
                owner: Some("org".into()),
                limit: 10,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_project_view() {
        let app = test_helpers::make_app();

        run(ProjectCmdCommand::View { id: 1 }, &app).await.unwrap();
    }

    #[tokio::test]
    async fn test_project_view_json() {
        let app = test_helpers::make_app_json();

        run(ProjectCmdCommand::View { id: 1 }, &app).await.unwrap();
    }

    #[tokio::test]
    async fn test_project_create() {
        let app = test_helpers::make_app();

        run(
            ProjectCmdCommand::Create {
                owner: Some("org".into()),
                title: "New Project".into(),
                body: Some("Description".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_project_create_json() {
        let app = test_helpers::make_app_json();

        run(
            ProjectCmdCommand::Create {
                owner: Some("org".into()),
                title: "New Project".into(),
                body: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_project_delete_with_yes() {
        let app = test_helpers::make_app();

        run(ProjectCmdCommand::Delete { id: 1, yes: true }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_project_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(ProjectCmdCommand::Delete { id: 1, yes: true }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_project_delete_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(ProjectCmdCommand::Delete { id: 1, yes: true }, &app).await;

        assert!(result.is_err());
    }
}
