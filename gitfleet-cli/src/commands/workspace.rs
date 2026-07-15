use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum WorkspaceCommand {
    #[command(about = "Define or update a named workspace.")]
    Define {
        #[arg(long)]
        name: String,
        #[arg(long)]
        repos: Vec<String>,
    },

    #[command(about = "List all workspaces.")]
    List,

    #[command(about = "Remove a workspace.")]
    Remove { name: String },

    #[command(about = "Archive all compatible repositories in a workspace.")]
    Archive { name: String },
}

pub async fn run(cmd: WorkspaceCommand, app: &App) -> Result<(), GitfleetError> {
    match cmd {
        WorkspaceCommand::Define { name, repos } => {
            let replacing = gitfleet_core::workspace::list()?
                .iter()
                .any(|workspace| workspace.name == name);

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": if replacing { "update" } else { "define" },
                        "workspace": name,
                        "repositories": repos,
                    }));
                } else {
                    app.renderer().render_box(
                        &format!(
                            "Would {} workspace '{name}'",
                            if replacing { "update" } else { "define" }
                        ),
                        "warning",
                    );
                }

                return Ok(());
            }

            if replacing {
                gitfleet_core::prompt::confirm_destructive(
                    &format!("Replace workspace '{name}'?"),
                    app.renderer().mode(),
                    app.renderer().yes(),
                )?;
            }

            let ws = gitfleet_core::workspace::define_with_defaults(
                &name,
                &repos,
                app.provider_id(),
                app.provider_host(),
            )?;

            if app.renderer().is_json() {
                app.renderer()
                    .write_result(&serde_json::to_value(&ws).unwrap_or_default());
            } else {
                app.renderer().render_success_box(
                    "Workspace defined",
                    &format!("'{}' with {} repositories", ws.name, ws.repositories.len()),
                );
            }

            Ok(())
        }

        WorkspaceCommand::List => {
            let workspaces = gitfleet_core::workspace::list()?;

            if app.renderer().is_json() {
                app.renderer()
                    .write_result(&serde_json::to_value(&workspaces).unwrap_or_default());
            } else {
                if workspaces.is_empty() {
                    app.renderer().write_value("No workspaces defined.");
                } else {
                    let rows: Vec<serde_json::Value> = workspaces
                        .iter()
                        .map(|ws| {
                            serde_json::json!({
                                "NAME": ws.name,
                                "REPOS": ws.repositories.len(),
                            })
                        })
                        .collect();

                    app.renderer().render_table_titled(
                        &rows,
                        Some("No workspaces defined."),
                        Some("Workspaces"),
                        None,
                    );
                }
            }

            Ok(())
        }

        WorkspaceCommand::Remove { name } => {
            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "remove",
                        "workspace": name,
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would remove workspace '{name}'"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Remove workspace '{name}'?"),
                app.renderer().mode(),
                app.renderer().yes(),
            )?;

            gitfleet_core::workspace::remove(&name)?;

            app.renderer()
                .render_success_box("Workspace removed", &name);

            Ok(())
        }

        WorkspaceCommand::Archive { name } => service::workspace::archive(&name, app).await,
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::output::Renderer;
    use gitfleet_core::output_state::OutputMode;
    use gitfleet_core::provider::ProviderId;
    use gitfleet_providers::{GitHubProvider, ProviderRegistry};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::super::test_helpers;
    use super::*;

    struct TestEnvironment {
        _directory: tempfile::TempDir,
        original_home: Option<String>,
    }

    impl Drop for TestEnvironment {
        fn drop(&mut self) {
            if let Some(home) = &self.original_home {
                // SAFETY: This test serializes process-environment mutation with `serial_test`.
                unsafe { std::env::set_var("GITFLEET_HOME", home) };
            } else {
                // SAFETY: This test serializes process-environment mutation with `serial_test`.
                unsafe { std::env::remove_var("GITFLEET_HOME") };
            }
        }
    }

    fn setup_test_env() -> TestEnvironment {
        let dir = tempfile::tempdir().unwrap();

        let gitfleet_dir = dir.path().join(".config").join("gitfleet");
        std::fs::create_dir_all(&gitfleet_dir).unwrap();

        let original_home = std::env::var("GITFLEET_HOME").ok();

        // SAFETY: This test serializes process-environment mutation with `serial_test`.
        unsafe { std::env::set_var("GITFLEET_HOME", dir.path().to_string_lossy().to_string()) };

        TestEnvironment {
            _directory: dir,
            original_home,
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_define() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        run(
            WorkspaceCommand::Define {
                name: "test-ws".into(),
                repos: vec!["org/repo1".into(), "org/repo2".into()],
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_define_json() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            WorkspaceCommand::Define {
                name: "json-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_define_empty_repos() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        run(
            WorkspaceCommand::Define {
                name: "empty-ws".into(),
                repos: vec![],
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_define_invalid_repo() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        let result = run(
            WorkspaceCommand::Define {
                name: "bad-ws".into(),
                repos: vec!["invalid".into()],
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_list_empty() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        run(WorkspaceCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_list_json_empty() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json();

        run(WorkspaceCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_archive_dry_run() {
        let _dir = setup_test_env();

        let define_app = test_helpers::make_app();

        run(
            WorkspaceCommand::Define {
                name: "archive-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &define_app,
        )
        .await
        .unwrap();

        let archive_app = test_helpers::make_app_dry_run();

        run(
            WorkspaceCommand::Archive {
                name: "archive-ws".into(),
            },
            &archive_app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_archive_skips_mismatched_provider() {
        let _dir = setup_test_env();

        let define_app = test_helpers::make_app();

        run(
            WorkspaceCommand::Define {
                name: "mixed-ws".into(),
                repos: vec!["gitlab@gitlab.com:org/repo1".into()],
            },
            &define_app,
        )
        .await
        .unwrap();

        let archive_app = test_helpers::make_app_dry_run();

        let result = run(
            WorkspaceCommand::Archive {
                name: "mixed-ws".into(),
            },
            &archive_app,
        )
        .await;

        assert!(matches!(result, Err(GitfleetError::PartialFailure(_))));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_archive_json_requires_yes() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            WorkspaceCommand::Define {
                name: "confirm-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        let result = run(
            WorkspaceCommand::Archive {
                name: "confirm-ws".into(),
            },
            &app,
        )
        .await;

        assert!(result.unwrap_err().to_string().contains("--yes"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_archive_reports_partial_provider_failure() {
        let _dir = setup_test_env();
        // SAFETY: This test serializes process-environment mutation with `serial_test`.
        unsafe { std::env::set_var("GITFLEET_GITHUB_TOKEN", "test-token") };

        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/repos/org/repo1"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("PATCH"))
            .and(path("/repos/org/repo2"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;

        let provider = GitHubProvider::with_base_url(&server.uri());
        let registry = ProviderRegistry::with_provider(ProviderId::GitHub, Box::new(provider));
        let renderer = Renderer::new(OutputMode::Silent).with_yes(true);
        let app = App::new(registry, renderer, ProviderId::GitHub, false);

        run(
            WorkspaceCommand::Define {
                name: "partial-ws".into(),
                repos: vec!["org/repo1".into(), "org/repo2".into()],
            },
            &app,
        )
        .await
        .unwrap();

        let result = run(
            WorkspaceCommand::Archive {
                name: "partial-ws".into(),
            },
            &app,
        )
        .await;

        // SAFETY: This test serializes process-environment mutation with `serial_test`.
        unsafe { std::env::remove_var("GITFLEET_GITHUB_TOKEN") };

        assert!(matches!(result, Err(GitfleetError::PartialFailure(_))));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_list_with_workspaces() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        run(
            WorkspaceCommand::Define {
                name: "ws-a".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        run(WorkspaceCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_list_json_with_workspaces() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            WorkspaceCommand::Define {
                name: "ws-b".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        run(WorkspaceCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_remove() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_yes();

        run(
            WorkspaceCommand::Define {
                name: "rm-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        run(
            WorkspaceCommand::Remove {
                name: "rm-ws".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_remove_nonexistent() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        let result = run(
            WorkspaceCommand::Remove {
                name: "no-such-ws".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_remove_json() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json_yes();

        run(
            WorkspaceCommand::Define {
                name: "rm-json-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        run(
            WorkspaceCommand::Remove {
                name: "rm-json-ws".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }
}
