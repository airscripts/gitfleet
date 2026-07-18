# Troubleshooting

## Command Not Found

Cargo installs binaries into `~/.cargo/bin`. Make sure that directory is on
`PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Refresh a local install after pulling source changes:

```bash
cargo install --path gitfleet --force
```

## Wrong Provider or Account

Check the active profile:

```bash
gitfleet auth status
gitfleet auth list
```

Switch profiles explicitly:

```bash
gitfleet auth switch work
```

For repository-local detection, run:

```bash
gitfleet auth detect
```

## Unsupported Capability

The selected provider does not support the requested command family or nested
operation. Confirm provider support with `gitfleet auth status` and see
[Providers](./providers.md).

Common examples are GitHub wiki operations, GitLab discussions, GitLab
repository secrets, and GitHub protected-tag operations. Those are provider
capability differences, not local installation problems.

## Authentication Fails

Confirm the token has the required provider permissions, the selected host is
correct, and automation has exported the expected `GITFLEET_*_TOKEN` variable.
Use `--debug` for redacted diagnostics.

## JSON Automation Fails on Destructive Commands

Add `--yes` to destructive operations when running with `--json`, in CI, or in a
non-interactive shell.

## Repository Could Not Be Resolved

Pass `--repo owner/repository` explicitly, or run the command from a Git clone
whose remote points at the intended provider. In scripts, prefer explicit
`--repo` values because they do not depend on the current working directory.
