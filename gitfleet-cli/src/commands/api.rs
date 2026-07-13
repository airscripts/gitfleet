use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnprocessableError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum ApiCommand {
    #[command(about = "Send a raw GET request to the provider API.")]
    Get {
        #[arg(long)]
        endpoint: String,
    },

    #[command(about = "Send a raw POST request to the provider API.")]
    Post {
        #[arg(long)]
        endpoint: String,
        #[arg(long)]
        body: String,
        #[arg(long)]
        yes: bool,
    },

    #[command(about = "Send a raw DELETE request to the provider API.")]
    Delete {
        #[arg(long)]
        endpoint: String,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: ApiCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.raw_api_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            p.id(),
            ProviderCapability::RawApi,
        ))
    })?;

    match cmd {
        ApiCommand::Get { endpoint } => {
            validate_endpoint(&endpoint)?;

            let data = ops.raw_get(&endpoint).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer()
                    .render_success_box("Response", &format!("{data}"));
            }

            Ok(())
        }

        ApiCommand::Post {
            endpoint,
            body,
            yes,
        } => {
            validate_endpoint(&endpoint)?;

            let parsed: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
                GitfleetError::from(UnprocessableError::new(format!("Invalid JSON body: {e}")))
            })?;

            if app.dry_run() {
                let preview = serde_json::json!({
                    "dry_run": true,
                    "action": "post",
                    "endpoint": endpoint,
                    "body": parsed,
                });

                if app.renderer().is_json() {
                    app.renderer().write_result(&preview);
                } else {
                    app.renderer()
                        .render_box(&format!("Would send {preview}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Send raw POST request to {endpoint}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let data = ops.raw_post(&endpoint, parsed).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer()
                    .render_success_box("Response", &format!("{data}"));
            }

            Ok(())
        }

        ApiCommand::Delete { endpoint, yes } => {
            validate_endpoint(&endpoint)?;

            if app.dry_run() {
                let preview = serde_json::json!({
                    "dry_run": true,
                    "action": "delete",
                    "endpoint": endpoint,
                });

                if app.renderer().is_json() {
                    app.renderer().write_result(&preview);
                } else {
                    app.renderer()
                        .render_box(&format!("Would send {preview}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Send raw DELETE request to {endpoint}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let data = ops.raw_delete(&endpoint).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer()
                    .render_success_box("Response", &format!("{data}"));
            }

            Ok(())
        }
    }
}

fn validate_endpoint(endpoint: &str) -> Result<(), GitfleetError> {
    if !endpoint.starts_with('/')
        || endpoint.starts_with("//")
        || endpoint.contains("://")
        || endpoint.contains('#')
        || endpoint.chars().any(char::is_control)
        || endpoint
            .split('?')
            .next()
            .unwrap_or(endpoint)
            .split('/')
            .any(|segment| {
                segment == "." || segment == ".." || segment.to_ascii_lowercase().contains("%2e")
            })
    {
        return Err(GitfleetError::from(UnprocessableError::new(
            "API endpoint must be a relative provider path beginning with '/'.",
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_api_get() {
        let app = test_helpers::make_app();

        run(
            ApiCommand::Get {
                endpoint: "/repos/org/repo".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_api_get_json() {
        let app = test_helpers::make_app_json();

        run(
            ApiCommand::Get {
                endpoint: "/repos/org/repo".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_api_get_human() {
        let app = test_helpers::make_app_human();

        run(
            ApiCommand::Get {
                endpoint: "/repos/org/repo".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_api_get_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ApiCommand::Get {
                endpoint: "/repos/org/repo".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_post() {
        let app = test_helpers::make_app();

        run(
            ApiCommand::Post {
                endpoint: "/repos/org/repo".into(),
                body: r#"{"key":"value"}"#.into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_api_post_json() {
        let app = test_helpers::make_app_json();

        run(
            ApiCommand::Post {
                endpoint: "/repos/org/repo".into(),
                body: r#"{"key":"value"}"#.into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_api_post_human() {
        let app = test_helpers::make_app_human();

        run(
            ApiCommand::Post {
                endpoint: "/repos/org/repo".into(),
                body: r#"{"key":"value"}"#.into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_api_post_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ApiCommand::Post {
                endpoint: "/repos/org/repo".into(),
                body: r#"{"key":"value"}"#.into(),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_post_invalid_json() {
        let app = test_helpers::make_app();

        let result = run(
            ApiCommand::Post {
                endpoint: "/repos/org/repo".into(),
                body: "not json".into(),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_delete() {
        let app = test_helpers::make_app();

        run(
            ApiCommand::Delete {
                endpoint: "/repos/org/repo".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_api_delete_json() {
        let app = test_helpers::make_app_json();

        run(
            ApiCommand::Delete {
                endpoint: "/repos/org/repo".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_api_delete_human() {
        let app = test_helpers::make_app_human();

        run(
            ApiCommand::Delete {
                endpoint: "/repos/org/repo".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_api_delete_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ApiCommand::Delete {
                endpoint: "/repos/org/repo".into(),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_delete_requires_confirmation() {
        let app = test_helpers::make_app();

        let result = run(
            ApiCommand::Delete {
                endpoint: "/repos/org/repo".into(),
                yes: false,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_rejects_absolute_endpoint() {
        let app = test_helpers::make_app();

        let result = run(
            ApiCommand::Get {
                endpoint: "https://example.com".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_endpoint_rejects_path_traversal() {
        assert!(validate_endpoint("/../admin").is_err());
        assert!(validate_endpoint("/api/%2e%2e/admin").is_err());
    }

    #[test]
    fn test_validate_endpoint_rejects_fragments() {
        assert!(validate_endpoint("/repos/org/repo#details").is_err());
    }
}
