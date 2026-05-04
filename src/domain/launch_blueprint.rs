use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LaunchTarget {
    pub profile: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub family: String,
    pub base_url: String,
    pub model: String,
    pub model_tiers: HashMap<String, String>,
    pub auth_mode: String,
    pub secret_key: String,
    pub literal_auth_token: String,
    pub test_url: String,
}

#[derive(Debug, Clone)]
pub struct LaunchBlueprint {
    pub profile: String,
    pub forwarded_args: Vec<String>,
    pub hide_banner: bool,
    pub mode: Option<String>,
}
