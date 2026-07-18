# Repository Lifecycle

Use `repo` for repository creation, discovery, cloning, metadata, stars, forks,
and retirement.

This workflow is for individual repositories. Use
[Workspace Fleets](./workspace-fleets.md) when the same action needs to run
across a defined set of repositories.

```bash
gitfleet repo create service-api --private --owner platform --owner-type org --initialize
gitfleet repo list --owner platform
gitfleet repo view platform/service-api
gitfleet repo clone platform/service-api
gitfleet repo edit platform/service-api --description "Service API"
```

Clone a whole owner set when setting up a local working copy for an organization
or user:

```bash
gitfleet repo clone --all --org platform --directory repos --dry-run
gitfleet repo clone --all --org platform --directory repos
gitfleet repo clone --all --user alice --directory user-repos --ssh
```

Bulk clone skips forks and archived repositories by default. Add
`--include-forks` or `--include-archived` when you need a complete mirror.

Fork and star workflows stay under `repo`:

```bash
gitfleet repo fork create owner/repository
gitfleet repo star owner/repository
gitfleet repo unstar owner/repository
```

Archive before delete when retirement needs to be reversible:

```bash
gitfleet repo archive owner/repository --yes
gitfleet repo delete owner/repository --yes
```

Creation, edit, archive, rename, fork, star, and delete commands all change
provider state. Clone commands only write local directories, but a bulk clone
can create many folders and should be previewed with `--dry-run`.
