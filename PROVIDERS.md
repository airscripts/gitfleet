# Provider Parity Matrix

This document compares every capability trait method across the
built-in providers. Methods are grouped by trait and classified as:

- **Supported** — both providers implement the method.
- **GitHub-only** — no GitLab equivalent exists (category c).
- **Missing** — GitLab has an equivalent API but the method is not
  yet implemented (category b). These are candidates for future
  implementation.
- **Empty** — marker trait with no methods.

## Summary

| Trait              | GitHub | GitLab | Status         |
|---------------------|--------|--------|----------------|
| RepoOps             | 12     | 12     | Parity         |
| ChangeOps           | 9      | 9      | Parity         |
| ReviewOps           | 3      | 0      | Missing        |
| IssueOps            | 6      | 6      | Parity         |
| LabelOps            | 3      | 3      | Parity         |
| NotificationOps     | 2      | 2      | Parity         |
| PipelineOps         | 8      | 8      | Parity         |
| ReleaseOps          | 5      | 5      | Parity         |
| PlanningOps         | 9      | 0      | Partial gap    |
| WikiOps             | 5      | 5      | Parity         |
| SiteOps             | 3      | 0      | Missing        |
| DiscussionOps       | 3      | 3      | Parity         |
| DeployOps           | 2      | 2      | Parity         |
| EnvironmentOps      | 2      | 2      | Parity         |
| RunnerOps           | 2      | 2      | Parity         |
| WebhookOps          | 4      | 4      | Parity         |
| AccessOps           | 6      | 6      | Parity         |
| IdentityOps         | 6      | 6      | Parity         |
| AnalyticsOps        | 2      | 2      | Parity         |
| SnippetOps          | 4      | 0      | Missing        |
| GovernanceOps       | 3      | 3      | Parity         |
| PolicyOps           | 6      | 0      | Missing        |
| SearchOps           | 3      | 3      | Parity         |
| CodeOps             | 2      | 2      | Parity         |
| SecretOps           | 4      | 4      | Parity         |
| VariableOps         | 3      | 3      | Parity         |
| LicenseOps          | 3      | 3      | Parity         |
| DependencyOps       | 2      | 2      | Parity         |
| AdvisoryOps         | 4      | 4      | Parity         |
| AttestationOps      | 2      | 2      | Parity         |
| BrowseOps           | 2      | 2      | Parity         |
| RawApiOps           | 3      | 3      | Parity         |
| RegistryOps         | 2      | 0      | Missing        |
| DevEnvOps           | 3      | 0      | GitHub-only    |
| TemplateOps         | 1      | 1      | Parity         |
| SecurityOps         | 0      | 0      | Empty marker   |
| MergeAutomationOps  | 0      | 0      | Empty marker   |
| **Total**           | **128**| **84** |                |

## Detailed Parity by Trait

### RepoOps — Parity

| Method                  | GitHub | GitLab |
|-------------------------|--------|--------|
| list_org_repos          | Yes    | Yes    |
| list_user_repos         | Yes    | Yes    |
| list_user_named_repos   | Yes    | Yes    |
| get_repo                | Yes    | Yes    |
| create_repo             | Yes    | Yes    |
| update_repo             | Yes    | Yes    |
| delete_repo             | Yes    | Yes    |
| star_repo               | Yes    | Yes    |
| unstar_repo             | Yes    | Yes    |
| fork_repo               | Yes    | Yes    |
| archive_repo            | Yes    | Yes    |
| unarchive_repo          | Yes    | Yes    |

### ChangeOps — Parity

| Method                | GitHub | GitLab |
|-----------------------|--------|--------|
| list_changes          | Yes    | Yes    |
| get_change            | Yes    | Yes    |
| create_change         | Yes    | Yes    |
| update_change         | Yes    | Yes    |
| merge_change          | Yes    | Yes    |
| comment_on_change     | Yes    | Yes    |
| lock_change           | Yes    | Yes    |
| unlock_change         | Yes    | Yes    |
| list_change_comments  | Yes    | Yes    |

### ReviewOps — Missing (GitLab award emojis API)

| Method                    | GitHub | GitLab |
|---------------------------|--------|--------|
| list_reactions_for_issue  | Yes    | Missing|
| create_reaction_for_issue | Yes    | Missing|
| delete_reaction_for_issue | Yes    | Missing|

GitLab equivalent: Award Emojis API (`/projects/:id/issues/:iid/award_emoji`).

### IssueOps — Parity

| Method              | GitHub | GitLab |
|---------------------|--------|--------|
| get_issue           | Yes    | Yes    |
| create_issue        | Yes    | Yes    |
| list_issues         | Yes    | Yes    |
| update_issue        | Yes    | Yes    |
| comment_on_issue    | Yes    | Yes    |
| list_issue_comments | Yes    | Yes    |

### LabelOps — Parity

| Method        | GitHub | GitLab |
|---------------|--------|--------|
| list_labels   | Yes    | Yes    |
| create_label  | Yes    | Yes    |
| delete_label  | Yes    | Yes    |

### NotificationOps — Parity

| Method                | GitHub | GitLab |
|-----------------------|--------|--------|
| list_notifications    | Yes    | Yes    |
| mark_notifications_read| Yes   | Yes    |

### PipelineOps — Parity

| Method            | GitHub | GitLab |
|-------------------|--------|--------|
| list_workflows    | Yes    | Yes    |
| get_workflow      | Yes    | Yes    |
| dispatch_workflow | Yes    | Yes    |
| list_runs         | Yes    | Yes    |
| get_run           | Yes    | Yes    |
| cancel_run        | Yes    | Yes    |
| rerun             | Yes    | Yes    |
| delete_run        | Yes    | Yes    |

### ReleaseOps — Parity

| Method               | GitHub | GitLab |
|----------------------|--------|--------|
| list_releases        | Yes    | Yes    |
| fetch_release_by_tag | Yes    | Yes    |
| create_release       | Yes    | Yes    |
| update_release       | Yes    | Yes    |
| delete_release       | Yes    | Yes    |

### PlanningOps — Partial gap

| Method            | GitHub | GitLab | Classification |
|--------------------|--------|--------|----------------|
| list_milestones    | Yes    | Missing| Missing (b)    |
| create_milestone   | Yes    | Missing| Missing (b)    |
| get_milestone      | Yes    | Missing| Missing (b)    |
| update_milestone   | Yes    | Missing| Missing (b)    |
| delete_milestone   | Yes    | Missing| Missing (b)    |
| list_projects      | Yes    | Missing| GitHub-only (c)|
| get_project        | Yes    | Missing| GitHub-only (c)|
| create_project     | Yes    | Missing| GitHub-only (c)|
| delete_project     | Yes    | Missing| GitHub-only (c)|

GitLab milestones: `/projects/:id/milestones`.
GitHub Projects (project boards) have no direct GitLab equivalent.
GitLab's "projects" are repositories, not project boards.

### WikiOps — Parity

| Method            | GitHub | GitLab |
|-------------------|--------|--------|
| list_wiki_pages   | Yes    | Yes    |
| get_wiki_page     | Yes    | Yes    |
| create_wiki_page  | Yes    | Yes    |
| update_wiki_page  | Yes    | Yes    |
| delete_wiki_page  | Yes    | Yes    |

### SiteOps — Missing (GitLab Pages API)

| Method        | GitHub | GitLab |
|---------------|--------|--------|
| get_pages     | Yes    | Missing|
| create_pages  | Yes    | Missing|
| remove_pages  | Yes    | Missing|

GitLab equivalent: Pages API (`/projects/:id/pages`).

### DiscussionOps — Parity

| Method             | GitHub | GitLab |
|--------------------|--------|--------|
| list_discussions   | Yes    | Yes    |
| get_discussion     | Yes    | Yes    |
| create_discussion  | Yes    | Yes    |

### DeployOps — Parity

| Method            | GitHub | GitLab |
|-------------------|--------|--------|
| list_deployments  | Yes    | Yes    |
| create_deployment | Yes    | Yes    |

### EnvironmentOps — Parity

| Method              | GitHub | GitLab |
|---------------------|--------|--------|
| list_environments   | Yes    | Yes    |
| create_environment  | Yes    | Yes    |

### RunnerOps — Parity

| Method         | GitHub | GitLab |
|----------------|--------|--------|
| list_runners   | Yes    | Yes    |
| remove_runner  | Yes    | Yes    |

### WebhookOps — Parity

| Method          | GitHub | GitLab |
|-----------------|--------|--------|
| list_webhooks   | Yes    | Yes    |
| create_webhook  | Yes    | Yes    |
| remove_webhook  | Yes    | Yes    |
| test_webhook    | Yes    | Yes    |

### AccessOps — Parity

| Method               | GitHub | GitLab |
|----------------------|--------|--------|
| invite_collaborator  | Yes    | Yes    |
| list_org_members     | Yes    | Yes    |
| remove_org_member    | Yes    | Yes    |
| list_teams           | Yes    | Yes    |
| create_team          | Yes    | Yes    |
| list_team_members    | Yes    | Yes    |

### IdentityOps — Parity

| Method         | GitHub | GitLab |
|----------------|--------|--------|
| list_ssh_keys  | Yes    | Yes    |
| add_ssh_key    | Yes    | Yes    |
| delete_ssh_key | Yes    | Yes    |
| list_gpg_keys  | Yes    | Yes    |
| add_gpg_key    | Yes    | Yes    |
| delete_gpg_key | Yes    | Yes    |

### AnalyticsOps — Parity

| Method              | GitHub | GitLab |
|---------------------|--------|--------|
| get_traffic_views   | Yes    | Yes    |
| get_traffic_clones  | Yes    | Yes    |

### SnippetOps — Missing (GitLab Snippets API)

| Method          | GitHub | GitLab |
|-----------------|--------|--------|
| list_snippets   | Yes    | Missing|
| get_snippet     | Yes    | Missing|
| create_snippet  | Yes    | Missing|
| delete_snippet  | Yes    | Missing|

GitLab equivalent: Snippets API (`/snippets`, `/projects/:id/snippets`).

### GovernanceOps — Parity

| Method          | GitHub | GitLab |
|-----------------|--------|--------|
| list_rulesets   | Yes    | Yes    |
| create_ruleset  | Yes    | Yes    |
| delete_ruleset  | Yes    | Yes    |

### PolicyOps — Missing (GitLab Protected Branches + Tags API)

| Method                 | GitHub | GitLab |
|------------------------|--------|--------|
| get_branch_protection  | Yes    | Missing|
| protect_branch         | Yes    | Missing|
| unprotect_branch       | Yes    | Missing|
| list_tag_protection    | Yes    | Missing|
| create_tag_protection  | Yes    | Missing|
| delete_tag_protection  | Yes    | Missing|

GitLab equivalent: Protected Branches API
(`/projects/:id/protected_branches`) and Protected Tags API
(`/projects/:id/protected_tags`).

### SearchOps — Parity

| Method         | GitHub | GitLab |
|----------------|--------|--------|
| search_issues  | Yes    | Yes    |
| search_repos   | Yes    | Yes    |
| search_code    | Yes    | Yes    |

### CodeOps — Parity

| Method            | GitHub | GitLab |
|-------------------|--------|--------|
| get_file_contents | Yes    | Yes    |
| search_code       | Yes    | Yes    |

### SecretOps — Parity

| Method              | GitHub | GitLab |
|---------------------|--------|--------|
| list_repo_secrets   | Yes    | Yes    |
| get_repo_public_key | Yes    | Yes    |
| set_repo_secret     | Yes    | Yes    |
| delete_repo_secret  | Yes    | Yes    |

### VariableOps — Parity

| Method              | GitHub | GitLab |
|---------------------|--------|--------|
| list_repo_variables | Yes    | Yes    |
| set_repo_variable   | Yes    | Yes    |
| delete_repo_variable| Yes    | Yes    |

### LicenseOps — Parity

| Method        | GitHub | GitLab |
|---------------|--------|--------|
| list_licenses | Yes    | Yes    |
| get_license   | Yes    | Yes    |
| repo_license  | Yes    | Yes    |

### DependencyOps — Parity

| Method              | GitHub | GitLab |
|---------------------|--------|--------|
| sbom                | Yes    | Yes    |
| review_dependencies | Yes    | Yes    |

### AdvisoryOps — Parity

| Method                   | GitHub | GitLab |
|--------------------------|--------|--------|
| list_dependabot_alerts   | Yes    | Yes    |
| list_codeql_alerts       | Yes    | Yes    |
| list_secret_scanning_alerts| Yes  | Yes    |
| get_dependabot_alert     | Yes    | Yes    |

### AttestationOps — Parity

| Method             | GitHub | GitLab |
|--------------------|--------|--------|
| list_attestations  | Yes    | Yes    |
| get_attestation    | Yes    | Yes    |

### BrowseOps — Parity

| Method         | GitHub | GitLab |
|----------------|--------|--------|
| list_contents  | Yes    | Yes    |
| file_contents  | Yes    | Yes    |

### RawApiOps — Parity

| Method      | GitHub | GitLab |
|-------------|--------|--------|
| raw_get     | Yes    | Yes    |
| raw_post    | Yes    | Yes    |
| raw_delete  | Yes    | Yes    |

### RegistryOps — Missing (GitLab Package Registry API)

| Method        | GitHub | GitLab |
|---------------|--------|--------|
| list_packages | Yes    | Missing|
| get_package   | Yes    | Missing|

GitLab equivalent: Package Registry API (`/projects/:id/packages`).

### DevEnvOps — GitHub-only

| Method            | GitHub | GitLab |
|-------------------|--------|--------|
| list_codespaces   | Yes    | N/A    |
| create_codespace  | Yes    | N/A    |
| delete_codespace  | Yes    | N/A    |

GitLab Workspaces (introduced in 16.x) are conceptually similar but
use a different API model. Deferred to post-0.1.0.

### TemplateOps — Parity

| Method               | GitHub | GitLab |
|----------------------|--------|--------|
| list_issue_templates | Yes    | Yes    |

### SecurityOps — Empty marker

No methods. Both providers should implement as empty impl blocks.

### MergeAutomationOps — Empty marker

No methods. Both providers should implement as empty impl blocks.

## GitLab Capability Set

The GitLab provider does not surface the following capabilities
(`capabilities()` does not include these variants):

- `Reviews`
- `Planning`
- `Site`
- `Security`
- `Registry`
- `DevelopmentEnvironments`
- `Snippets`
- `MergeAutomation`
- `RepositoryPolicies`

When a CLI command requires one of these capabilities and the active
provider is GitLab, the caller receives a `GitfleetError` indicating
the provider does not support the operation.
