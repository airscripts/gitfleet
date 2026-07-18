# Multi-provider Profiles

Use profiles to keep provider, host, and account identity separate.

Profiles are the safest way to work across GitHub, GitHub Enterprise, GitLab.com,
and self-managed GitLab without changing command names. Each profile should map
to one account on one host.

```bash
gitfleet auth login --provider github --profile personal
gitfleet auth login --provider github --host github.example.com --profile work
gitfleet auth login --provider gitlab --host git.example.com --profile work-gitlab
gitfleet auth list
gitfleet auth switch work
```

Inside a repository, use detection to choose the matching profile from the
remote:

```bash
gitfleet auth detect
gitfleet auth status
```

Automation can select a profile without mutating active local state:

```bash
GITFLEET_PROFILE=work gitfleet repo list --json
```

For scripts, always set `GITFLEET_PROFILE` explicitly. That keeps the script
independent from whichever profile a developer last selected on their machine.
