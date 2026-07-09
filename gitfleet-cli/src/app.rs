use gitfleet_core::errors::GitfleetError;
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::ProviderId;
use gitfleet_providers::ProviderRegistry;

pub struct App {
    registry: ProviderRegistry,
    renderer: Renderer,
    provider_id: ProviderId,
    dry_run: bool,
}

impl App {
    pub fn new(
        registry: ProviderRegistry,
        renderer: Renderer,
        provider_id: ProviderId,
        dry_run: bool,
    ) -> Self {
        Self {
            registry,
            renderer,
            provider_id,
            dry_run,
        }
    }

    pub fn from_config(renderer: Renderer, dry_run: bool) -> Result<Self, GitfleetError> {
        let registry = ProviderRegistry::new();

        let profile = gitfleet_core::config::get_resolved_profile()
            .unwrap_or_else(|_| gitfleet_core::constants::DEFAULT_PROFILE_NAME.to_string());

        let provider_id = resolve_provider_id(&profile)?;
        Ok(Self::new(registry, renderer, provider_id, dry_run))
    }

    #[allow(dead_code)]
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn provider_id(&self) -> ProviderId {
        self.provider_id
    }

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn provider(&self) -> Result<&dyn gitfleet_core::provider::GitProvider, GitfleetError> {
        self.registry
            .get(self.provider_id)
            .map_err(GitfleetError::UnsupportedCapability)
    }
}

fn resolve_provider_id(profile: &str) -> Result<ProviderId, GitfleetError> {
    let p = gitfleet_core::config::get_profile(profile)?;

    match p {
        Some(profile) => match profile.provider.as_deref() {
            Some("gitlab") => Ok(ProviderId::GitLab),
            _ => Ok(ProviderId::GitHub),
        },

        None => Ok(ProviderId::GitHub),
    }
}
