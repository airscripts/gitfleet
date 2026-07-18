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
provider state. Use `repo list` and `repo view` first when reviewing a
repository before mutation.
