use std::collections::HashSet;

use crate::provider::ProviderCapability;

pub struct OperationFamily {
    pub name: &'static str,
    pub description: &'static str,
    pub capability: Option<ProviderCapability>,
}

static OPERATION_DEFINITIONS: &[(&str, &str, Option<ProviderCapability>)] = &[
    ("auth", "Manage provider accounts and profiles.", None),
    (
        "repo",
        "Manage repositories.",
        Some(ProviderCapability::Repositories),
    ),
    (
        "change",
        "Manage proposed changes.",
        Some(ProviderCapability::Changes),
    ),
    (
        "review",
        "Manage reviews and conversations.",
        Some(ProviderCapability::Reviews),
    ),
    (
        "issue",
        "Manage issues and work items.",
        Some(ProviderCapability::Issues),
    ),
    (
        "pipeline",
        "Manage pipelines, runs, artifacts, and caches.",
        Some(ProviderCapability::Pipelines),
    ),
    (
        "release",
        "Manage releases and assets.",
        Some(ProviderCapability::Releases),
    ),
    ("workspace", "Manage repository fleets.", None),
    (
        "govern",
        "Inspect and govern repository fleets.",
        Some(ProviderCapability::Governance),
    ),
    (
        "policy",
        "Manage repository policies.",
        Some(ProviderCapability::RepositoryPolicies),
    ),
    (
        "planning",
        "Manage boards, milestones, and iterations.",
        Some(ProviderCapability::Planning),
    ),
    (
        "wiki",
        "Manage repository wikis.",
        Some(ProviderCapability::Wiki),
    ),
    (
        "site",
        "Manage static repository sites.",
        Some(ProviderCapability::Site),
    ),
    (
        "discussion",
        "Manage provider discussions.",
        Some(ProviderCapability::Discussions),
    ),
    (
        "inbox",
        "Manage notifications, activity, and review requests.",
        Some(ProviderCapability::Notifications),
    ),
    (
        "search",
        "Search provider resources.",
        Some(ProviderCapability::Search),
    ),
    (
        "code",
        "Inspect repository code and history.",
        Some(ProviderCapability::Code),
    ),
    (
        "label",
        "Manage and synchronize labels.",
        Some(ProviderCapability::Labels),
    ),
    (
        "template",
        "Discover repository templates.",
        Some(ProviderCapability::Templates),
    ),
    (
        "deps",
        "Inspect dependencies and dependency changes.",
        Some(ProviderCapability::Dependencies),
    ),
    (
        "advisory",
        "Manage security advisories.",
        Some(ProviderCapability::Advisories),
    ),
    (
        "attestation",
        "Inspect and verify artifact provenance.",
        Some(ProviderCapability::Attestations),
    ),
    (
        "security",
        "Inspect security alerts, audit, and compliance.",
        Some(ProviderCapability::Security),
    ),
    (
        "registry",
        "Manage packages and container images.",
        Some(ProviderCapability::Registry),
    ),
    (
        "dev",
        "Manage hosted development environments.",
        Some(ProviderCapability::DevelopmentEnvironments),
    ),
    (
        "deploy",
        "Manage deployments.",
        Some(ProviderCapability::Deployments),
    ),
    (
        "environment",
        "Manage deployment environments.",
        Some(ProviderCapability::Environments),
    ),
    (
        "secret",
        "Manage provider secrets.",
        Some(ProviderCapability::Secrets),
    ),
    (
        "variable",
        "Manage provider variables.",
        Some(ProviderCapability::Variables),
    ),
    (
        "runner",
        "Manage pipeline runners.",
        Some(ProviderCapability::Runners),
    ),
    (
        "webhook",
        "Manage webhooks and deliveries.",
        Some(ProviderCapability::Webhooks),
    ),
    (
        "access",
        "Manage organizations, groups, teams, and access.",
        Some(ProviderCapability::Access),
    ),
    (
        "identity",
        "Manage provider account identity keys.",
        Some(ProviderCapability::Identity),
    ),
    (
        "analytics",
        "Inspect repository and pipeline analytics.",
        Some(ProviderCapability::Analytics),
    ),
    (
        "snippet",
        "Manage provider-hosted snippets.",
        Some(ProviderCapability::Snippets),
    ),
    (
        "license",
        "Discover and inspect licenses.",
        Some(ProviderCapability::Licenses),
    ),
    (
        "browse",
        "Open provider resources in a browser.",
        Some(ProviderCapability::Browsing),
    ),
    (
        "api",
        "Make a raw request to the active provider.",
        Some(ProviderCapability::RawApi),
    ),
    ("alias", "Manage Gitfleet command aliases.", None),
    ("completion", "Generate shell completions.", None),
    ("config", "Manage Gitfleet configuration.", None),
    ("help", "Show help for Gitfleet or a command.", None),
    ("version", "Show version information.", None),
];

pub fn operation_families() -> Vec<OperationFamily> {
    OPERATION_DEFINITIONS
        .iter()
        .map(|(name, description, capability)| OperationFamily {
            name,
            description,
            capability: *capability,
        })
        .collect()
}

pub fn get_operation_family(name: &str) -> Option<OperationFamily> {
    OPERATION_DEFINITIONS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(name, description, capability)| OperationFamily {
            name,
            description,
            capability: *capability,
        })
}

pub fn validate_capability_contract() -> Result<(), String> {
    let mut seen_capabilities = HashSet::new();

    for family in operation_families() {
        if family.name.is_empty() || family.description.is_empty() {
            return Err(format!(
                "Operation family '{}' must have a name and description.",
                family.name
            ));
        }

        if let Some(capability) = family.capability {
            if capability.to_string().is_empty() {
                return Err(format!(
                    "Operation family '{}' has an unnamed capability.",
                    family.name
                ));
            }

            if !seen_capabilities.insert(capability) {
                return Err(format!(
                    "Capability '{}' is mapped to multiple operation families.",
                    capability
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_families_count() {
        let families = operation_families();

        assert_eq!(families.len(), OPERATION_DEFINITIONS.len());
    }

    #[test]
    fn test_operation_families_all_have_names() {
        let families = operation_families();

        for f in &families {
            assert!(!f.name.is_empty());

            assert!(!f.description.is_empty());
        }
    }

    #[test]
    fn test_get_operation_family_known() {
        let family = get_operation_family("repo").unwrap();

        assert_eq!(family.name, "repo");

        assert_eq!(family.capability, Some(ProviderCapability::Repositories));
    }

    #[test]
    fn test_get_operation_family_auth() {
        let family = get_operation_family("auth").unwrap();

        assert_eq!(family.name, "auth");

        assert!(family.capability.is_none());
    }

    #[test]
    fn test_get_operation_family_unknown() {
        assert!(get_operation_family("nonexistent").is_none());
    }

    #[test]
    fn test_validate_capability_contract() {
        assert!(validate_capability_contract().is_ok());
    }

    #[test]
    fn test_get_operation_family_all_defined() {
        for (name, _, _) in OPERATION_DEFINITIONS {
            let family = get_operation_family(name);

            assert!(family.is_some(), "Missing operation family for: {name}");
        }
    }
}
