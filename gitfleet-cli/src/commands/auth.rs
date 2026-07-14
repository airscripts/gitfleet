use std::path::Path;

use clap::{Subcommand, ValueEnum};
use gitfleet_core::errors::{ConfigError, GitfleetError, TokenRequiredError};
use gitfleet_core::provider::ProviderId;

use crate::app::App;

fn mask_token(token: &str) -> String {
    let characters: Vec<char> = token.chars().collect();

    if characters.len() <= 12 {
        return "*".repeat(characters.len());
    }

    let prefix: String = characters.iter().take(8).collect();
    let suffix: String = characters.iter().rev().take(4).rev().collect();

    format!("{prefix}...{suffix}")
}

fn shell_quote(value: &Path) -> String {
    let value = value.to_string_lossy();

    format!("'{}'", value.replace('\'', "'\\''"))
}

fn normalize_setup_git_host(
    host: Option<&str>,
    default_host: &str,
) -> Result<String, GitfleetError> {
    gitfleet_core::config::normalize_host(host.unwrap_or(default_host)).map_err(GitfleetError::from)
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ProviderArg {
    #[value(name = "github")]
    GitHub,

    #[value(name = "gitlab")]
    GitLab,
}

impl ProviderArg {
    fn id(&self) -> ProviderId {
        match self {
            Self::GitHub => ProviderId::GitHub,
            Self::GitLab => ProviderId::GitLab,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    #[command(about = "Authenticate with a provider token.")]
    Login {
        #[arg(long)]
        host: Option<String>,

        #[arg(long, value_enum)]
        provider: Option<ProviderArg>,

        #[arg(long, default_value = "default")]
        profile: String,
    },

    #[command(about = "Remove stored credentials.")]
    Logout {
        #[arg(long)]
        yes: bool,

        #[arg(long)]
        profile: Option<String>,
    },

    #[command(about = "Show authentication status.")]
    Status {
        #[arg(long)]
        show_token: bool,

        #[arg(long)]
        capabilities: bool,
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
            host,
            provider,
            profile,
        } => {
            let token = gitfleet_core::prompt::prompt_password("Enter provider token:")?;

            if token.trim().is_empty() {
                return Err(GitfleetError::from(TokenRequiredError::new(
                    gitfleet_core::constants::ERROR_AUTH_NO_TOKEN,
                    vec![],
                )));
            }

            let provider =
                provider
                    .map(|provider| provider.id())
                    .unwrap_or_else(|| match host.as_deref() {
                        Some(host) if host.contains("gitlab") => ProviderId::GitLab,
                        _ => ProviderId::GitHub,
                    });

            let host = host.unwrap_or_else(|| match provider {
                ProviderId::GitHub => "github.com".to_string(),
                ProviderId::GitLab => "gitlab.com".to_string(),
            });
            let host = gitfleet_core::config::normalize_host(&host)?;

            let provider_name = match provider {
                ProviderId::GitHub => "github",
                ProviderId::GitLab => "gitlab",
            };

            gitfleet_core::config::add_profile(
                &profile,
                gitfleet_core::types::Profile {
                    token: Some(token.trim().to_string()),
                    host: Some(host),
                    provider: Some(provider_name.to_string()),
                    extra: Default::default(),
                },
            )?;

            app.renderer()
                .render_success_box("Logged in", "Authentication successful.");

            Ok(())
        }

        AuthCommand::Logout { yes, profile } => {
            let target = profile
                .as_deref()
                .map(|name| format!("profile '{name}'"))
                .unwrap_or_else(|| "all stored credentials".to_string());

            gitfleet_core::prompt::confirm_destructive(
                &format!("Remove {target}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            if let Some(profile) = profile {
                gitfleet_core::config::remove_profile(&profile)?;
            } else {
                gitfleet_core::config::clear_credentials()?;
            }

            app.renderer()
                .render_success_box("Credentials removed", &target);

            Ok(())
        }

        AuthCommand::Status {
            show_token,
            capabilities,
        } => {
            if capabilities {
                let capability_names: Vec<String> =
                    app.capabilities().iter().map(ToString::to_string).collect();

                let status = serde_json::json!({
                    "profile": app.profile_name(),
                    "provider": app.provider_id().to_string(),
                    "host": app.provider_host(),
                    "capabilities": capability_names,
                });

                if app.renderer().is_json() {
                    app.renderer().write_result(&status);
                } else {
                    let rows: Vec<serde_json::Value> = status["capabilities"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|capability| {
                            serde_json::json!({
                                "CAPABILITY": capability,
                            })
                        })
                        .collect();

                    app.renderer().render_summary(
                        "Active Provider",
                        &[
                            ("Profile", app.profile_name().to_string()),
                            ("Provider", app.provider_id().to_string()),
                            ("Host", app.provider_host().to_string()),
                        ],
                    );
                    app.renderer().render_table_titled(
                        &rows,
                        Some("No capabilities reported."),
                        Some("Capabilities"),
                        Some(&["CAPABILITY"]),
                    );
                }

                return Ok(());
            }

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
                                let t = gitfleet_core::config::get_profile(&p.name)
                                    .ok()
                                    .flatten()
                                    .and_then(|profile| profile.token)
                                    .unwrap_or_default();

                                mask_token(&t)
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
            } else {
                app.renderer().write_value(&mask_token(&token));
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
            let remote_url = gitfleet_core::git::get_remote_url(None)?;
            let host = gitfleet_core::git::get_remote_host(&remote_url)?;

            let profile = gitfleet_core::config::find_profile_by_host(&host)?.ok_or_else(|| {
                ConfigError::new(format!("No profile configured for host \"{host}\"."))
            })?;

            gitfleet_core::config::set_active_profile(&profile)?;

            app.renderer()
                .write_value(&format!("Switched to profile '{profile}' for {host}."));

            Ok(())
        }

        AuthCommand::SetupGit { host } => {
            let host_str = normalize_setup_git_host(host.as_deref(), app.provider_host())?;

            let token = gitfleet_core::config::get_token_for_host(&host_str);

            if token.is_none() {
                return Err(GitfleetError::from(TokenRequiredError::new(
                    "No token found. Run `gitfleet auth login` first.",
                    vec![],
                )));
            }

            let exe = std::env::current_exe().map_err(|e| {
                GitfleetError::new(format!("Failed to determine gitfleet executable path: {e}"))
            })?;

            let helper = format!("!{} git-credential", shell_quote(&exe));

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

#[cfg(test)]
mod tests {
    use super::{mask_token, normalize_setup_git_host, shell_quote};

    #[test]
    fn mask_token_preserves_ascii_prefix_and_suffix() {
        assert_eq!(mask_token("abcdefghijklmnop"), "abcdefgh...mnop");
    }

    #[test]
    fn mask_token_handles_multibyte_characters() {
        let masked = mask_token("токен-с-юникодом");

        assert!(masked.starts_with("токен-с-") || masked.starts_with("токен-с"));
        assert!(masked.ends_with("одом"));
        assert!(masked.contains("..."));
    }

    #[test]
    fn mask_token_redacts_short_tokens_completely() {
        assert_eq!(mask_token("short"), "*****");
        assert_eq!(mask_token(""), "");
    }

    #[test]
    fn shell_quote_protects_paths_for_git_helpers() {
        let quoted = shell_quote(std::path::Path::new("/tmp/git fleet/it's"));

        assert_eq!(quoted, "'/tmp/git fleet/it'\\''s'");
    }

    #[test]
    fn normalize_setup_git_host_canonicalizes_host() {
        assert_eq!(
            normalize_setup_git_host(Some(" GitHub.COM/"), "gitlab.com").unwrap(),
            "github.com"
        );
    }
}
