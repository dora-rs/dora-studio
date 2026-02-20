use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A time range specified in milliseconds since epoch.
///
/// Used for filtering traces, logs, and metrics by time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time in milliseconds since epoch
    pub start_ms: u64,
    /// End time in milliseconds since epoch
    pub end_ms: u64,
}

/// A single trace span.
///
/// Represents an operation within a trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// The unique identifier of the trace this span belongs to
    pub trace_id: String,
    /// The unique identifier of this span
    pub span_id: String,
    /// The ID of the parent span, if any
    pub parent_span_id: Option<String>,
    /// The name of the service that generated this span
    pub service_name: String,
    /// The name of the operation represented by this span
    pub operation_name: String,
    /// The start time of the span in milliseconds since epoch
    pub start_time_ms: u64,
    /// The duration of the span in milliseconds
    pub duration_ms: u64,
    /// The status code of the span (0=Unset, 1=Ok, 2=Error)
    pub status_code: i32,
    /// Whether the span represents an error
    pub has_error: bool,
    /// Key-value attributes associated with the span
    pub attributes: HashMap<String, String>,
}

/// A single log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// The time the log was created in milliseconds since epoch
    pub timestamp_ms: u64,
    /// The severity level of the log (e.g., INFO, ERROR)
    pub severity: String,
    /// The log message body
    pub body: String,
    /// The name of the service that generated this log
    pub service_name: String,
    /// Key-value attributes associated with the log
    pub attributes: HashMap<String, String>,
}

/// A single point in a metric time series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// The time of the data point in milliseconds since epoch
    pub timestamp_ms: u64,
    /// The value of the metric at this time
    pub value: f64,
}

/// A metric time series with its labels and data points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    /// The name of the metric
    pub metric_name: String,
    /// The name of the service that generated this metric
    pub service_name: String,
    /// Labels associated with this time series
    pub labels: HashMap<String, String>,
    /// The list of data points in this series
    pub points: Vec<MetricPoint>,
}

/// Information about a service discovered in the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// The name of the service
    pub name: String,
    /// The count of operations or traces associated with this service
    pub num_operations: u64,
}

/// Query parameters for trace queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceQuery {
    /// Filter by service name
    pub service_name: Option<String>,
    /// Filter by operation name
    pub operation_name: Option<String>,
    /// Minimum span duration in milliseconds
    pub min_duration_ms: Option<u64>,
    /// Maximum span duration in milliseconds
    pub max_duration_ms: Option<u64>,
    /// Time range for the query
    pub time_range: Option<TimeRange>,
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Filter by specific tags
    pub tags: HashMap<String, String>,
}

/// Query parameters for metric queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricQuery {
    /// Filter by metric name
    pub metric_name: Option<String>,
    /// Filter by service name
    pub service_name: Option<String>,
    /// Time range for the query
    pub time_range: Option<TimeRange>,
    /// Step size in seconds for the graph
    pub step_seconds: Option<u64>,
    /// Aggregation function (e.g., "AVG", "MAX")
    pub aggregation: Option<String>,
    /// Group results by these labels
    pub group_by: Vec<String>,
    /// Filter by specific label values
    pub filters: HashMap<String, String>,
}

/// Query parameters for log queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogQuery {
    /// Filter by service name
    pub service_name: Option<String>,
    /// Filter by severity (e.g., "ERROR")
    pub severity: Option<String>,
    /// Filter by log body content
    pub body_contains: Option<String>,
    /// Time range for the query
    pub time_range: Option<TimeRange>,
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Filter by specific attributes
    pub attributes: HashMap<String, String>,
}

/// A paginated query result.
///
/// Wraps a list of items with the total count available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    /// The items returned in this page
    pub items: Vec<T>,
    /// Total number of items matching the query (if available)
    pub total: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_query_default() {
        let q = TraceQuery::default();
        assert!(q.service_name.is_none());
        assert!(q.operation_name.is_none());
        assert!(q.min_duration_ms.is_none());
        assert!(q.limit.is_none());
        assert!(q.tags.is_empty());
    }

    #[test]
    fn test_metric_query_default() {
        let q = MetricQuery::default();
        assert!(q.metric_name.is_none());
        assert!(q.group_by.is_empty());
        assert!(q.filters.is_empty());
    }

    #[test]
    fn test_log_query_default() {
        let q = LogQuery::default();
        assert!(q.service_name.is_none());
        assert!(q.severity.is_none());
        assert!(q.attributes.is_empty());
    }

    #[test]
    fn test_span_serialization_roundtrip() {
        let span = Span {
            trace_id: "abc123".to_string(),
            span_id: "span1".to_string(),
            parent_span_id: Some("parent1".to_string()),
            service_name: "my-service".to_string(),
            operation_name: "GET /api".to_string(),
            start_time_ms: 1700000000000,
            duration_ms: 150,
            status_code: 0,
            has_error: false,
            attributes: HashMap::from([("http.method".to_string(), "GET".to_string())]),
        };

        let json = serde_json::to_string(&span).unwrap();
        let deserialized: Span = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.trace_id, "abc123");
        assert_eq!(deserialized.duration_ms, 150);
        assert_eq!(deserialized.attributes.get("http.method").unwrap(), "GET");
    }

    #[test]
    fn test_log_entry_serialization_roundtrip() {
        let entry = LogEntry {
            timestamp_ms: 1700000000000,
            severity: "ERROR".to_string(),
            body: "something went wrong".to_string(),
            service_name: "my-service".to_string(),
            attributes: HashMap::new(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: LogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.severity, "ERROR");
        assert_eq!(deserialized.body, "something went wrong");
    }

    #[test]
    fn test_query_result_serialization() {
        let result = QueryResult {
            items: vec![ServiceInfo {
                name: "svc".to_string(),
                num_operations: 5,
            }],
            total: Some(1),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: QueryResult<ServiceInfo> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.items[0].name, "svc");
        assert_eq!(deserialized.total, Some(1));
    }
}
