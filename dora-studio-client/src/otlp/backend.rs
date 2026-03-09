use crate::otlp::error::OtlpError;
use crate::otlp::types::*;

/// Contract for read-only telemetry backends.
///
/// Concrete backends implement this trait directly. The `TelemetryClient` enum
/// in `mod.rs` dispatches to them, avoiding the need for `async-trait`.
pub trait TelemetryBackend {
    /// Check that the backend is reachable and authenticated.
    fn health_check(&self) -> impl std::future::Future<Output = Result<(), OtlpError>> + Send;

    /// List services known to the backend.
    fn list_services(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<ServiceInfo>, OtlpError>> + Send;

    /// Query trace spans.
    fn query_traces(
        &self,
        query: &TraceQuery,
    ) -> impl std::future::Future<Output = Result<QueryResult<Span>, OtlpError>> + Send;

    /// Query metric time series.
    fn query_metrics(
        &self,
        query: &MetricQuery,
    ) -> impl std::future::Future<Output = Result<QueryResult<MetricSeries>, OtlpError>> + Send;

    /// Query log entries.
    fn query_logs(
        &self,
        query: &LogQuery,
    ) -> impl std::future::Future<Output = Result<QueryResult<LogEntry>, OtlpError>> + Send;

    /// Human-readable name of this backend (e.g. "SigNoz @ http://localhost:3301").
    fn display_name(&self) -> String;
}
