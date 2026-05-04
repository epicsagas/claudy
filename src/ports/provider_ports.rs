use std::collections::HashSet;

use crate::providers::index::{ProviderIndex, ServiceDescriptor};

// Re-export so domain can depend on ports instead of adapters.
pub use crate::providers::index::ProviderIndex as ConcreteProviderIndex;

pub trait ProviderIndexPort {
    fn all(&self) -> &[ServiceDescriptor];
    fn ids(&self) -> Vec<String>;
    fn get(&self, id: &str) -> Option<&ServiceDescriptor>;
    fn categories(&self) -> Vec<String>;
    fn providers_by_category(&self, category: &str) -> Vec<&ServiceDescriptor>;
    fn builtin_secret_keys(&self) -> HashSet<String>;
}

impl ProviderIndexPort for ProviderIndex {
    fn all(&self) -> &[ServiceDescriptor] {
        ProviderIndex::all(self)
    }
    fn ids(&self) -> Vec<String> {
        ProviderIndex::ids(self)
    }
    fn get(&self, id: &str) -> Option<&ServiceDescriptor> {
        ProviderIndex::get(self, id)
    }
    fn categories(&self) -> Vec<String> {
        ProviderIndex::categories(self)
    }
    fn providers_by_category(&self, category: &str) -> Vec<&ServiceDescriptor> {
        ProviderIndex::providers_by_category(self, category)
    }
    fn builtin_secret_keys(&self) -> HashSet<String> {
        ProviderIndex::builtin_secret_keys(self)
    }
}
