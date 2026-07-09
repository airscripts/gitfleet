use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, TokenRequiredError};

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    #[command(about = "Authenticate with a provider token.")]
    Login {
        #[arg(long)]
        token: Option<String>,
        #[arg(long, default_value = "github.com")]
        host: String,
        #[arg(long, default_value = "default")]
        profile: String,
    },

    #[command(about = "Remove stored credentials.")]
    Logout {
        #[arg(long)]
        yes: bool,
    },

    #[command(about = "Show authentication status.")]
    Status {
        #[arg(long)]
        show_token: bool,
    },

    #[command(about = "Print the current token.")]
    Token {
        #[arg(long)]
        raw: bool,
    },

    #[command(about = "List all configured profiles.")]
    List,

    #[command(about = "Switch the active profile.")]
    Switch { name: Option<String> },

    #[command(about = "Detect the profile for the current repository.")]
    Detect,

    #[command(about = "Configure git to use gitfleet as credential helper.")]
    SetupGit {
        #[arg(long)]
        host: Option<String>,
    },
}

pub async fn run(cmd: AuthCommand, app: &App) -> Result<(), GitfleetError> {
    match cmd {
        AuthCommand::Login {
            token,
            host,
            profile,
        } => {
            let token = match token {
                Some(t) => t,
                None => gitfleet_core::prompt::prompt_text("Enter provider token:")?,
            };

            if token.trim().is_empty() {
                return Err(GitfleetError::from(TokenRequiredError::new(
                    gitfleet_core::constants::ERROR_AUTH_NO_TOKEN,
                    vec![],
                )));
            }

            let provider = if host.contains("gitlab") {
                "gitlab"
            } else {
                "github"
            };

            gitfleet_core::config::add_profile(
                &profile,
                gitfleet_core::types::Profile {
                    token: Some(token.trim().to_string()),
                    host: Some(host),
                    provider: Some(provider.to_string()),
                    extra: Default::default(),
                },
            )?;

            app.renderer()
                .render_success_box("Logged in", "Authentication successful.");

            Ok(())
        }

        AuthCommand::Logout { yes } => {
            gitfleet_core::prompt::confirm_destructive(
                "Remove stored credentials?",
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            gitfleet_core::config::clear_credentials()?;

            app.renderer().write_value("Logged out successfully.");

            Ok(())
        }

        AuthCommand::Status { show_token } => {
            let profiles = gitfleet_core::config::list_profiles()?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&profiles).unwrap_or_default();

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = profiles
                    .iter()
                    .map(|p| {
                        if show_token {
                            let token_display = if p.has_token {
                                let t = gitfleet_core::config::read("token").unwrap_or_default();

                                if t.len() > 12 {
                                    format!("{}...{}", &t[..8], &t[t.len().saturating_sub(4)..])
                                } else {
                                    t
                                }
                            } else {
                                "-".to_string()
                            };

                            serde_json::json!({
                                "NAME": p.name,
                                "ACTIVE": p.active,
                                "HAS TOKEN": p.has_token,
                                "TOKEN": token_display,
                            })
                        } else {
                            serde_json::json!({
                                "NAME": p.name,
                                "ACTIVE": p.active,
                                "HAS TOKEN": p.has_token,
                            })
                        }
                    })
                    .collect();

                let columns = if show_token {
                    Some(&["NAME", "ACTIVE", "HAS TOKEN", "TOKEN"][..])
                } else {
                    Some(&["NAME", "ACTIVE", "HAS TOKEN"][..])
                };

                app.renderer().render_table_titled(
                    &rows,
                    Some("No profiles found."),
                    Some("Profiles"),
                    columns,
                );
            }

            Ok(())
        }

        AuthCommand::Token { raw } => {
            let token = gitfleet_core::config::get_token()?;

            if raw {
                app.renderer().write_value(&token);
            } else if token.len() > 12 {
                let masked = format!(
                    "{}...{}",
                    &token[..8],
                    &token[token.len().saturating_sub(4)..]
                );

                app.renderer().write_value(&masked);
            } else {
                app.renderer().write_value(&token);
            }

            Ok(())
        }

        AuthCommand::List => {
            let profiles = gitfleet_core::config::list_profiles()?;

            if app.renderer().is_json() {
                app.renderer()
                    .write_result(&serde_json::to_value(&profiles).unwrap_or_default());
            } else {
                let rows: Vec<serde_json::Value> = profiles
                    .iter()
                    .map(|p| {
                        serde_json::json!({
                            "NAME": p.name,
                            "ACTIVE": p.active,
                            "HAS TOKEN": p.has_token,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No profiles found."),
                    Some("Profiles"),
                    Some(&["NAME", "ACTIVE", "HAS TOKEN"]),
                );
            }

            Ok(())
        }

        AuthCommand::Switch { name } => {
            let profile_name = match name {
                Some(n) => n,
                None => gitfleet_core::prompt::prompt_text("Profile name?")?,
            };

            gitfleet_core::config::set_active_profile(&profile_name)?;

            app.renderer()
                .write_value(&format!("Switched to profile '{profile_name}'."));

            Ok(())
        }

        AuthCommand::Detect => {
            match gitfleet_core::git::get_repo_root() {
                Ok(root) => app
                    .renderer()
                    .write_value(&format!("Repository root: {}", root.display())),
                Err(_) => app.renderer().write_value("Not inside a git repository."),
            }

            Ok(())
        }

        AuthCommand::SetupGit { host } => {
            let host_str = host.unwrap_or_else(gitfleet_core::config::get_host);

            let token = gitfleet_core::config::get_token_optional();

            if token.is_none() {
                return Err(GitfleetError::from(TokenRequiredError::new(
                    "No token found. Run `gitfleet auth login` first.",
                    vec![],
                )));
            }

            let exe = std::env::current_exe().map_err(|e| {
                GitfleetError::new(format!("Failed to determine gitfleet executable path: {e}"))
            })?;

            let helper = format!("!{} git-credential", exe.display());

            let scope = format!("credential.https://{host_str}.helper");

            let status = std::process::Command::new("git")
                .args(["config", "--global", &scope, &helper])
                .status()
                .map_err(|e| GitfleetError::new(format!("Failed to run git config: {e}")))?;

            if !status.success() {
                return Err(GitfleetError::new(
                    "Failed to configure git credential helper.",
                ));
            }

            app.renderer().render_success_box(
                "Git configured",
                &format!("Credential helper set for {host_str}."),
            );

            Ok(())
        }
    }
}
