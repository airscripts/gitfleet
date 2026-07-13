use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    #[command(about = "Set a configuration value.")]
    Set { key: String, value: String },

    #[command(about = "Get a configuration value.")]
    Get { key: String },

    #[command(about = "Unset a configuration value.")]
    Unset { key: String },
}

pub async fn run(cmd: ConfigCommand, app: &App) -> Result<(), GitfleetError> {
    match cmd {
        ConfigCommand::Set { key, value } => {
            gitfleet_core::config::write(&key, &value)?;

            let display_value = if is_sensitive_key(&key) {
                "[redacted]"
            } else {
                &value
            };

            app.renderer()
                .write_value(&format!("Set {key} = {display_value}"));

            Ok(())
        }

        ConfigCommand::Get { key } => {
            match gitfleet_core::config::read(&key) {
                Some(_value) if is_sensitive_key(&key) => app.renderer().write_value("[redacted]"),
                Some(value) => app.renderer().write_value(&value),
                None => app.renderer().write_value(&format!("{key} is not set")),
            }

            Ok(())
        }

        ConfigCommand::Unset { key } => {
            gitfleet_core::config::unset(&key)?;

            app.renderer().write_value(&format!("Unset {key}"));

            Ok(())
        }
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();

    key == "token"
        || key.contains("api_key")
        || key.contains("access_key")
        || key.contains("private_key")
        || key.contains("bearer")
        || key.contains("auth")
        || key.contains("password")
        || key.contains("secret")
        || key.contains("credential")
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
        std::env::remove_var("GITFLEET_CREDENTIAL_STORE");
        std::env::set_var("GITFLEET_TEST_CREDENTIAL_STORE", "1");
        (dir, original_home)
    }

    fn cleanup_test_env(_dir: tempfile::TempDir, original_home: Option<String>) {
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }
        std::env::remove_var("GITFLEET_CREDENTIAL_STORE");
        std::env::remove_var("GITFLEET_TEST_CREDENTIAL_STORE");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_set_token() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            ConfigCommand::Set {
                key: "token".into(),
                value: "ghp_abc123".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_set_host() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            ConfigCommand::Set {
                key: "host".into(),
                value: "github.com".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_set_json() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            ConfigCommand::Set {
                key: "token".into(),
                value: "ghp_json".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_get_unset_key() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            ConfigCommand::Get {
                key: "token".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_get_set_key() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            ConfigCommand::Set {
                key: "token".into(),
                value: "ghp_get".into(),
            },
            &app,
        )
        .await
        .unwrap();

        run(
            ConfigCommand::Get {
                key: "token".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_get_json() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            ConfigCommand::Get {
                key: "token".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_unset_token() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            ConfigCommand::Set {
                key: "token".into(),
                value: "ghp_tmp".into(),
            },
            &app,
        )
        .await
        .unwrap();

        run(
            ConfigCommand::Unset {
                key: "token".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_unset_host() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            ConfigCommand::Set {
                key: "host".into(),
                value: "gitlab.com".into(),
            },
            &app,
        )
        .await
        .unwrap();

        run(ConfigCommand::Unset { key: "host".into() }, &app)
            .await
            .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_unset_json() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            ConfigCommand::Set {
                key: "token".into(),
                value: "ghp_json_unset".into(),
            },
            &app,
        )
        .await
        .unwrap();

        run(
            ConfigCommand::Unset {
                key: "token".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_set_arbitrary_key() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            ConfigCommand::Set {
                key: "default_org".into(),
                value: "myorg".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_get_arbitrary_key() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            ConfigCommand::Set {
                key: "default_org".into(),
                value: "myorg".into(),
            },
            &app,
        )
        .await
        .unwrap();

        run(
            ConfigCommand::Get {
                key: "default_org".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_unset_arbitrary_key() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        run(
            ConfigCommand::Set {
                key: "default_org".into(),
                value: "myorg".into(),
            },
            &app,
        )
        .await
        .unwrap();

        run(
            ConfigCommand::Unset {
                key: "default_org".into(),
            },
            &app,
        )
        .await
        .unwrap();

        cleanup_test_env(dir, original_home);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_config_unset_nonexistent_key() {
        let (dir, original_home) = setup_test_env();

        let app = test_helpers::make_app();

        let result = run(
            ConfigCommand::Unset {
                key: "nonexistent".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());

        cleanup_test_env(dir, original_home);
    }
}
