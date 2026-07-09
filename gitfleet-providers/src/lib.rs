pub mod github;
pub mod gitlab;
pub mod registry;

pub use github::GitHubProvider;
pub use gitlab::GitLabProvider;
pub use registry::ProviderRegistry;
