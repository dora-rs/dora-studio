use crate::otlp::types::{LogQuery, MetricQuery, TimeRange, TraceQuery};

/// Default time range: last 1 hour.
fn default_time_range() -> TimeRange {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    TimeRange {
        start_ms: now_ms.saturating_sub(3_600_000),
        end_ms: now_ms,
    }
}

/// Build the JSON payload for a SigNoz `/api/v3/query_range` trace query.
pub fn build_trace_query(query: &TraceQuery) -> serde_json::Value {
    let tr = query.time_range.clone().unwrap_or_else(default_time_range);
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    let mut filters = Vec::new();

    if let Some(ref svc) = query.service_name {
        filters.push(serde_json::json!({
            "key": {"key": "serviceName", "dataType": "string", "type": "tag", "isColumn": true},
            "op": "=",
            "value": svc
        }));
    }

    if let Some(ref op) = query.operation_name {
        filters.push(serde_json::json!({
            "key": {"key": "name", "dataType": "string", "type": "tag", "isColumn": true},
            "op": "=",
            "value": op
        }));
    }

    if let Some(min_dur) = query.min_duration_ms {
        filters.push(serde_json::json!({
            "key": {"key": "durationNano", "dataType": "float64", "type": "tag", "isColumn": true},
            "op": ">=",
            "value": min_dur * 1_000_000
        }));
    }

    if let Some(max_dur) = query.max_duration_ms {
        filters.push(serde_json::json!({
            "key": {"key": "durationNano", "dataType": "float64", "type": "tag", "isColumn": true},
            "op": "<=",
            "value": max_dur * 1_000_000
        }));
    }

    for (k, v) in &query.tags {
        filters.push(serde_json::json!({
            "key": {"key": k, "dataType": "string", "type": "tag", "isColumn": false},
            "op": "=",
            "value": v
        }));
    }

    serde_json::json!({
        "start": tr.start_ms * 1_000_000,
        "end": tr.end_ms * 1_000_000,
        "compositeQuery": {
            "queryType": "builder",
            "panelType": "list",
            "builderQueries": {
                "A": {
                    "dataSource": "traces",
                    "queryName": "A",
                    "expression": "A",
                    "aggregateOperator": "noop",
                    "aggregateAttribute": {},
                    "filters": {
                        "op": "AND",
                        "items": filters
                    },
                    "limit": limit,
                    "offset": offset,
                    "orderBy": [{"columnName": "timestamp", "order": "desc"}],
                    "selectColumns": [
                        {"key": "serviceName", "dataType": "string", "type": "tag", "isColumn": true},
                        {"key": "name", "dataType": "string", "type": "tag", "isColumn": true},
                        {"key": "durationNano", "dataType": "float64", "type": "tag", "isColumn": true},
                        {"key": "traceID", "dataType": "string", "type": "tag", "isColumn": true},
                        {"key": "spanID", "dataType": "string", "type": "tag", "isColumn": true},
                        {"key": "parentSpanID", "dataType": "string", "type": "tag", "isColumn": true},
                        {"key": "statusCode", "dataType": "int64", "type": "tag", "isColumn": true},
                        {"key": "hasError", "dataType": "bool", "type": "tag", "isColumn": true}
                    ]
                }
            }
        }
    })
}

/// Build the JSON payload for a SigNoz `/api/v3/query_range` log query.
pub fn build_log_query(query: &LogQuery) -> serde_json::Value {
    let tr = query.time_range.clone().unwrap_or_else(default_time_range);
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    let mut filters = Vec::new();

    if let Some(ref svc) = query.service_name {
        filters.push(serde_json::json!({
            "key": {"key": "service_name", "dataType": "string", "type": "resource", "isColumn": true},
            "op": "=",
            "value": svc
        }));
    }

    if let Some(ref severity) = query.severity {
        filters.push(serde_json::json!({
            "key": {"key": "severity_text", "dataType": "string", "type": "tag", "isColumn": true},
            "op": "=",
            "value": severity
        }));
    }

    if let Some(ref body_contains) = query.body_contains {
        filters.push(serde_json::json!({
            "key": {"key": "body", "dataType": "string", "type": "tag", "isColumn": true},
            "op": "contains",
            "value": body_contains
        }));
    }

    for (k, v) in &query.attributes {
        filters.push(serde_json::json!({
            "key": {"key": k, "dataType": "string", "type": "tag", "isColumn": false},
            "op": "=",
            "value": v
        }));
    }

    serde_json::json!({
        "start": tr.start_ms * 1_000_000,
        "end": tr.end_ms * 1_000_000,
        "compositeQuery": {
            "queryType": "builder",
            "panelType": "list",
            "builderQueries": {
                "A": {
                    "dataSource": "logs",
                    "queryName": "A",
                    "expression": "A",
                    "aggregateOperator": "noop",
                    "aggregateAttribute": {},
                    "filters": {
                        "op": "AND",
                        "items": filters
                    },
                    "limit": limit,
                    "offset": offset,
                    "orderBy": [{"columnName": "timestamp", "order": "desc"}],
                    "selectColumns": [
                        {"key": "service_name", "dataType": "string", "type": "resource", "isColumn": true},
                        {"key": "severity_text", "dataType": "string", "type": "tag", "isColumn": true},
                        {"key": "body", "dataType": "string", "type": "tag", "isColumn": true}
                    ]
                }
            }
        }
    })
}

/// Build the JSON payload for a SigNoz `/api/v3/query_range` metric query.
pub fn build_metric_query(query: &MetricQuery) -> serde_json::Value {
    let tr = query.time_range.clone().unwrap_or_else(default_time_range);
    let step = query.step_seconds.unwrap_or(60);
    let aggregation = query.aggregation.as_deref().unwrap_or("avg");

    let metric_name = query.metric_name.as_deref().unwrap_or("signoz_calls_total");

    let mut filters = Vec::new();

    if let Some(ref svc) = query.service_name {
        filters.push(serde_json::json!({
            "key": {"key": "service_name", "dataType": "string", "type": "resource", "isColumn": false},
            "op": "=",
            "value": svc
        }));
    }

    for (k, v) in &query.filters {
        filters.push(serde_json::json!({
            "key": {"key": k, "dataType": "string", "type": "tag", "isColumn": false},
            "op": "=",
            "value": v
        }));
    }

    let group_by: Vec<serde_json::Value> = query
        .group_by
        .iter()
        .map(|g| {
            serde_json::json!({
                "key": g,
                "dataType": "string",
                "type": "tag",
                "isColumn": false
            })
        })
        .collect();

    serde_json::json!({
        "start": tr.start_ms * 1_000_000,
        "end": tr.end_ms * 1_000_000,
        "step": step,
        "compositeQuery": {
            "queryType": "builder",
            "panelType": "time_series",
            "builderQueries": {
                "A": {
                    "dataSource": "metrics",
                    "queryName": "A",
                    "expression": "A",
                    "aggregateOperator": aggregation,
                    "aggregateAttribute": {
                        "key": metric_name,
                        "dataType": "float64",
                        "type": "Sum",
                        "isColumn": true,
                        "isMonotonic": true
                    },
                    "filters": {
                        "op": "AND",
                        "items": filters
                    },
                    "groupBy": group_by,
                    "orderBy": []
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otlp::types::TimeRange;
    use std::collections::HashMap;

    #[test]
    fn test_build_trace_query_minimal() {
        let query = TraceQuery::default();
        let payload = build_trace_query(&query);

        let cq = &payload["compositeQuery"];
        assert_eq!(cq["queryType"], "builder");
        assert_eq!(cq["panelType"], "list");

        let bq = &cq["builderQueries"]["A"];
        assert_eq!(bq["dataSource"], "traces");
        assert_eq!(bq["limit"], 100);
    }

    #[test]
    fn test_build_trace_query_with_filters() {
        let query = TraceQuery {
            service_name: Some("my-service".to_string()),
            operation_name: Some("GET /api".to_string()),
            min_duration_ms: Some(100),
            time_range: Some(TimeRange {
                start_ms: 1000,
                end_ms: 2000,
            }),
            limit: Some(50),
            ..Default::default()
        };

        let payload = build_trace_query(&query);
        assert_eq!(payload["start"], 1000 * 1_000_000u64);
        assert_eq!(payload["end"], 2000 * 1_000_000u64);

        let filters = &payload["compositeQuery"]["builderQueries"]["A"]["filters"]["items"];
        let filters = filters.as_array().unwrap();
        assert_eq!(filters.len(), 3); // service, operation, min_duration
    }

    #[test]
    fn test_build_trace_query_with_tags() {
        let mut tags = HashMap::new();
        tags.insert("http.method".to_string(), "POST".to_string());

        let query = TraceQuery {
            tags,
            time_range: Some(TimeRange {
                start_ms: 1000,
                end_ms: 2000,
            }),
            ..Default::default()
        };

        let payload = build_trace_query(&query);
        let filters = &payload["compositeQuery"]["builderQueries"]["A"]["filters"]["items"];
        let filters = filters.as_array().unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0]["value"], "POST");
    }

    #[test]
    fn test_build_log_query_minimal() {
        let query = LogQuery::default();
        let payload = build_log_query(&query);

        let bq = &payload["compositeQuery"]["builderQueries"]["A"];
        assert_eq!(bq["dataSource"], "logs");
        assert_eq!(bq["limit"], 100);
    }

    #[test]
    fn test_build_log_query_with_filters() {
        let query = LogQuery {
            service_name: Some("web-app".to_string()),
            severity: Some("ERROR".to_string()),
            body_contains: Some("timeout".to_string()),
            time_range: Some(TimeRange {
                start_ms: 1000,
                end_ms: 2000,
            }),
            ..Default::default()
        };

        let payload = build_log_query(&query);
        let filters = &payload["compositeQuery"]["builderQueries"]["A"]["filters"]["items"];
        let filters = filters.as_array().unwrap();
        assert_eq!(filters.len(), 3);
    }

    #[test]
    fn test_build_metric_query_minimal() {
        let query = MetricQuery::default();
        let payload = build_metric_query(&query);

        let bq = &payload["compositeQuery"]["builderQueries"]["A"];
        assert_eq!(bq["dataSource"], "metrics");
        assert_eq!(bq["aggregateOperator"], "avg");
        assert_eq!(payload["step"], 60);
    }

    #[test]
    fn test_build_metric_query_with_options() {
        let query = MetricQuery {
            metric_name: Some("http_requests_total".to_string()),
            service_name: Some("gateway".to_string()),
            step_seconds: Some(300),
            aggregation: Some("sum".to_string()),
            group_by: vec!["status_code".to_string()],
            time_range: Some(TimeRange {
                start_ms: 1000,
                end_ms: 2000,
            }),
            ..Default::default()
        };

        let payload = build_metric_query(&query);
        assert_eq!(payload["step"], 300);

        let bq = &payload["compositeQuery"]["builderQueries"]["A"];
        assert_eq!(bq["aggregateOperator"], "sum");
        assert_eq!(bq["aggregateAttribute"]["key"], "http_requests_total");

        let gb = bq["groupBy"].as_array().unwrap();
        assert_eq!(gb.len(), 1);
        assert_eq!(gb[0]["key"], "status_code");
    }
}
