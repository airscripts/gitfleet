# Gitfleet Roadmap

This roadmap begins after the TypeScript-based Gitfleet `0.1.0` foundation in
`PLAN.md` is complete. It tracks major platform work that is intentionally out
of scope for the initial refactor.

## Rust Rewrite

Rewrite the validated Gitfleet architecture in Rust after the TypeScript
implementation has stabilized its domain contracts, operation registry, GitHub
provider behavior, CLI semantics, and JSON schemas.

The rewrite must:

- preserve the public command names, options, exit codes, and JSON contracts;
- preserve configuration and credential file compatibility within Gitfleet;
- keep provider-neutral domain and application boundaries;
- keep all network traffic inside provider adapters;
- retain a single operation catalog for CLI exposure;
- provide bounded concurrent fleet operations and cancellation;
- replace the Node.js runtime without reducing supported capabilities;
- ship only after fixture-based parity and live GitHub playbooks pass against
  both implementations.

The Rust runtime, CLI, HTTP, serialization, and async libraries will be
selected during this milestone using measured prototypes. Library selection is
not part of the TypeScript foundation.

## GitLab Provider

Add GitLab as the second provider against the provider contracts established
by the Gitfleet foundation. Support GitLab.com and self-managed instances.

Initial GitLab coverage must include:

- authentication, profiles, host selection, and remote detection;
- repositories and groups;
- merge requests through `change`;
- reviews, discussions, issues, labels, and milestones;
- GitLab CI/CD pipelines, jobs, logs, artifacts, and caches;
- releases, deployments, environments, variables, and protected secrets;
- issue boards through `planning`;
- project wikis and GitLab Pages through `wiki` and `site`;
- package and container registries;
- runners, webhooks, access management, snippets, and analytics;
- security, policy, governance, and fleet operations where the GitLab tier and
  API expose the required capability;
- GitLab Workspaces through `dev` when supported by the target instance.

The provider must declare capabilities dynamically because GitLab features
vary by version, subscription tier, and deployment type. Unsupported features
must produce stable capability errors rather than GitHub-shaped emulation.

Completion requires the shared provider contract suite, GitLab-specific unit
and integration tests, reversible live playbooks for GitLab.com, and a
self-managed compatibility matrix.
