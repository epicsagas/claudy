use crate::config::registry::AppRegistry;
use crate::domain::launch_blueprint::LaunchTarget;
use crate::ports::launch_ports::ProfileGateway;
use crate::providers::index::ProviderIndex;

pub struct RoutingAdapter<'a> {
    pub catalog: &'a ProviderIndex,
    pub config: &'a AppRegistry,
}

impl<'a> ProfileGateway for RoutingAdapter<'a> {
    fn resolve_target(&self, profile: &str) -> anyhow::Result<LaunchTarget> {
        crate::routing::resolver::route_profile(profile, self.catalog, self.config).map_err(|_| {
            anyhow::anyhow!(
                "The command or profile '{}' is not recognized. Use 'claudy ls' to see available profiles.",
                profile
            )
        })
    }
}
