# Output

Gitfleet defaults to human-readable output:

```bash
gitfleet issue list --repo owner/repository
```

Use `--json` for scripts and automation:

```bash
gitfleet issue list --repo owner/repository --json
```

Structured command output writes to stdout. Status, diagnostics, and tracing
write to stderr.

## Global Options

| Option | Use |
| ------ | --- |
| `--json` | Return machine-readable output. |
| `--debug` | Enable redacted diagnostic logs. |
| `--theme dark\|light\|auto` | Select terminal colors. |
| `--yes` | Confirm destructive operations in automation or JSON mode. |
| `--dry-run` | Preview supported bulk mutations. |

Human output is for reading. JSON output is the contract to parse.

## Practical Guidance

Use human output while exploring commands because tables and success boxes are
optimized for scanning. Switch to `--json` when storing results, passing data to
another command, or making assertions in CI.

Do not parse progress text, debug logs, or human tables. They may change as the
user experience improves. JSON fields are the intended automation surface for
scripts.
