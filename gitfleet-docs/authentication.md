# Authentication

Use `auth` to create and manage provider profiles.

```bash
gitfleet auth login
gitfleet auth status
gitfleet auth token
gitfleet auth list
gitfleet auth switch work
gitfleet auth detect
```

## Token Storage

Profile metadata is stored at `~/.config/gitfleet/credentials.toml` with mode
`0600`. Tokens use the operating-system credential store by default. If no
native credential store is available, Gitfleet reports an error rather than
writing plaintext credentials.

Users who explicitly accept plaintext storage can opt in:

```bash
export GITFLEET_CREDENTIAL_STORE=file
gitfleet auth login
```

Plaintext storage is permission-protected but not encrypted. Use it only on a
trusted machine, and prefer the operating-system credential store whenever it is
available.

## Profile Lifecycle

Create profiles for long-lived identities and switch between them explicitly:

```bash
gitfleet auth login --profile personal
gitfleet auth login --profile work
gitfleet auth list
gitfleet auth switch work
```

`auth detect` is useful inside an existing clone because it can select the
profile that matches the current remote. `auth logout --profile NAME` removes
only one profile; omitting `--profile` removes all stored credentials after
confirmation.

## Automation

For headless systems and CI, set a provider token through the environment:

```bash
export GITFLEET_GITHUB_TOKEN=github_pat_...
export GITFLEET_GITLAB_TOKEN=glpat-...
```

Use `GITFLEET_PROFILE` when automation needs a named profile. Destructive JSON
or non-interactive operations require `--yes`.

Keep CI tokens scoped to the commands the job runs. A read-only inventory job
should not receive delete permissions, and a release job should not receive
organization-admin permissions unless it also manages access.

## Token Scope Guidance

GitHub classic tokens commonly need `repo` for private repositories,
`workflow` for workflow file changes, `notifications` for inbox operations,
`read:org` or `admin:org` for organization access, `gist` for snippets,
package scopes for registry operations, `security_events` for security alerts,
`codespace` for development environments, and `delete_repo` for repository
deletion.

For GitLab, `api` gives the complete Gitfleet command surface. Narrower tokens
such as `read_api`, `read_user`, `read_repository`, `write_repository`,
`create_runner`, and `manage_runner` can be used for smaller workflows.

Do not grant GitLab `sudo` or `admin_mode` for normal Gitfleet use.
