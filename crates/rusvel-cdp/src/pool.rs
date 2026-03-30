//! Static map of Chrome profile id → CDP HTTP base for multi-profile harvest scans.
//! Full lifecycle (connect, health, reconnect) builds on [`crate::CdpClient`].

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeProfileConfig {
    pub id: String,
    #[serde(default)]
    pub platform: String,
    pub cdp_endpoint: String,
}

#[derive(Debug, Clone, Default)]
pub struct CdpPool {
    profiles: Vec<ChromeProfileConfig>,
}

impl CdpPool {
    #[must_use]
    pub fn new(profiles: Vec<ChromeProfileConfig>) -> Self {
        Self { profiles }
    }

    #[must_use]
    pub fn endpoint_for_profile(&self, profile_id: &str) -> Option<&str> {
        self.profiles
            .iter()
            .find(|p| p.id == profile_id)
            .map(|p| p.cdp_endpoint.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_for_profile_resolves() {
        let pool = CdpPool::new(vec![ChromeProfileConfig {
            id: "p1".into(),
            platform: "upwork".into(),
            cdp_endpoint: "http://127.0.0.1:9223".into(),
        }]);
        assert_eq!(
            pool.endpoint_for_profile("p1"),
            Some("http://127.0.0.1:9223")
        );
        assert!(pool.endpoint_for_profile("missing").is_none());
    }
}
