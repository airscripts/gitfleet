# Automation and JSON

Use `--json` for machine-readable results and `--yes` for destructive
operations in non-interactive contexts.

Automation should be explicit about profile, repository, output mode, and
confirmation. Avoid relying on the active local profile or current working
directory unless the script is intentionally repository-local.

```bash
GITFLEET_GITHUB_TOKEN=github_pat_... gitfleet repo list --json
GITFLEET_PROFILE=work gitfleet issue list --repo owner/repository --json
gitfleet repo clone --all --org platform --directory repos --dry-run --json
gitfleet repo delete owner/old-repository --json --yes
```

Status and debug logs go to stderr. Structured output goes to stdout.

Use `--dry-run` before supported bulk mutations:

```bash
gitfleet workspace archive platform --dry-run --json
```

Use `--dry-run` before bulk local operations too:

```bash
gitfleet repo clone --all --user alice --directory user-repos --dry-run --json
```

Parse stdout for JSON results and leave stderr for progress or debug logs. If a
job needs diagnostics, add `--debug` and keep logs redacted before sharing them.
