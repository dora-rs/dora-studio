use std::sync::Mutex;

use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::otlp::config::{AuthMethod, BackendConfig, SigNozConfig};
use crate::otlp::create_backend;
use crate::otlp::types::{Span, TraceQuery};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum SignozRequest {
    HealthCheck,
    QueryTraces(TraceQuery),
}

#[derive(Debug, Clone)]
pub enum SignozResponse {
    HealthOk,
    HealthError(String),
    Traces(Vec<Span>),
    TracesError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ConnectionStatus {
    #[default]
    Unknown,
    Connected,
    Disconnected,
    Error,
}

// ---------------------------------------------------------------------------
// Global statics  (same pattern as src/api.rs)
// ---------------------------------------------------------------------------

static SIGNOZ_RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);
static SIGNOZ_SENDER: Mutex<Option<UnboundedSender<SignozRequest>>> = Mutex::new(None);
static PENDING_SIGNOZ_RESPONSES: Mutex<Vec<SignozResponse>> = Mutex::new(Vec::new());
static SIGNOZ_CONNECTION_STATUS: Mutex<ConnectionStatus> = Mutex::new(ConnectionStatus::Unknown);
static SIGNOZ_CONFIGURED: Mutex<bool> = Mutex::new(false);

// ---------------------------------------------------------------------------
// Login support
// ---------------------------------------------------------------------------

/// Attempt to log in to SigNoz and obtain a JWT access token.
///
/// POST /api/v1/login  { "email": "…", "password": "…" }
/// Returns the accessJwt string on success.
async fn signoz_login(base_url: &str, email: &str, password: &str) -> Result<String, String> {
    let url = format!("{}/api/v1/login", base_url.trim_end_matches('/'));
    let body = serde_json::json!({ "email": email, "password": password });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("login request failed: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("login failed (HTTP {}): {}", status.as_u16(), text));
    }

    // Response shape: { "accessJwt": "…", "refreshJwt": "…", "userId": "…" }
    let parsed: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("login response parse error: {}", e))?;

    parsed["accessJwt"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "login response missing accessJwt field".to_string())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

const DEFAULT_SIGNOZ_BASE_URL: &str = "http://localhost:8080";

/// Read SigNoz connection parameters from environment variables.
///
/// Defaults to `http://localhost:8080` when `SIGNOZ_BASE_URL` is not set.
///
/// Priority:
/// 1. `SIGNOZ_API_KEY` → ApiKey auth
/// 2. `SIGNOZ_EMAIL` + `SIGNOZ_PASSWORD` → login at startup for JWT (handled later)
/// 3. Neither → AuthMethod::None (will fail on auth-required instances)
pub fn signoz_config_from_env() -> Option<BackendConfig> {
    let base_url = std::env::var("SIGNOZ_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_SIGNOZ_BASE_URL.to_string());

    let auth = match std::env::var("SIGNOZ_API_KEY") {
        Ok(key) if !key.is_empty() => AuthMethod::ApiKey {
            header_name: "SIGNOZ-API-KEY".to_string(),
            key,
        },
        // email/password login is handled in the background thread;
        // we start with None here and upgrade after login succeeds.
        _ => AuthMethod::None,
    };

    Some(BackendConfig::SigNoz(SigNozConfig {
        base_url,
        auth,
        timeout_secs: 30,
    }))
}

/// Check whether `SIGNOZ_EMAIL` + `SIGNOZ_PASSWORD` are set.
fn login_credentials_from_env() -> Option<(String, String)> {
    let email = std::env::var("SIGNOZ_EMAIL").ok()?;
    let password = std::env::var("SIGNOZ_PASSWORD").ok()?;
    if email.is_empty() || password.is_empty() {
        return None;
    }
    Some((email, password))
}

/// Attempt to initialise the SigNoz bridge from env vars.
///
/// Returns `true` when a valid config was found and the background runtime
/// was started (or was already running).
pub fn init_signoz_from_env() -> bool {
    // Already initialised?
    {
        let rt = SIGNOZ_RUNTIME.lock().unwrap();
        if rt.is_some() {
            return *SIGNOZ_CONFIGURED.lock().unwrap();
        }
    }

    // signoz_config_from_env always returns Some (defaults to localhost:8080)
    let config = signoz_config_from_env().unwrap();

    let login_creds = login_credentials_from_env();

    // Mark as initialised immediately (prevents double-init).
    {
        let mut rt_lock = SIGNOZ_RUNTIME.lock().unwrap();
        *rt_lock = Some(Runtime::new().expect("marker runtime"));
    }
    *SIGNOZ_CONFIGURED.lock().unwrap() = true;

    let (sender, mut receiver) = unbounded_channel::<SignozRequest>();
    *SIGNOZ_SENDER.lock().unwrap() = Some(sender);

    std::thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create SigNoz Tokio runtime");

        rt.block_on(async {
            // If email+password are provided and no API key was set, log in first.
            let final_config = match (&config, login_creds) {
                (BackendConfig::SigNoz(cfg), Some((email, password)))
                    if matches!(cfg.auth, AuthMethod::None) =>
                {
                    eprintln!("[SigNoz] Logging in as {} ...", email);
                    match signoz_login(&cfg.base_url, &email, &password).await {
                        Ok(token) => {
                            eprintln!("[SigNoz] Login succeeded, using JWT for auth");
                            BackendConfig::SigNoz(SigNozConfig {
                                base_url: cfg.base_url.clone(),
                                auth: AuthMethod::BearerToken { token },
                                timeout_secs: cfg.timeout_secs,
                            })
                        }
                        Err(e) => {
                            eprintln!("[SigNoz] Login failed: {}", e);
                            push_response(SignozResponse::HealthError(format!(
                                "Login failed: {}",
                                e
                            )));
                            *SIGNOZ_CONNECTION_STATUS.lock().unwrap() = ConnectionStatus::Error;
                            // Fall through with no auth — health check will also fail,
                            // but at least the user sees the login error.
                            config
                        }
                    }
                }
                _ => config,
            };

            let client = match create_backend(final_config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[SigNoz] Failed to create backend: {}", e);
                    push_response(SignozResponse::HealthError(format!("{}", e)));
                    *SIGNOZ_CONNECTION_STATUS.lock().unwrap() = ConnectionStatus::Error;
                    return;
                }
            };

            eprintln!("[SigNoz] Runtime started, waiting for requests...");
            while let Some(request) = receiver.recv().await {
                match request {
                    SignozRequest::HealthCheck => match client.health_check().await {
                        Ok(()) => {
                            eprintln!("[SigNoz] Health check OK");
                            *SIGNOZ_CONNECTION_STATUS.lock().unwrap() = ConnectionStatus::Connected;
                            push_response(SignozResponse::HealthOk);
                        }
                        Err(e) => {
                            eprintln!("[SigNoz] Health check failed: {}", e);
                            *SIGNOZ_CONNECTION_STATUS.lock().unwrap() = ConnectionStatus::Error;
                            push_response(SignozResponse::HealthError(format!("{}", e)));
                        }
                    },
                    SignozRequest::QueryTraces(query) => match client.query_traces(&query).await {
                        Ok(result) => {
                            eprintln!("[SigNoz] Query returned {} spans", result.items.len());
                            push_response(SignozResponse::Traces(result.items));
                        }
                        Err(e) => {
                            eprintln!("[SigNoz] Query failed: {}", e);
                            push_response(SignozResponse::TracesError(format!("{}", e)));
                        }
                    },
                }
            }
        });
    });

    eprintln!("[SigNoz] Bridge initialised");
    true
}

/// Whether a valid SigNoz config was found.
pub fn is_signoz_configured() -> bool {
    *SIGNOZ_CONFIGURED.lock().unwrap()
}

/// Current connection status (updated after health check results).
pub fn get_connection_status() -> ConnectionStatus {
    *SIGNOZ_CONNECTION_STATUS.lock().unwrap()
}

/// Send a health-check request to the background runtime.
pub fn request_health_check() {
    send_request(SignozRequest::HealthCheck);
}

/// Send a trace query request to the background runtime.
pub fn request_traces(query: TraceQuery) {
    send_request(SignozRequest::QueryTraces(query));
}

/// Drain all pending responses. Returns an empty vec when there is nothing new.
pub fn take_signoz_responses() -> Vec<SignozResponse> {
    let mut lock = PENDING_SIGNOZ_RESPONSES.lock().unwrap();
    std::mem::take(&mut *lock)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn send_request(req: SignozRequest) {
    if let Some(sender) = SIGNOZ_SENDER.lock().unwrap().as_ref() {
        let _ = sender.send(req);
    }
}

fn push_response(resp: SignozResponse) {
    PENDING_SIGNOZ_RESPONSES.lock().unwrap().push(resp);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialize tests that touch process-global environment variables.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_connection_status_default() {
        let status = ConnectionStatus::default();
        assert_eq!(status, ConnectionStatus::Unknown);
    }

    #[test]
    fn test_signoz_config_from_env_default() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_signoz_env();
        let config = signoz_config_from_env().expect("should return Some with default URL");
        match config {
            BackendConfig::SigNoz(cfg) => {
                assert_eq!(cfg.base_url, "http://localhost:8080");
                assert!(matches!(cfg.auth, AuthMethod::None));
            }
        }
        clear_signoz_env();
    }

    /// Clear all SigNoz env vars to avoid cross-test pollution.
    fn clear_signoz_env() {
        std::env::remove_var("SIGNOZ_BASE_URL");
        std::env::remove_var("SIGNOZ_API_KEY");
        std::env::remove_var("SIGNOZ_EMAIL");
        std::env::remove_var("SIGNOZ_PASSWORD");
    }

    #[test]
    fn test_signoz_config_from_env_present() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_signoz_env();
        std::env::set_var("SIGNOZ_BASE_URL", "http://localhost:3301");

        let config = signoz_config_from_env().expect("should return Some");
        match config {
            BackendConfig::SigNoz(cfg) => {
                assert_eq!(cfg.base_url, "http://localhost:3301");
                assert!(matches!(cfg.auth, AuthMethod::None));
            }
        }

        clear_signoz_env();
    }

    #[test]
    fn test_signoz_config_from_env_with_api_key() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_signoz_env();
        std::env::set_var("SIGNOZ_BASE_URL", "http://example.com");
        std::env::set_var("SIGNOZ_API_KEY", "my-secret");

        let config = signoz_config_from_env().expect("should return Some");
        match config {
            BackendConfig::SigNoz(cfg) => {
                assert_eq!(cfg.base_url, "http://example.com");
                match cfg.auth {
                    AuthMethod::ApiKey { key, .. } => assert_eq!(key, "my-secret"),
                    _ => panic!("Expected ApiKey auth"),
                }
            }
        }

        clear_signoz_env();
    }

    #[test]
    fn test_login_credentials_from_env_missing() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_signoz_env();
        assert!(login_credentials_from_env().is_none());
    }

    #[test]
    fn test_login_credentials_from_env_present() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_signoz_env();
        std::env::set_var("SIGNOZ_EMAIL", "user@example.com");
        std::env::set_var("SIGNOZ_PASSWORD", "pass123");

        let creds = login_credentials_from_env().expect("should return Some");
        assert_eq!(creds.0, "user@example.com");
        assert_eq!(creds.1, "pass123");

        clear_signoz_env();
    }

    #[test]
    fn test_login_credentials_from_env_empty() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_signoz_env();
        std::env::set_var("SIGNOZ_EMAIL", "");
        std::env::set_var("SIGNOZ_PASSWORD", "pass");
        assert!(login_credentials_from_env().is_none());

        std::env::set_var("SIGNOZ_EMAIL", "user@example.com");
        std::env::set_var("SIGNOZ_PASSWORD", "");
        assert!(login_credentials_from_env().is_none());

        clear_signoz_env();
    }

    #[test]
    fn test_take_signoz_responses_empty() {
        let responses = take_signoz_responses();
        assert!(responses.is_empty());
    }

    #[test]
    fn test_push_and_take_responses() {
        push_response(SignozResponse::HealthOk);
        push_response(SignozResponse::TracesError("oops".to_string()));

        let responses = take_signoz_responses();
        assert_eq!(responses.len(), 2);
        assert!(matches!(responses[0], SignozResponse::HealthOk));
        assert!(matches!(responses[1], SignozResponse::TracesError(_)));

        let responses2 = take_signoz_responses();
        assert!(responses2.is_empty());
    }
}
