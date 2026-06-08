// Re-export provider types from llm-kernel.
// The catalog is now maintained in llm-kernel's catalog.json.
pub use llm_kernel::provider::{ProviderIndex, ServiceDescriptor};

/// Simple model descriptor for `model_choices` in catalog entries.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub description: String,
}

/// Legacy free function preserved for backward compat.
pub fn load_index() -> anyhow::Result<ProviderIndex> {
    Ok(ProviderIndex::embedded().clone())
}
