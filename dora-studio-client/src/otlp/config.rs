use serde::{Deserialize, Serialize};

/// Authentication method for connecting to a backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthMethod {
    #[serde(rename = "api_key")]
    ApiKey { header_name: String, key: String },
    #[serde(rename = "bearer_token")]
    BearerToken { token: String },
    #[serde(rename = "none")]
    None,
}

/// Configuration for a SigNoz backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigNozConfig {
    pub base_url: String,
    pub auth: AuthMethod,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

/// Tagged enum of all supported backend configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "backend")]
pub enum BackendConfig {
    #[serde(rename = "signoz")]
    SigNoz(SigNozConfig),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signoz_config_serde_roundtrip() {
        let config = SigNozConfig {
            base_url: "http://localhost:3301".to_string(),
            auth: AuthMethod::None,
            timeout_secs: 30,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SigNozConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.base_url, "http://localhost:3301");
        assert_eq!(deserialized.timeout_secs, 30);
    }

    #[test]
    fn test_auth_method_api_key_serde() {
        let auth = AuthMethod::ApiKey {
            header_name: "SIGNOZ-API-KEY".to_string(),
            key: "secret123".to_string(),
        };
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("api_key"));
        assert!(json.contains("SIGNOZ-API-KEY"));
        let deserialized: AuthMethod = serde_json::from_str(&json).unwrap();
        match deserialized {
            AuthMethod::ApiKey { header_name, key } => {
                assert_eq!(header_name, "SIGNOZ-API-KEY");
                assert_eq!(key, "secret123");
            }
            _ => panic!("Expected ApiKey variant"),
        }
    }

    #[test]
    fn test_auth_method_bearer_token_serde() {
        let auth = AuthMethod::BearerToken {
            token: "my-token".to_string(),
        };
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("bearer_token"));
        let deserialized: AuthMethod = serde_json::from_str(&json).unwrap();
        match deserialized {
            AuthMethod::BearerToken { token } => assert_eq!(token, "my-token"),
            _ => panic!("Expected BearerToken variant"),
        }
    }

    #[test]
    fn test_backend_config_signoz_serde() {
        let config = BackendConfig::SigNoz(SigNozConfig {
            base_url: "http://signoz.example.com".to_string(),
            auth: AuthMethod::ApiKey {
                header_name: "X-API-KEY".to_string(),
                key: "test-key".to_string(),
            },
            timeout_secs: 60,
        });
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("signoz"));
        let deserialized: BackendConfig = serde_json::from_str(&json).unwrap();
        match deserialized {
            BackendConfig::SigNoz(cfg) => {
                assert_eq!(cfg.base_url, "http://signoz.example.com");
                assert_eq!(cfg.timeout_secs, 60);
            }
        }
    }

    #[test]
    fn test_signoz_config_default_timeout() {
        let json = r#"{"base_url":"http://localhost:3301","auth":{"type":"none"}}"#;
        let config: SigNozConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.timeout_secs, 30);
    }
}
