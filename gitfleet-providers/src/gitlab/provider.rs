use gitfleet_core::provider::{
    AccessOps, AnalyticsOps, BrowseOps, ChangeOps, CodeOps, DeployOps, EnvironmentOps, GitProvider,
    GovernanceOps, IdentityOps, IssueOps, LabelOps, LicenseOps, NotificationOps, PipelineOps,
    PlanningOps, PolicyOps, ProviderCapability, ProviderId, RawApiOps, RegistryOps, ReleaseOps,
    RepoOps, ReviewOps, RunnerOps, SearchOps, SecretOps, SiteOps, SnippetOps, TemplateOps,
    VariableOps, WebhookOps, WikiOps,
};

static CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::Repositories,
    ProviderCapability::Changes,
    ProviderCapability::Reviews,
    ProviderCapability::Issues,
    ProviderCapability::Pipelines,
    ProviderCapability::Releases,
    ProviderCapability::Planning,
    ProviderCapability::Wiki,
    ProviderCapability::Site,
    ProviderCapability::Webhooks,
    ProviderCapability::Access,
    ProviderCapability::Identity,
    ProviderCapability::Notifications,
    ProviderCapability::Search,
    ProviderCapability::Code,
    ProviderCapability::Labels,
    ProviderCapability::Templates,
    ProviderCapability::Environments,
    ProviderCapability::Runners,
    ProviderCapability::Variables,
    ProviderCapability::Browsing,
    ProviderCapability::RawApi,
    ProviderCapability::Deployments,
    ProviderCapability::Analytics,
    ProviderCapability::Governance,
    ProviderCapability::Secrets,
    ProviderCapability::Licenses,
    ProviderCapability::Snippets,
    ProviderCapability::RepositoryPolicies,
    ProviderCapability::Registry,
];

pub struct GitLabProvider {
    client: crate::gitlab::client::ProviderClient,
}

impl Default for GitLabProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GitLabProvider {
    pub fn new() -> Self {
        Self {
            client: crate::gitlab::client::ProviderClient::new(),
        }
    }

    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            client: crate::gitlab::client::ProviderClient::with_base_url(base_url),
        }
    }

    pub fn with_host(host: &str) -> Self {
        Self {
            client: crate::gitlab::client::ProviderClient::with_host(host),
        }
    }

    pub fn with_context(context: &gitfleet_core::provider::ProviderContext) -> Self {
        Self {
            client: crate::gitlab::client::ProviderClient::with_context(
                &context.host,
                context.token.clone(),
            ),
        }
    }
}

impl GitProvider for GitLabProvider {
    fn id(&self) -> ProviderId {
        ProviderId::GitLab
    }

    fn default_host(&self) -> &'static str {
        "gitlab.com"
    }

    fn capabilities(&self) -> &[ProviderCapability] {
        CAPABILITIES
    }

    fn repo_ops(&self) -> Option<&dyn RepoOps> {
        Some(&self.client)
    }

    fn change_ops(&self) -> Option<&dyn ChangeOps> {
        Some(&self.client)
    }

    fn review_ops(&self) -> Option<&dyn ReviewOps> {
        Some(&self.client)
    }

    fn issue_ops(&self) -> Option<&dyn IssueOps> {
        Some(&self.client)
    }

    fn pipeline_ops(&self) -> Option<&dyn PipelineOps> {
        Some(&self.client)
    }

    fn release_ops(&self) -> Option<&dyn ReleaseOps> {
        Some(&self.client)
    }

    fn planning_ops(&self) -> Option<&dyn PlanningOps> {
        Some(&self.client)
    }

    fn wiki_ops(&self) -> Option<&dyn WikiOps> {
        Some(&self.client)
    }

    fn site_ops(&self) -> Option<&dyn SiteOps> {
        Some(&self.client)
    }

    fn webhook_ops(&self) -> Option<&dyn WebhookOps> {
        Some(&self.client)
    }

    fn access_ops(&self) -> Option<&dyn AccessOps> {
        Some(&self.client)
    }

    fn identity_ops(&self) -> Option<&dyn IdentityOps> {
        Some(&self.client)
    }

    fn notification_ops(&self) -> Option<&dyn NotificationOps> {
        Some(&self.client)
    }

    fn search_ops(&self) -> Option<&dyn SearchOps> {
        Some(&self.client)
    }

    fn code_ops(&self) -> Option<&dyn CodeOps> {
        Some(&self.client)
    }

    fn label_ops(&self) -> Option<&dyn LabelOps> {
        Some(&self.client)
    }

    fn template_ops(&self) -> Option<&dyn TemplateOps> {
        Some(&self.client)
    }

    fn environment_ops(&self) -> Option<&dyn EnvironmentOps> {
        Some(&self.client)
    }

    fn runner_ops(&self) -> Option<&dyn RunnerOps> {
        Some(&self.client)
    }

    fn variable_ops(&self) -> Option<&dyn VariableOps> {
        Some(&self.client)
    }

    fn browse_ops(&self) -> Option<&dyn BrowseOps> {
        Some(&self.client)
    }

    fn raw_api_ops(&self) -> Option<&dyn RawApiOps> {
        Some(&self.client)
    }

    fn deploy_ops(&self) -> Option<&dyn DeployOps> {
        Some(&self.client)
    }

    fn analytics_ops(&self) -> Option<&dyn AnalyticsOps> {
        Some(&self.client)
    }

    fn governance_ops(&self) -> Option<&dyn GovernanceOps> {
        Some(&self.client)
    }

    fn secret_ops(&self) -> Option<&dyn SecretOps> {
        Some(&self.client)
    }

    fn license_ops(&self) -> Option<&dyn LicenseOps> {
        Some(&self.client)
    }

    fn snippet_ops(&self) -> Option<&dyn SnippetOps> {
        Some(&self.client)
    }

    fn policy_ops(&self) -> Option<&dyn PolicyOps> {
        Some(&self.client)
    }

    fn registry_ops(&self) -> Option<&dyn RegistryOps> {
        Some(&self.client)
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::provider::ProviderId;

    use super::*;

    #[test]
    fn test_gitlab_provider_new() {
        let provider = GitLabProvider::new();

        assert_eq!(provider.id(), ProviderId::GitLab);

        assert_eq!(provider.default_host(), "gitlab.com");
    }

    #[test]
    fn test_gitlab_provider_with_base_url() {
        let provider = GitLabProvider::with_base_url("http://localhost:8080");

        assert_eq!(provider.id(), ProviderId::GitLab);
    }

    #[test]
    fn test_gitlab_provider_default() {
        let provider = GitLabProvider::default();

        assert_eq!(provider.id(), ProviderId::GitLab);
    }

    #[test]
    fn test_gitlab_provider_capabilities() {
        let provider = GitLabProvider::new();

        let caps = provider.capabilities();

        assert!(caps.contains(&ProviderCapability::Repositories));

        assert!(caps.contains(&ProviderCapability::Changes));
        assert!(caps.contains(&ProviderCapability::Issues));

        assert!(caps.contains(&ProviderCapability::Pipelines));
        assert!(caps.contains(&ProviderCapability::Labels));

        assert!(caps.contains(&ProviderCapability::Deployments));
        assert!(caps.contains(&ProviderCapability::Analytics));

        assert!(caps.contains(&ProviderCapability::Governance));
        assert!(caps.contains(&ProviderCapability::Secrets));

        assert!(caps.contains(&ProviderCapability::Licenses));
        assert!(!caps.contains(&ProviderCapability::Dependencies));
        assert!(!caps.contains(&ProviderCapability::Advisories));
        assert!(!caps.contains(&ProviderCapability::Attestations));
        assert!(!caps.contains(&ProviderCapability::Discussions));
    }
}
