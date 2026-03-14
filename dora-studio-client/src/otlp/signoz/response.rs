use serde::Deserialize;

/// Top-level response from SigNoz query endpoints.
#[derive(Debug, Deserialize)]
pub struct SigNozResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub data: Option<SigNozResponseData>,
    #[serde(default)]
    pub error: Option<String>,
}

/// The `data` field inside a SigNoz response.
#[derive(Debug, Deserialize)]
pub struct SigNozResponseData {
    #[serde(default)]
    pub result: Vec<SigNozResultEntry>,
    #[serde(default, rename = "newResult")]
    pub new_result: Option<SigNozNewResult>,
}

/// A single result entry with a table of rows or time series.
#[derive(Debug, Deserialize)]
pub struct SigNozResultEntry {
    #[serde(default)]
    pub query_name: Option<String>,
    #[serde(default)]
    pub series: Option<Vec<SigNozTimeSeries>>,
    #[serde(default)]
    pub list: Option<Vec<SigNozListRow>>,
}

/// Newer result format used in some SigNoz responses.
#[derive(Debug, Deserialize)]
pub struct SigNozNewResult {
    #[serde(default)]
    pub data: SigNozNewResultData,
}

/// Inner data of the new result format.
#[derive(Debug, Default, Deserialize)]
pub struct SigNozNewResultData {
    #[serde(default)]
    pub result: Vec<SigNozResultEntry>,
}

/// A time series returned for metric queries.
#[derive(Debug, Deserialize)]
pub struct SigNozTimeSeries {
    #[serde(default)]
    pub labels: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub values: Vec<SigNozTimeSeriesValue>,
}

/// A single (timestamp, value) point in a time series.
#[derive(Debug, Deserialize)]
pub struct SigNozTimeSeriesValue {
    pub timestamp: u64,
    pub value: serde_json::Value,
}

/// A row returned for list-type queries (traces, logs).
#[derive(Debug, Deserialize)]
pub struct SigNozListRow {
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// Response from the SigNoz services endpoint.
#[derive(Debug, Deserialize)]
pub struct SigNozServicesResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub data: Vec<SigNozServiceEntry>,
}

/// A single service entry from the services endpoint.
#[derive(Debug, Deserialize)]
pub struct SigNozServiceEntry {
    #[serde(rename = "serviceName")]
    pub service_name: String,
    #[serde(default, rename = "numOperations")]
    pub num_operations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signoz_response_success() {
        let json = r#"{
            "status": "success",
            "data": {
                "result": [
                    {
                        "query_name": "A",
                        "list": [
                            {
                                "timestamp": "2024-01-01T00:00:00Z",
                                "data": {"serviceName": "my-svc", "name": "GET /api"}
                            }
                        ]
                    }
                ]
            }
        }"#;

        let resp: SigNozResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "success");
        assert!(resp.data.is_some());
        let data = resp.data.unwrap();
        assert_eq!(data.result.len(), 1);
        let list = data.result[0].list.as_ref().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].data["serviceName"], "my-svc");
    }

    #[test]
    fn test_signoz_response_error() {
        let json = r#"{"status": "error", "error": "something went wrong"}"#;
        let resp: SigNozResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "error");
        assert_eq!(resp.error.as_deref(), Some("something went wrong"));
    }

    #[test]
    fn test_signoz_services_response() {
        let json = r#"{
            "status": "success",
            "data": [
                {"serviceName": "frontend", "numOperations": 12},
                {"serviceName": "backend", "numOperations": 35}
            ]
        }"#;

        let resp: SigNozServicesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].service_name, "frontend");
        assert_eq!(resp.data[0].num_operations, 12);
        assert_eq!(resp.data[1].service_name, "backend");
    }

    #[test]
    fn test_signoz_time_series() {
        let json = r#"{
            "query_name": "A",
            "series": [
                {
                    "labels": {"service_name": "web"},
                    "values": [
                        {"timestamp": 1700000000, "value": 42.5},
                        {"timestamp": 1700000060, "value": 43.1}
                    ]
                }
            ]
        }"#;

        let entry: SigNozResultEntry = serde_json::from_str(json).unwrap();
        let series = entry.series.as_ref().unwrap();
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].labels.get("service_name").unwrap(), "web");
        assert_eq!(series[0].values.len(), 2);
    }

    #[test]
    fn test_signoz_response_empty_data() {
        let json = r#"{"status": "success", "data": {"result": []}}"#;
        let resp: SigNozResponse = serde_json::from_str(json).unwrap();
        assert!(resp.data.unwrap().result.is_empty());
    }
}
