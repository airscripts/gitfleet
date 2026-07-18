# Workspace Fleets

Use `workspace` for bounded multi-repository execution.

Workspaces are for repeated operations across a named set of repositories. They
make the target list explicit, preserve result ordering, and let Gitfleet report
skipped repositories when a target does not match the active profile.

```bash
gitfleet workspace define \
  --name platform \
  --repos owner/api \
  --repos owner/web

gitfleet workspace list
gitfleet workspace archive platform --dry-run
gitfleet workspace archive platform --yes
```

Workspace targets may include provider and host:

```text
github@github.com:owner/repository
github@github.example.com:platform/repository
gitlab@gitlab.com:group/project
```

Repositories that do not match the active profile provider and host are reported
as skipped.

Use `--dry-run` before any fleet mutation. A workspace command can affect many
repositories, so review the target list, skipped list, and planned action before
adding `--yes`.
