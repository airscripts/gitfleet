use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, NotFoundError};

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum AliasCommand {
    #[command(about = "Set a command alias.")]
    Set {
        name: String,
        expansion: String,
        #[arg(long)]
        force: bool,
    },

    #[command(about = "Get a command alias expansion.")]
    Get { name: String },

    #[command(about = "List all aliases.")]
    List,

    #[command(about = "Delete an alias.")]
    Delete {
        name: String,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: AliasCommand, app: &App) -> Result<(), GitfleetError> {
    match cmd {
        AliasCommand::Set {
            name,
            expansion,
            force,
        } => {
            gitfleet_core::config::set_alias(&name, &expansion, force)?;

            if app.renderer().is_json() {
                app.renderer().write_result(&serde_json::json!({
                    "alias": name,
                    "expansion": expansion,
                    "action": "set",
                }));
            } else {
                app.renderer()
                    .render_success_box("Alias set", &format!("{name} = {expansion}"));
            }

            Ok(())
        }

        AliasCommand::Get { name } => {
            match gitfleet_core::config::get_alias(&name) {
                Some(expansion) => {
                    if app.renderer().is_json() {
                        app.renderer().write_result(&serde_json::json!({
                            "alias": name,
                            "expansion": expansion,
                        }));
                    } else {
                        app.renderer()
                            .render_key_values(&[("Alias", name), ("Expansion", expansion)]);
                    }
                }

                None => {
                    return Err(GitfleetError::from(NotFoundError::new(format!(
                        "Alias '{name}' not found."
                    ))));
                }
            }

            Ok(())
        }

        AliasCommand::List => {
            let aliases = gitfleet_core::config::list_aliases()?;

            if app.renderer().is_json() {
                let rows: Vec<serde_json::Value> = aliases
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "NAME": a.name,
                            "EXPANSION": a.expansion,
                        })
                    })
                    .collect();

                app.renderer().write_result(&serde_json::Value::Array(rows));
            } else if aliases.is_empty() {
                app.renderer().write_value("No aliases configured.");
            } else {
                let rows: Vec<serde_json::Value> = aliases
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "NAME": a.name,
                            "EXPANSION": a.expansion,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No aliases configured."),
                    Some("Aliases"),
                    Some(&["NAME", "EXPANSION"]),
                );
            }

            Ok(())
        }

        AliasCommand::Delete { name, yes } => {
            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete alias '{name}'?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            gitfleet_core::config::delete_alias(&name)?;

            if app.renderer().is_json() {
                app.renderer().write_result(&serde_json::json!({
                    "alias": name,
                    "action": "deleted",
                }));
            } else {
                app.renderer().render_success_box("Alias deleted", &name);
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    fn setup_test_env() -> (tempfile::TempDir, Option<String>) {
        let dir = tempfile::tempdir().unwrap();

        let gitfleet_dir = dir.path().join(".config").join("gitfleet");
        std::fs::create_dir_all(&gitfleet_dir).unwrap();

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", dir.path().to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var("GITFLEET_PROFILE");
        (dir, original_home)
    }

    fn cleanup_test_env(_dir: tempfile::TempDir, original_home: Option<String>) {
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_set() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            AliasCommand::Set {
                name: "ll".into(),
                expansion: "repo list".into(),
                force: false,
            },
            &app,
        )
        .await
        .unwrap();

        assert_eq!(
            gitfleet_core::config::get_alias("ll"),
            Some("repo list".to_string())
        );

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_set_force_overwrite() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            AliasCommand::Set {
                name: "ll".into(),
                expansion: "repo list".into(),
                force: false,
            },
            &app,
        )
        .await
        .unwrap();

        run(
            AliasCommand::Set {
                name: "ll".into(),
                expansion: "repo view".into(),
                force: true,
            },
            &app,
        )
        .await
        .unwrap();

        assert_eq!(
            gitfleet_core::config::get_alias("ll"),
            Some("repo view".to_string())
        );

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_set_duplicate_without_force_errors() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            AliasCommand::Set {
                name: "ll".into(),
                expansion: "repo list".into(),
                force: false,
            },
            &app,
        )
        .await
        .unwrap();

        let result = run(
            AliasCommand::Set {
                name: "ll".into(),
                expansion: "repo view".into(),
                force: false,
            },
            &app,
        )
        .await;

        assert!(result.is_err());

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_set_json() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            AliasCommand::Set {
                name: "ll".into(),
                expansion: "repo list".into(),
                force: false,
            },
            &app,
        )
        .await
        .unwrap();

        assert_eq!(
            gitfleet_core::config::get_alias("ll"),
            Some("repo list".to_string())
        );

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_get_existing() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        gitfleet_core::config::set_alias("co", "checkout", false).unwrap();

        run(AliasCommand::Get { name: "co".into() }, &app)
            .await
            .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_get_nonexistent_errors() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        let result = run(
            AliasCommand::Get {
                name: "nope".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_get_json() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_json();

        gitfleet_core::config::set_alias("co", "checkout", false).unwrap();

        run(AliasCommand::Get { name: "co".into() }, &app)
            .await
            .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_list_empty() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(AliasCommand::List, &app).await.unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_list_with_entries() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        gitfleet_core::config::set_alias("ll", "repo list", false).unwrap();

        gitfleet_core::config::set_alias("co", "checkout", false).unwrap();

        run(AliasCommand::List, &app).await.unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_list_json() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_json();

        gitfleet_core::config::set_alias("ll", "repo list", false).unwrap();

        run(AliasCommand::List, &app).await.unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_list_json_empty() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_json();

        run(AliasCommand::List, &app).await.unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_delete_with_yes() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        gitfleet_core::config::set_alias("temp", "repo list", false).unwrap();

        run(
            AliasCommand::Delete {
                name: "temp".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();

        assert_eq!(gitfleet_core::config::get_alias("temp"), None);

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_delete_nonexistent_errors() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        let result = run(
            AliasCommand::Delete {
                name: "nonexistent".into(),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_delete_json() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_json();

        gitfleet_core::config::set_alias("temp", "repo list", false).unwrap();

        run(
            AliasCommand::Delete {
                name: "temp".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_set_human() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_human();

        run(
            AliasCommand::Set {
                name: "ll".into(),
                expansion: "repo list".into(),
                force: false,
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_list_human() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_human();

        gitfleet_core::config::set_alias("ll", "repo list", false).unwrap();

        run(AliasCommand::List, &app).await.unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_get_human() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_human();

        gitfleet_core::config::set_alias("co", "checkout", false).unwrap();

        run(AliasCommand::Get { name: "co".into() }, &app)
            .await
            .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_delete_human() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_human();

        gitfleet_core::config::set_alias("temp", "repo list", false).unwrap();

        run(
            AliasCommand::Delete {
                name: "temp".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_alias_full_lifecycle() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            AliasCommand::Set {
                name: "ll".into(),
                expansion: "repo list".into(),
                force: false,
            },
            &app,
        )
        .await
        .unwrap();

        run(AliasCommand::Get { name: "ll".into() }, &app)
            .await
            .unwrap();

        run(AliasCommand::List, &app).await.unwrap();

        run(
            AliasCommand::Delete {
                name: "ll".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();

        assert_eq!(gitfleet_core::config::get_alias("ll"), None);

        cleanup_test_env(dir, original_home);
    }
}
