use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A time range specified in milliseconds since epoch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_ms: u64,
    pub end_ms: u64,
}

/// A single trace span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub service_name: String,
    pub operation_name: String,
    pub start_time_ms: u64,
    pub duration_ms: u64,
    pub status_code: i32,
    pub has_error: bool,
    pub attributes: HashMap<String, String>,
}

/// A single log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp_ms: u64,
    pub severity: String,
    pub body: String,
    pub service_name: String,
    pub attributes: HashMap<String, String>,
}

/// A single point in a metric time series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp_ms: u64,
    pub value: f64,
}

/// A metric time series with its labels and data points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    pub metric_name: String,
    pub service_name: String,
    pub labels: HashMap<String, String>,
    pub points: Vec<MetricPoint>,
}

/// Information about a service discovered in the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub num_operations: u64,
}

/// Query parameters for trace queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceQuery {
    pub service_name: Option<String>,
    pub operation_name: Option<String>,
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub time_range: Option<TimeRange>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub tags: HashMap<String, String>,
}

/// Query parameters for metric queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricQuery {
    pub metric_name: Option<String>,
    pub service_name: Option<String>,
    pub time_range: Option<TimeRange>,
    pub step_seconds: Option<u64>,
    pub aggregation: Option<String>,
    pub group_by: Vec<String>,
    pub filters: HashMap<String, String>,
}

/// Query parameters for log queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogQuery {
    pub service_name: Option<String>,
    pub severity: Option<String>,
    pub body_contains: Option<String>,
    pub time_range: Option<TimeRange>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub attributes: HashMap<String, String>,
}

/// A paginated query result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub items: Vec<T>,
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
