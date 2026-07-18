# Safety

Gitfleet treats destructive operations as explicit actions.

## Confirmation

Destructive operations in human mode prompt for confirmation. In JSON mode,
CI, or other non-interactive contexts, destructive operations require `--yes`.

Examples:

```bash
gitfleet repo delete owner/repository --yes
gitfleet release delete v1.0.0 --repo owner/repository --yes
gitfleet wiki delete usage --repo group/project --yes
```

## Dry Runs

Bulk mutations provide `--dry-run` when a meaningful preview exists:

```bash
gitfleet workspace archive platform --dry-run
```

Dry runs should be used before broad fleet changes.

## Review Order

For risky operations, use this order:

1. Run the nearest read-only command to inspect current state.
2. Run the mutating command with `--dry-run` when supported.
3. Run the command interactively and answer the confirmation prompt.
4. Add `--yes` only when the exact command is ready for automation.

This keeps the command history readable and makes reviews easier before
destructive changes.

## Unsupported Capabilities

If a provider does not support an operation, Gitfleet returns an unsupported
capability error. It does not emulate missing provider behavior through hidden
fallbacks.
