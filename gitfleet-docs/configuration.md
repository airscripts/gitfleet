# Configuration

Gitfleet configuration lives under `~/.config/gitfleet/`. The format is TOML,
and environment variables use the `GITFLEET_` prefix.

## Profile Resolution

When `GITFLEET_PROFILE` is set, Gitfleet selects that named profile. An unknown
profile is an error.

Otherwise, resolution checks:

1. A trusted repository-local profile when `GITFLEET_TRUST_REPO_CONFIG=true`.
2. The active profile.
3. The first configured profile in sorted order.
4. The default profile.

Within the selected profile, a stored profile token takes precedence over the
provider environment token. Environment tokens are used for the default provider
host when the profile has no stored token.

This order lets interactive users keep a local active profile while automation
can still select a profile explicitly. If a command appears to use the wrong
account, check `GITFLEET_PROFILE`, then `gitfleet auth status`, then any trusted
repository-local configuration.

## Environment Variables

| Key | Purpose |
| --- | ------- |
| `GITFLEET_GITHUB_TOKEN` | Supplies a GitHub token for automation. |
| `GITFLEET_GITLAB_TOKEN` | Supplies a GitLab token for automation. |
| `GITFLEET_PROFILE` | Selects a named profile. |
| `GITFLEET_TRUST_REPO_CONFIG` | Allows `.gitfleetrc` to select a profile when set to `true`. |
| `GITFLEET_CREDENTIAL_STORE` | Uses plaintext file credential storage when set to `file`. |
| `GITFLEET_HOME` | Overrides the home directory used for `.config/gitfleet/`. |
| `GITFLEET_CI` | Enables non-interactive behavior when present. |
| `GITFLEET_LOG` | Overrides the tracing filter. |
| `GITFLEET_NO_COLOR` | Disables colored output when present. |
| `GITFLEET_TERM` | Contributes to terminal theme detection. |
| `GITFLEET_COLORTERM` | Enables true-color-aware theme detection. |
| `GITFLEET_COLORFGBG` | Supplies terminal foreground/background hints. |

Live playbook variables such as `GITFLEET_PLAYBOOK_REPO` are developer
validation settings, not normal user configuration.

Copy [../.env.example](../.env.example) when preparing shell or CI variables.
Gitfleet does not automatically load dotenv files.

## Repository-local Configuration

Repository-local profile selection is disabled by default. Enable it only for
repositories you trust:

```bash
export GITFLEET_TRUST_REPO_CONFIG=true
```

This protects users from a cloned repository silently selecting a different
profile. Prefer explicit `GITFLEET_PROFILE` in CI and shared scripts.
