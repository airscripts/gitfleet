use gitfleet_core::errors::UnsupportedCapabilityError;
use gitfleet_core::provider::{GitProvider, ProviderCapability, ProviderContext, ProviderId};

pub struct ProviderRegistry {
    providers: std::collections::HashMap<ProviderId, Box<dyn GitProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut providers: std::collections::HashMap<ProviderId, Box<dyn GitProvider>> =
            std::collections::HashMap::new();

        let github = crate::github::GitHubProvider::new();
        providers.insert(github.id(), Box::new(github));

        let gitlab = crate::gitlab::GitLabProvider::new();
        providers.insert(gitlab.id(), Box::new(gitlab));

        Self { providers }
    }

    pub fn with_provider(id: ProviderId, provider: Box<dyn GitProvider>) -> Self {
        let mut providers: std::collections::HashMap<ProviderId, Box<dyn GitProvider>> =
            std::collections::HashMap::new();
        providers.insert(id, provider);
        Self { providers }
    }

    pub fn with_host(provider_id: ProviderId, host: &str) -> Self {
        let mut providers: std::collections::HashMap<ProviderId, Box<dyn GitProvider>> =
            std::collections::HashMap::new();

        let github: Box<dyn GitProvider> = match provider_id {
            ProviderId::GitHub => Box::new(crate::github::GitHubProvider::with_host(host)),
            ProviderId::GitLab => Box::new(crate::github::GitHubProvider::new()),
        };

        providers.insert(ProviderId::GitHub, github);
        let gitlab: Box<dyn GitProvider> = match provider_id {
            ProviderId::GitHub => Box::new(crate::gitlab::GitLabProvider::new()),
            ProviderId::GitLab => Box::new(crate::gitlab::GitLabProvider::with_host(host)),
        };

        providers.insert(ProviderId::GitLab, gitlab);
        Self { providers }
    }

    pub fn with_context(context: &ProviderContext) -> Self {
        let mut providers: std::collections::HashMap<ProviderId, Box<dyn GitProvider>> =
            std::collections::HashMap::new();

        let github: Box<dyn GitProvider> = match context.provider {
            ProviderId::GitHub => Box::new(crate::github::GitHubProvider::with_context(context)),
            ProviderId::GitLab => Box::new(crate::github::GitHubProvider::new()),
        };
        providers.insert(ProviderId::GitHub, github);

        let gitlab: Box<dyn GitProvider> = match context.provider {
            ProviderId::GitHub => Box::new(crate::gitlab::GitLabProvider::new()),
            ProviderId::GitLab => Box::new(crate::gitlab::GitLabProvider::with_context(context)),
        };
        providers.insert(ProviderId::GitLab, gitlab);

        Self { providers }
    }

    pub fn get(
        &self,
        provider: ProviderId,
    ) -> Result<&dyn GitProvider, UnsupportedCapabilityError> {
        self.providers
            .get(&provider)
            .map(|p| p.as_ref())
            .ok_or_else(|| {
                UnsupportedCapabilityError::new(provider, ProviderCapability::Repositories)
            })
    }

    pub fn require_capability(
        &self,
        provider: ProviderId,
        capability: ProviderCapability,
    ) -> Result<&dyn GitProvider, UnsupportedCapabilityError> {
        let implementation = self.get(provider)?;

        if !implementation.capabilities().contains(&capability) {
            return Err(UnsupportedCapabilityError::new(provider, capability));
        }

        Ok(implementation)
    }
}

pub fn validate_capability_list(capabilities: &[ProviderCapability]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();

    for capability in capabilities {
        if capability.to_string().is_empty() {
            return Err("Provider capability has an empty display name.".to_string());
        }

        if !seen.insert(*capability) {
            return Err(format!(
                "Provider capability '{}' is duplicated.",
                capability
            ));
        }
    }

    Ok(())
}

pub fn validate_provider_capabilities(provider: &dyn GitProvider) -> Result<(), String> {
    validate_capability_list(provider.capabilities())?;

    for capability in provider.capabilities() {
        let implemented = match capability {
            ProviderCapability::Repositories => provider.repo_ops().is_some(),
            ProviderCapability::Changes => provider.change_ops().is_some(),
            ProviderCapability::Reviews => provider.review_ops().is_some(),
            ProviderCapability::Issues => provider.issue_ops().is_some(),
            ProviderCapability::Pipelines => provider.pipeline_ops().is_some(),
            ProviderCapability::Releases => provider.release_ops().is_some(),
            ProviderCapability::Planning => provider.planning_ops().is_some(),
            ProviderCapability::Wiki => provider.wiki_ops().is_some(),
            ProviderCapability::Site => provider.site_ops().is_some(),
            ProviderCapability::Discussions => provider.discussion_ops().is_some(),
            ProviderCapability::Security => provider.security_ops().is_some(),
            ProviderCapability::Registry => provider.registry_ops().is_some(),
            ProviderCapability::DevelopmentEnvironments => provider.dev_env_ops().is_some(),
            ProviderCapability::Deployments => provider.deploy_ops().is_some(),
            ProviderCapability::Environments => provider.environment_ops().is_some(),
            ProviderCapability::Runners => provider.runner_ops().is_some(),
            ProviderCapability::Webhooks => provider.webhook_ops().is_some(),
            ProviderCapability::Access => provider.access_ops().is_some(),
            ProviderCapability::Identity => provider.identity_ops().is_some(),
            ProviderCapability::Analytics => provider.analytics_ops().is_some(),
            ProviderCapability::Snippets => provider.snippet_ops().is_some(),
            ProviderCapability::Governance => provider.governance_ops().is_some(),
            ProviderCapability::MergeAutomation => provider.merge_automation_ops().is_some(),
            ProviderCapability::RepositoryPolicies => provider.policy_ops().is_some(),
            ProviderCapability::Notifications => provider.notification_ops().is_some(),
            ProviderCapability::Search => provider.search_ops().is_some(),
            ProviderCapability::Code => provider.code_ops().is_some(),
            ProviderCapability::Labels => provider.label_ops().is_some(),
            ProviderCapability::Templates => provider.template_ops().is_some(),
            ProviderCapability::Dependencies => provider.dependency_ops().is_some(),
            ProviderCapability::Advisories => provider.advisory_ops().is_some(),
            ProviderCapability::Attestations => provider.attestation_ops().is_some(),
            ProviderCapability::Secrets => provider.secret_ops().is_some(),
            ProviderCapability::Variables => provider.variable_ops().is_some(),
            ProviderCapability::Licenses => provider.license_ops().is_some(),
            ProviderCapability::Browsing => provider.browse_ops().is_some(),
            ProviderCapability::RawApi => provider.raw_api_ops().is_some(),
        };

        if !implemented {
            return Err(format!(
                "Provider '{}' declares '{}' without implementing its operation trait.",
                provider.id(),
                capability
            ));
        }
    }

    Ok(())
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::provider::ProviderId;

    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = ProviderRegistry::new();

        let github = registry.get(ProviderId::GitHub);

        assert!(github.is_ok());

        let gitlab = registry.get(ProviderId::GitLab);

        assert!(gitlab.is_ok());
    }

    #[test]
    fn test_registry_get_github() {
        let registry = ProviderRegistry::new();

        let provider = registry.get(ProviderId::GitHub).unwrap();

        assert_eq!(provider.id(), ProviderId::GitHub);

        assert_eq!(provider.default_host(), "github.com");
    }

    #[test]
    fn test_registry_get_gitlab() {
        let registry = ProviderRegistry::new();

        let provider = registry.get(ProviderId::GitLab).unwrap();

        assert_eq!(provider.id(), ProviderId::GitLab);

        assert_eq!(provider.default_host(), "gitlab.com");
    }

    #[test]
    fn test_registry_require_capability_supported() {
        let registry = ProviderRegistry::new();

        let result =
            registry.require_capability(ProviderId::GitHub, ProviderCapability::Repositories);
        assert!(result.is_ok());

        assert_eq!(result.unwrap().id(), ProviderId::GitHub);
    }

    #[test]
    fn test_registry_require_capability_all_github() {
        let registry = ProviderRegistry::new();

        let capabilities = [
            ProviderCapability::Repositories,
            ProviderCapability::Changes,
            ProviderCapability::Issues,
            ProviderCapability::Pipelines,
            ProviderCapability::Releases,
            ProviderCapability::Wiki,
            ProviderCapability::Discussions,
            ProviderCapability::Webhooks,
            ProviderCapability::Search,
            ProviderCapability::Labels,
            ProviderCapability::Notifications,
        ];

        for cap in &capabilities {
            let result = registry.require_capability(ProviderId::GitHub, *cap);

            assert!(result.is_ok(), "GitHub should support {:?}", cap);
        }
    }

    #[test]
    fn test_registry_require_capability_gitlab_supported() {
        let registry = ProviderRegistry::new();

        let capabilities = [
            ProviderCapability::Repositories,
            ProviderCapability::Changes,
            ProviderCapability::Issues,
            ProviderCapability::Pipelines,
            ProviderCapability::Labels,
            ProviderCapability::Webhooks,
        ];

        for cap in &capabilities {
            let result = registry.require_capability(ProviderId::GitLab, *cap);

            assert!(result.is_ok(), "GitLab should support {:?}", cap);
        }
    }

    #[test]
    fn test_github_provider_has_many_capabilities() {
        let registry = ProviderRegistry::new();

        let provider = registry.get(ProviderId::GitHub).unwrap();

        assert!(provider.capabilities().len() > 10);
    }

    #[test]
    fn test_gitlab_provider_has_capabilities() {
        let registry = ProviderRegistry::new();

        let provider = registry.get(ProviderId::GitLab).unwrap();

        assert!(provider.capabilities().len() > 5);
    }

    #[test]
    fn test_github_provider_ops() {
        let registry = ProviderRegistry::new();

        let provider = registry.get(ProviderId::GitHub).unwrap();

        assert!(provider.repo_ops().is_some());

        assert!(provider.change_ops().is_some());
        assert!(provider.issue_ops().is_some());

        assert!(provider.pipeline_ops().is_some());
        assert!(provider.release_ops().is_some());

        assert!(provider.label_ops().is_some());
        assert!(provider.notification_ops().is_some());

        assert!(provider.webhook_ops().is_some());
        assert!(provider.search_ops().is_some());

        assert!(provider.code_ops().is_some());
        assert!(provider.environment_ops().is_some());

        assert!(provider.runner_ops().is_some());
        assert!(provider.secret_ops().is_some());

        assert!(provider.variable_ops().is_some());
        assert!(provider.discussion_ops().is_some());

        assert!(provider.deploy_ops().is_some());
        assert!(provider.access_ops().is_some());

        assert!(provider.identity_ops().is_some());
        assert!(provider.analytics_ops().is_some());

        assert!(provider.governance_ops().is_some());
        assert!(provider.license_ops().is_some());

        assert!(provider.browse_ops().is_some());
    }

    #[test]
    fn test_gitlab_provider_ops() {
        let registry = ProviderRegistry::new();

        let provider = registry.get(ProviderId::GitLab).unwrap();

        assert!(provider.repo_ops().is_some());

        assert!(provider.change_ops().is_some());
        assert!(provider.issue_ops().is_some());

        assert!(provider.pipeline_ops().is_some());
        assert!(provider.release_ops().is_some());

        assert!(provider.label_ops().is_some());
        assert!(provider.notification_ops().is_some());

        assert!(provider.webhook_ops().is_some());
        assert!(provider.search_ops().is_some());

        assert!(provider.code_ops().is_some());
        assert!(provider.environment_ops().is_some());

        assert!(provider.runner_ops().is_some());
        assert!(provider.variable_ops().is_some());

        assert!(provider.discussion_ops().is_some());
        assert!(provider.access_ops().is_some());

        assert!(provider.identity_ops().is_some());
        assert!(provider.browse_ops().is_some());

        assert!(provider.template_ops().is_some());
    }

    #[test]
    fn test_gitlab_provider_missing_ops() {
        let registry = ProviderRegistry::new();

        let provider = registry.get(ProviderId::GitLab).unwrap();

        assert!(provider.deploy_ops().is_some());

        assert!(provider.analytics_ops().is_some());
        assert!(provider.governance_ops().is_some());

        assert!(provider.secret_ops().is_some());
        assert!(provider.license_ops().is_some());

        assert!(provider.dependency_ops().is_some());
        assert!(provider.advisory_ops().is_some());

        assert!(provider.attestation_ops().is_some());
    }

    #[test]
    fn test_builtin_provider_capability_lists_are_valid() {
        let registry = ProviderRegistry::new();

        for provider_id in [ProviderId::GitHub, ProviderId::GitLab] {
            let provider = registry.get(provider_id).unwrap();

            assert!(validate_provider_capabilities(provider).is_ok());
        }
    }
}
