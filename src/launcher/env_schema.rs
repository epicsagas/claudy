use std::collections::HashMap;

/// Validated environment variable map. Rejects malformed entries on construction.
#[derive(Debug, Clone)]
pub struct EnvMap {
    vars: HashMap<String, String>,
}

impl EnvMap {
    /// Parse from KEY=VALUE strings. Returns error on malformed entries.
    pub fn from_env_slice(pairs: &[String]) -> anyhow::Result<Self> {
        let mut vars = HashMap::new();
        for pair in pairs {
            let (key, value) = pair
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("malformed env pair: {:?}", pair))?;
            Self::validate_key(key)?;
            vars.insert(key.to_string(), value.to_string());
        }
        Ok(Self { vars })
    }

    /// Parse from KEY=VALUE strings, silently dropping malformed entries.
    pub fn from_env_slice_lenient(pairs: &[String]) -> Self {
        let vars = pairs
            .iter()
            .filter_map(|s| s.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Self { vars }
    }

    /// Inherit current process environment.
    pub fn inherit_system() -> Self {
        let vars: HashMap<String, String> = std::env::vars().collect();
        Self { vars }
    }

    fn validate_key(key: &str) -> anyhow::Result<()> {
        if key.is_empty() {
            anyhow::bail!("empty env key");
        }
        let first = key.as_bytes()[0];
        if !first.is_ascii_alphabetic() && first != b'_' {
            anyhow::bail!("env key must start with letter or underscore: {:?}", key);
        }
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.vars.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(|s| s.as_str())
    }

    pub fn remove(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn to_env_slice(&self) -> Vec<String> {
        self.vars
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.vars.iter()
    }

    pub fn contains_prefix(&self, prefix: &str) -> bool {
        self.vars.keys().any(|k| k.starts_with(prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_env_slice() {
        let pairs = vec!["PATH=/usr/bin".to_string(), "HOME=/root".to_string()];
        let map = EnvMap::from_env_slice(&pairs).expect("parse");
        assert_eq!(map.get("PATH"), Some("/usr/bin"));
        assert_eq!(map.get("HOME"), Some("/root"));
    }

    #[test]
    fn test_rejects_malformed() {
        let pairs = vec!["NO_EQUALS_SIGN".to_string()];
        let result = EnvMap::from_env_slice(&pairs);
        assert!(result.is_err());
    }

    #[test]
    fn test_rejects_empty_key() {
        let pairs = vec!["=value".to_string()];
        let result = EnvMap::from_env_slice(&pairs);
        assert!(result.is_err());
    }

    #[test]
    fn test_rejects_numeric_start() {
        let pairs = vec!["1KEY=value".to_string()];
        let result = EnvMap::from_env_slice(&pairs);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_get_remove() {
        let pairs = vec!["KEY=val".to_string()];
        let mut map = EnvMap::from_env_slice(&pairs).expect("parse");
        assert_eq!(map.get("KEY"), Some("val"));
        map.set("KEY", "new");
        assert_eq!(map.get("KEY"), Some("new"));
        map.remove("KEY");
        assert_eq!(map.get("KEY"), None);
    }

    #[test]
    fn test_to_env_slice_roundtrip() {
        let pairs = vec!["A=1".to_string(), "B=2".to_string()];
        let map = EnvMap::from_env_slice(&pairs).expect("parse");
        let out = map.to_env_slice();
        let out_map: HashMap<_, _> = out
            .iter()
            .filter_map(|s| s.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        assert_eq!(out_map.get("A").map(|s| s.as_str()), Some("1"));
        assert_eq!(out_map.get("B").map(|s| s.as_str()), Some("2"));
    }
}
