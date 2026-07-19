use gitfleet_core::errors::GitfleetError;
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{ProviderContext, ProviderId, TokenSource};
use gitfleet_providers::ProviderRegistry;

pub struct App {
    registry: ProviderRegistry,
    renderer: Renderer,
    context: ProviderContext,
    dry_run: bool,
}

impl App {
    #[cfg(test)]
    pub fn new(
        registry: ProviderRegistry,
        renderer: Renderer,
        provider_id: ProviderId,
        dry_run: bool,
    ) -> Self {
        let context = ProviderContext {
            profile_name: gitfleet_core::constants::DEFAULT_PROFILE_NAME.to_string(),
            provider: provider_id,

            host: match provider_id {
                ProviderId::GitHub => "github.com".to_string(),
                ProviderId::GitLab => "gitlab.com".to_string(),
            },

            token: None,
            token_source: gitfleet_core::provider::TokenSource::None,

            capabilities: registry
                .get(provider_id)
                .map(|provider| provider.capabilities().to_vec())
                .unwrap_or_default(),
        };

        Self::new_with_context(registry, renderer, context, dry_run)
    }

    fn new_with_context(
        registry: ProviderRegistry,
        renderer: Renderer,
        context: ProviderContext,
        dry_run: bool,
    ) -> Self {
        Self {
            registry,
            renderer,
            context,
            dry_run,
        }
    }

    pub fn from_config(renderer: Renderer, dry_run: bool) -> Result<Self, GitfleetError> {
        let context = gitfleet_core::config::resolve_provider_context()?;
        let registry = ProviderRegistry::with_context(&context);

        let provider = registry
            .get(context.provider)
            .map_err(GitfleetError::UnsupportedCapability)?;

        let context = context.with_capabilities(provider.capabilities());

        Ok(Self::new_with_context(registry, renderer, context, dry_run))
    }

    pub fn without_config(renderer: Renderer, dry_run: bool) -> Self {
        let registry = ProviderRegistry::new();
        let context = ProviderContext {
            profile_name: gitfleet_core::constants::DEFAULT_PROFILE_NAME.to_string(),
            provider: ProviderId::GitHub,
            host: "github.com".to_string(),
            token: None,
            token_source: gitfleet_core::provider::TokenSource::None,
            capabilities: Vec::new(),
        };

        Self::new_with_context(registry, renderer, context, dry_run)
    }

    #[allow(dead_code)]
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn provider_id(&self) -> ProviderId {
        self.context.provider
    }

    pub fn provider_host(&self) -> &str {
        &self.context.host
    }

    pub fn profile_name(&self) -> &str {
        &self.context.profile_name
    }

    pub fn capabilities(&self) -> &[gitfleet_core::provider::ProviderCapability] {
        &self.context.capabilities
    }

    pub fn token_source(&self) -> TokenSource {
        self.context.token_source
    }

    pub fn has_token(&self) -> bool {
        self.context.token.is_some()
    }

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn provider(&self) -> Result<&dyn gitfleet_core::provider::GitProvider, GitfleetError> {
        self.registry
            .get(self.context.provider)
            .map_err(GitfleetError::UnsupportedCapability)
    }

    pub fn shared_provider(
        &self,
    ) -> Result<std::sync::Arc<dyn gitfleet_core::provider::GitProvider>, GitfleetError> {
        self.registry
            .get_shared(self.context.provider)
            .map_err(GitfleetError::UnsupportedCapability)
    }
}
