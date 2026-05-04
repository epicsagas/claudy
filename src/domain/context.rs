use crate::ports::config_ports::{ConcreteAppPaths, ConcreteAppRegistry, ConcreteSecretVault};
use crate::ports::provider_ports::ConcreteProviderIndex;
use crate::ports::ui_ports::{OutputPort, PrompterPort};

pub use crate::domain::commands::{DomainCommand, Options};

pub struct Context {
    pub paths: ConcreteAppPaths,
    pub config: ConcreteAppRegistry,
    pub secrets: ConcreteSecretVault,
    pub catalog: ConcreteProviderIndex,
    pub output: Box<dyn OutputPort>,
    pub prompt: Box<dyn PrompterPort>,
    pub options: Options,
}
