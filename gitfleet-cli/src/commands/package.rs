use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnprocessableError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum PackageCommand {
    #[command(about = "List packages.")]
    List {
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        package_type: Option<String>,
        #[arg(long, default_value = "10")]
        limit: u32,
    },

    #[command(about = "View a package.")]
    View {
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        package_type: String,
        #[arg(long)]
        package_name: String,
    },
}

pub async fn run(cmd: PackageCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.registry_ops().ok_or_else(|| {
        GitfleetError::UnsupportedCapability(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::Registry,
        ))
    })?;

    match cmd {
        PackageCommand::List {
            owner,
            package_type,
            limit,
        } => {
            let owner_str = owner.as_deref().ok_or_else(|| {
                GitfleetError::from(UnprocessableError::new(
                    "Package owner is required. Use --owner OWNER.",
                ))
            })?;

            let data = ops
                .list_packages(owner_str, package_type.as_deref(), limit)
                .await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&data).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize packages: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = data
                    .iter()
                    .map(|pkg| {
                        serde_json::json!({
                            "ID": pkg.id,
                            "NAME": pkg.name,
                            "TYPE": pkg.package_type,
                            "VISIBILITY": pkg.visibility,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No packages found."),
                    Some("Packages"),
                    None,
                );
            }

            Ok(())
        }

        PackageCommand::View {
            owner,
            package_type,
            package_name,
        } => {
            let owner_str = owner.as_deref().ok_or_else(|| {
                GitfleetError::from(UnprocessableError::new(
                    "Package owner is required. Use --owner OWNER.",
                ))
            })?;

            let data = ops
                .get_package(owner_str, &package_type, &package_name)
                .await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer().render_success_box("Package", &package_name);
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
    async fn test_pkg_list() {
        let app = test_helpers::make_app();

        let result = run(
            PackageCommand::List {
                owner: None,
                package_type: None,
                limit: 10,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pkg_list_with_owner_and_type() {
        let app = test_helpers::make_app();

        run(
            PackageCommand::List {
                owner: Some("org".into()),
                package_type: Some("npm".into()),
                limit: 5,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pkg_list_json() {
        let app = test_helpers::make_app_json();

        let result = run(
            PackageCommand::List {
                owner: None,
                package_type: None,
                limit: 10,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pkg_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            PackageCommand::List {
                owner: None,
                package_type: None,
                limit: 10,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pkg_view() {
        let app = test_helpers::make_app();

        let result = run(
            PackageCommand::View {
                owner: None,
                package_type: "npm".into(),
                package_name: "pkg1".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pkg_view_with_owner() {
        let app = test_helpers::make_app();

        run(
            PackageCommand::View {
                owner: Some("org".into()),
                package_type: "npm".into(),
                package_name: "pkg1".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_pkg_view_json() {
        let app = test_helpers::make_app_json();

        let result = run(
            PackageCommand::View {
                owner: None,
                package_type: "npm".into(),
                package_name: "pkg1".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pkg_view_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            PackageCommand::View {
                owner: None,
                package_type: "npm".into(),
                package_name: "pkg1".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
