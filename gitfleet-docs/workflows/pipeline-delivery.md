# Pipeline Delivery

Use `pipeline` to inspect definitions and runs, then pair it with `release`,
`deploy`, `environment`, `runner`, `registry`, and `webhook` as needed.

Start with inspection commands. They tell you which workflow definition, run ID,
ref, environment, or artifact name to use before triggering or changing
delivery state.

```bash
gitfleet pipeline list-def --repo owner/repository
gitfleet pipeline view-def <workflow-id> --repo owner/repository
gitfleet pipeline list-runs --repo owner/repository
gitfleet pipeline view-run <run-id> --repo owner/repository
gitfleet pipeline trigger <workflow-id> --repo owner/repository --ref main
gitfleet release list --repo owner/repository
gitfleet deploy list --repo owner/repository
```

Cancel and rerun are active run operations:

```bash
gitfleet pipeline cancel <run-id> --repo owner/repository --yes
gitfleet pipeline rerun <run-id> --repo owner/repository
```

Trigger, cancel, rerun, deployment creation, release creation, and webhook
changes are mutating operations. In CI, combine them with explicit `--repo`,
`--json`, and `--yes` only where confirmation is required.
