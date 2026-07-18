# First Run

Install Gitfleet, authenticate, inspect the active profile, and run a read-only
repository command.

This workflow is intentionally conservative. It starts with authentication and
read commands so you can confirm the selected account, provider, host, and
repository context before making changes.

```bash
cargo install --path gitfleet

gitfleet auth login
gitfleet auth status
gitfleet repo list
gitfleet issue list --repo owner/repository
```

Use `gitfleet help` and nested help when exploring:

```bash
gitfleet help
gitfleet help repo
gitfleet help change create
```

Start with read commands until the profile, provider, and repository target are
clear. Destructive commands prompt in human mode and require `--yes` in
automation.

Next, read [Authentication](../authentication.md) if you need multiple accounts
or enterprise hosts, and [Command Reference](../commands/README.md) when you are
ready to choose a command family.
