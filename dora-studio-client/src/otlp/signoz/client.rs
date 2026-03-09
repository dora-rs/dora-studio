use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::otlp::backend::TelemetryBackend;
use crate::otlp::config::{AuthMethod, SigNozConfig};
use crate::otlp::error::OtlpError;
use crate::otlp::types::*;

use super::query::{build_log_query, build_metric_query, build_trace_query};
use super::response::*;

/// A SigNoz backend client.
pub struct SigNozBackend {
    config: SigNozConfig,
    client: reqwest::Client,
}

impl SigNozBackend {
    /// Create a new `SigNozBackend` from configuration.
    pub fn new(config: SigNozConfig) -> Result<Self, OtlpError> {
        if config.base_url.is_empty() {
            return Err(OtlpError::ConnectionFailed(
                "base_url must not be empty".to_string(),
            ));
        }

        let mut default_headers = HeaderMap::new();
        default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        match &config.auth {
            AuthMethod::ApiKey { header_name, key } => {
                let name = HeaderName::try_from(header_name.as_str()).map_err(|e| {
                    OtlpError::ConnectionFailed(format!("invalid auth header name: {}", e))
                })?;
                let val = HeaderValue::from_str(key).map_err(|e| {
                    OtlpError::ConnectionFailed(format!("invalid auth header value: {}", e))
                })?;
                default_headers.insert(name, val);
            }
            AuthMethod::BearerToken { token } => {
                let val = HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| {
                    OtlpError::ConnectionFailed(format!("invalid bearer token: {}", e))
                })?;
                default_headers.insert("Authorization", val);
            }
            AuthMethod::None => {}
        }

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| {
                OtlpError::ConnectionFailed(format!("failed to build HTTP client: {}", e))
            })?;

        Ok(Self { config, client })
    }

    /// Build the full URL for a given path.
    fn url(&self, path: &str) -> String {
        let base = self.config.base_url.trim_end_matches('/');
        format!("{}{}", base, path)
    }

    /// Send a GET request and deserialize the response.
    async fn get_request<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, OtlpError> {
        let url = self.url(path);
        let resp = self.client.get(&url).send().await?;
        let status = resp.status();

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(OtlpError::AuthenticationFailed(format!(
                "HTTP {}",
                status.as_u16()
            )));
        }

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OtlpError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        let body = resp.text().await?;
        serde_json::from_str(&body).map_err(OtlpError::from)
    }

    /// Send a POST request with a JSON body and return the raw response text.
    async fn post_request(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<String, OtlpError> {
        let url = self.url(path);
        let resp = self.client.post(&url).json(body).send().await?;
        let status = resp.status();

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(OtlpError::AuthenticationFailed(format!(
                "HTTP {}",
                status.as_u16()
            )));
        }

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(OtlpError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }

        resp.text().await.map_err(OtlpError::from)
    }

    /// Send a composite query and parse the SigNoz response wrapper.
    async fn send_query(&self, payload: &serde_json::Value) -> Result<SigNozResponse, OtlpError> {
        let text = self.post_request("/api/v3/query_range", payload).await?;
        let resp: SigNozResponse = serde_json::from_str(&text)?;

        if resp.status == "error" {
            return Err(OtlpError::Backend(
                resp.error.unwrap_or_else(|| "unknown error".to_string()),
            ));
        }

        Ok(resp)
    }

    /// Extract result entries from the SigNoz response, handling both old and new formats.
    fn extract_result_entries(resp: &SigNozResponse) -> &[SigNozResultEntry] {
        if let Some(ref data) = resp.data {
            if let Some(ref new_result) = data.new_result {
                return &new_result.data.result;
            }
            return &data.result;
        }
        &[]
    }

    /// Parse list-type results into `Span` values.
    fn parse_trace_results(resp: &SigNozResponse) -> Vec<Span> {
        let entries = Self::extract_result_entries(resp);
        let mut spans = Vec::new();

        for entry in entries {
            if let Some(ref list) = entry.list {
                for row in list {
                    let data = &row.data;
                    let span = Span {
                        trace_id: json_str(data, "traceID"),
                        span_id: json_str(data, "spanID"),
                        parent_span_id: data
                            .get("parentSpanID")
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.is_empty())
                            .map(String::from),
                        service_name: json_str(data, "serviceName"),
                        operation_name: json_str(data, "name"),
                        start_time_ms: data
                            .get("timestamp")
                            .and_then(parse_timestamp)
                            .or_else(|| row.timestamp.as_deref().and_then(parse_iso8601_to_ms))
                            .unwrap_or(0),
                        duration_ms: data
                            .get("durationNano")
                            .and_then(|v| v.as_f64())
                            .map(|n| (n / 1_000_000.0) as u64)
                            .unwrap_or(0),
                        status_code: data.get("statusCode").and_then(|v| v.as_i64()).unwrap_or(0)
                            as i32,
                        has_error: data
                            .get("hasError")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                        attributes: extract_string_map(data),
                    };
                    spans.push(span);
                }
            }
        }
        spans
    }

    /// Parse list-type results into `LogEntry` values.
    fn parse_log_results(resp: &SigNozResponse) -> Vec<LogEntry> {
        let entries = Self::extract_result_entries(resp);
        let mut logs = Vec::new();

        for entry in entries {
            if let Some(ref list) = entry.list {
                for row in list {
                    let data = &row.data;
                    let log = LogEntry {
                        timestamp_ms: data
                            .get("timestamp")
                            .and_then(parse_timestamp)
                            .or_else(|| row.timestamp.as_deref().and_then(parse_iso8601_to_ms))
                            .unwrap_or(0),
                        severity: json_str(data, "severity_text"),
                        body: json_str(data, "body"),
                        service_name: json_str(data, "service_name"),
                        attributes: extract_string_map(data),
                    };
                    logs.push(log);
                }
            }
        }
        logs
    }

    /// Parse time-series results into `MetricSeries` values.
    fn parse_metric_results(resp: &SigNozResponse) -> Vec<MetricSeries> {
        let entries = Self::extract_result_entries(resp);
        let mut metrics = Vec::new();

        for entry in entries {
            if let Some(ref series_list) = entry.series {
                for ts in series_list {
                    let points: Vec<MetricPoint> = ts
                        .values
                        .iter()
                        .map(|v| MetricPoint {
                            timestamp_ms: v.timestamp,
                            value: v.value.as_f64().unwrap_or(0.0),
                        })
                        .collect();

                    let metric = MetricSeries {
                        metric_name: ts.labels.get("__name__").cloned().unwrap_or_default(),
                        service_name: ts.labels.get("service_name").cloned().unwrap_or_default(),
                        labels: ts.labels.clone(),
                        points,
                    };
                    metrics.push(metric);
                }
            }
        }
        metrics
    }
}

impl TelemetryBackend for SigNozBackend {
    async fn health_check(&self) -> Result<(), OtlpError> {
        let url = self.url("/api/v1/health");
        let resp = self.client.get(&url).send().await?;
        let status = resp.status();

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(OtlpError::AuthenticationFailed(format!(
                "HTTP {}",
                status.as_u16()
            )));
        }

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OtlpError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        Ok(())
    }

    async fn list_services(&self) -> Result<Vec<ServiceInfo>, OtlpError> {
        let resp: SigNozServicesResponse = self.get_request("/api/v1/services").await?;
        Ok(resp
            .data
            .into_iter()
            .map(|e| ServiceInfo {
                name: e.service_name,
                num_operations: e.num_operations,
            })
            .collect())
    }

    async fn query_traces(&self, query: &TraceQuery) -> Result<QueryResult<Span>, OtlpError> {
        let payload = build_trace_query(query);
        let resp = self.send_query(&payload).await?;
        let items = Self::parse_trace_results(&resp);
        Ok(QueryResult {
            total: Some(items.len() as u64),
            items,
        })
    }

    async fn query_metrics(
        &self,
        query: &MetricQuery,
    ) -> Result<QueryResult<MetricSeries>, OtlpError> {
        let payload = build_metric_query(query);
        let resp = self.send_query(&payload).await?;
        let items = Self::parse_metric_results(&resp);
        Ok(QueryResult {
            total: Some(items.len() as u64),
            items,
        })
    }

    async fn query_logs(&self, query: &LogQuery) -> Result<QueryResult<LogEntry>, OtlpError> {
        let payload = build_log_query(query);
        let resp = self.send_query(&payload).await?;
        let items = Self::parse_log_results(&resp);
        Ok(QueryResult {
            total: Some(items.len() as u64),
            items,
        })
    }

    fn display_name(&self) -> String {
        format!("SigNoz @ {}", self.config.base_url)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn json_str(map: &HashMap<String, serde_json::Value>, key: &str) -> String {
    map.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn parse_timestamp(v: &serde_json::Value) -> Option<u64> {
    if let Some(n) = v.as_u64() {
        // If the value looks like nanoseconds (> 1e15), convert to ms
        if n > 1_000_000_000_000_000 {
            Some(n / 1_000_000)
        } else if n > 1_000_000_000_000 {
            // Already milliseconds
            Some(n)
        } else {
            // Seconds
            Some(n * 1000)
        }
    } else if let Some(s) = v.as_str() {
        s.parse::<u64>().ok().map(|n| {
            if n > 1_000_000_000_000_000 {
                n / 1_000_000
            } else if n > 1_000_000_000_000 {
                n
            } else {
                n * 1000
            }
        })
    } else {
        None
    }
}

/// Parse an ISO 8601 / RFC 3339 timestamp string to milliseconds since epoch.
/// Handles formats like "2026-02-02T19:40:37.126981Z" and "2026-02-02T19:40:37Z".
fn parse_iso8601_to_ms(s: &str) -> Option<u64> {
    // Expected: "YYYY-MM-DDTHH:MM:SS[.frac]Z"
    let s = s.trim();
    let (date_part, time_part) = s.split_once('T')?;
    // Handle +00:00 offset
    let time_part = time_part
        .strip_suffix('Z')
        .or_else(|| time_part.strip_suffix("+00:00"))
        .or(Some(time_part))?;

    let mut date_iter = date_part.splitn(3, '-');
    let year: i64 = date_iter.next()?.parse().ok()?;
    let month: i64 = date_iter.next()?.parse().ok()?;
    let day: i64 = date_iter.next()?.parse().ok()?;

    let (time_hms, frac_str) = if let Some((hms, frac)) = time_part.split_once('.') {
        (hms, frac)
    } else {
        (time_part, "0")
    };

    let mut time_iter = time_hms.splitn(3, ':');
    let hour: i64 = time_iter.next()?.parse().ok()?;
    let minute: i64 = time_iter.next()?.parse().ok()?;
    let second: i64 = time_iter.next()?.parse().ok()?;

    // Parse fractional seconds to milliseconds
    let frac_ms: u64 = if frac_str.len() >= 3 {
        frac_str[..3].parse().unwrap_or(0)
    } else {
        let padded = format!("{:0<3}", frac_str);
        padded.parse().unwrap_or(0)
    };

    // Days from epoch (1970-01-01) using a simplified calculation
    let days = days_from_civil(year, month, day);
    let total_secs = days * 86400 + hour * 3600 + minute * 60 + second;

    if total_secs < 0 {
        return None;
    }

    Some(total_secs as u64 * 1000 + frac_ms)
}

/// Convert a civil date to days since 1970-01-01 (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) as u64 + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

fn extract_string_map(data: &HashMap<String, serde_json::Value>) -> HashMap<String, String> {
    data.iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otlp::config::AuthMethod;

    #[test]
    fn test_new_signoz_backend_empty_url() {
        let config = SigNozConfig {
            base_url: "".to_string(),
            auth: AuthMethod::None,
            timeout_secs: 30,
        };
        let result = SigNozBackend::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_signoz_backend_valid_config() {
        let config = SigNozConfig {
            base_url: "http://localhost:3301".to_string(),
            auth: AuthMethod::None,
            timeout_secs: 30,
        };
        let backend = SigNozBackend::new(config).unwrap();
        assert_eq!(backend.display_name(), "SigNoz @ http://localhost:3301");
    }

    #[test]
    fn test_new_signoz_backend_with_api_key() {
        let config = SigNozConfig {
            base_url: "http://signoz.example.com".to_string(),
            auth: AuthMethod::ApiKey {
                header_name: "SIGNOZ-API-KEY".to_string(),
                key: "test-key-123".to_string(),
            },
            timeout_secs: 60,
        };
        let backend = SigNozBackend::new(config);
        assert!(backend.is_ok());
    }

    #[test]
    fn test_new_signoz_backend_with_bearer_token() {
        let config = SigNozConfig {
            base_url: "http://signoz.example.com".to_string(),
            auth: AuthMethod::BearerToken {
                token: "my-token".to_string(),
            },
            timeout_secs: 30,
        };
        let backend = SigNozBackend::new(config);
        assert!(backend.is_ok());
    }

    #[test]
    fn test_url_building() {
        let config = SigNozConfig {
            base_url: "http://localhost:3301/".to_string(),
            auth: AuthMethod::None,
            timeout_secs: 30,
        };
        let backend = SigNozBackend::new(config).unwrap();
        assert_eq!(
            backend.url("/api/v1/health"),
            "http://localhost:3301/api/v1/health"
        );
    }

    #[test]
    fn test_parse_trace_results() {
        let resp = SigNozResponse {
            status: "success".to_string(),
            data: Some(SigNozResponseData {
                result: vec![SigNozResultEntry {
                    query_name: Some("A".to_string()),
                    series: None,
                    list: Some(vec![SigNozListRow {
                        timestamp: Some("1700000000000".to_string()),
                        data: HashMap::from([
                            ("traceID".to_string(), serde_json::json!("trace-1")),
                            ("spanID".to_string(), serde_json::json!("span-1")),
                            ("serviceName".to_string(), serde_json::json!("web")),
                            ("name".to_string(), serde_json::json!("GET /api")),
                            ("durationNano".to_string(), serde_json::json!(150_000_000.0)),
                            ("statusCode".to_string(), serde_json::json!(0)),
                            ("hasError".to_string(), serde_json::json!(false)),
                        ]),
                    }]),
                }],
                new_result: None,
            }),
            error: None,
        };

        let spans = SigNozBackend::parse_trace_results(&resp);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].trace_id, "trace-1");
        assert_eq!(spans[0].operation_name, "GET /api");
        assert_eq!(spans[0].duration_ms, 150);
        assert!(!spans[0].has_error);
    }

    #[test]
    fn test_parse_log_results() {
        let resp = SigNozResponse {
            status: "success".to_string(),
            data: Some(SigNozResponseData {
                result: vec![SigNozResultEntry {
                    query_name: Some("A".to_string()),
                    series: None,
                    list: Some(vec![SigNozListRow {
                        timestamp: Some("1700000000000".to_string()),
                        data: HashMap::from([
                            ("severity_text".to_string(), serde_json::json!("ERROR")),
                            ("body".to_string(), serde_json::json!("connection timeout")),
                            ("service_name".to_string(), serde_json::json!("backend")),
                        ]),
                    }]),
                }],
                new_result: None,
            }),
            error: None,
        };

        let logs = SigNozBackend::parse_log_results(&resp);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].severity, "ERROR");
        assert_eq!(logs[0].body, "connection timeout");
        assert_eq!(logs[0].service_name, "backend");
    }

    #[test]
    fn test_parse_metric_results() {
        let resp = SigNozResponse {
            status: "success".to_string(),
            data: Some(SigNozResponseData {
                result: vec![SigNozResultEntry {
                    query_name: Some("A".to_string()),
                    series: Some(vec![SigNozTimeSeries {
                        labels: HashMap::from([
                            ("__name__".to_string(), "http_requests_total".to_string()),
                            ("service_name".to_string(), "web".to_string()),
                        ]),
                        values: vec![
                            SigNozTimeSeriesValue {
                                timestamp: 1700000000000,
                                value: serde_json::json!(42.5),
                            },
                            SigNozTimeSeriesValue {
                                timestamp: 1700000060000,
                                value: serde_json::json!(43.1),
                            },
                        ],
                    }]),
                    list: None,
                }],
                new_result: None,
            }),
            error: None,
        };

        let metrics = SigNozBackend::parse_metric_results(&resp);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].metric_name, "http_requests_total");
        assert_eq!(metrics[0].service_name, "web");
        assert_eq!(metrics[0].points.len(), 2);
        assert!((metrics[0].points[0].value - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_timestamp_nanoseconds() {
        let val = serde_json::json!(1700000000000000000u64);
        assert_eq!(parse_timestamp(&val), Some(1700000000000));
    }

    #[test]
    fn test_parse_timestamp_milliseconds() {
        let val = serde_json::json!(1700000000000u64);
        assert_eq!(parse_timestamp(&val), Some(1700000000000));
    }

    #[test]
    fn test_parse_timestamp_seconds() {
        let val = serde_json::json!(1700000000u64);
        assert_eq!(parse_timestamp(&val), Some(1700000000000));
    }

    #[test]
    fn test_parse_timestamp_string() {
        let val = serde_json::json!("1700000000000000000");
        assert_eq!(parse_timestamp(&val), Some(1700000000000));
    }

    #[test]
    fn test_parse_iso8601_basic() {
        assert_eq!(
            parse_iso8601_to_ms("2026-02-02T19:40:37.126981Z"),
            Some(1770061237126)
        );
    }

    #[test]
    fn test_parse_iso8601_no_frac() {
        assert_eq!(
            parse_iso8601_to_ms("2026-02-02T19:40:37Z"),
            Some(1770061237000)
        );
    }

    #[test]
    fn test_parse_iso8601_epoch() {
        assert_eq!(parse_iso8601_to_ms("1970-01-01T00:00:00Z"), Some(0));
    }

    #[test]
    fn test_extract_string_map() {
        let data = HashMap::from([
            ("key1".to_string(), serde_json::json!("val1")),
            ("key2".to_string(), serde_json::json!(42)),
            ("key3".to_string(), serde_json::json!("val3")),
        ]);
        let result = extract_string_map(&data);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("key1").unwrap(), "val1");
        assert_eq!(result.get("key3").unwrap(), "val3");
    }
}
