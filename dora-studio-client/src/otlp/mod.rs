pub mod backend;
pub mod bridge;
pub mod config;
pub mod error;
pub mod signoz;
pub mod types;

pub use bridge::{
    get_connection_status, init_signoz_from_env, is_signoz_configured, request_health_check,
    request_traces, take_signoz_responses, ConnectionStatus, SignozResponse,
};
pub use config::{AuthMethod, BackendConfig, SigNozConfig};
pub use error::OtlpError;
pub use signoz::SigNozBackend;
pub use types::*;

use backend::TelemetryBackend;

/// Enum-dispatch wrapper over concrete telemetry backends.
///
/// Each variant delegates to the underlying backend's `TelemetryBackend` impl.
/// This avoids pulling in `async-trait` as a dependency.
pub enum TelemetryClient {
    SigNoz(SigNozBackend),
}

impl TelemetryClient {
    pub async fn health_check(&self) -> Result<(), OtlpError> {
        match self {
            TelemetryClient::SigNoz(b) => b.health_check().await,
        }
    }

    pub async fn list_services(&self) -> Result<Vec<ServiceInfo>, OtlpError> {
        match self {
            TelemetryClient::SigNoz(b) => b.list_services().await,
        }
    }

    pub async fn query_traces(&self, query: &TraceQuery) -> Result<QueryResult<Span>, OtlpError> {
        match self {
            TelemetryClient::SigNoz(b) => b.query_traces(query).await,
        }
    }

    pub async fn query_metrics(
        &self,
        query: &MetricQuery,
    ) -> Result<QueryResult<MetricSeries>, OtlpError> {
        match self {
            TelemetryClient::SigNoz(b) => b.query_metrics(query).await,
        }
    }

    pub async fn query_logs(&self, query: &LogQuery) -> Result<QueryResult<LogEntry>, OtlpError> {
        match self {
            TelemetryClient::SigNoz(b) => b.query_logs(query).await,
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            TelemetryClient::SigNoz(b) => b.display_name(),
        }
    }
}

/// Create a telemetry client from a backend configuration.
pub fn create_backend(config: BackendConfig) -> Result<TelemetryClient, OtlpError> {
    match config {
        BackendConfig::SigNoz(cfg) => {
            let backend = SigNozBackend::new(cfg)?;
            Ok(TelemetryClient::SigNoz(backend))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_backend_signoz() {
        let config = BackendConfig::SigNoz(SigNozConfig {
            base_url: "http://localhost:3301".to_string(),
            auth: AuthMethod::None,
            timeout_secs: 30,
        });
        let client = create_backend(config).unwrap();
        assert_eq!(client.display_name(), "SigNoz @ http://localhost:3301");
    }

    #[test]
    fn test_create_backend_invalid_config() {
        let config = BackendConfig::SigNoz(SigNozConfig {
            base_url: "".to_string(),
            auth: AuthMethod::None,
            timeout_secs: 30,
        });
        assert!(create_backend(config).is_err());
    }
}
