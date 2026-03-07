use std::fmt;

/// Errors that can occur when interacting with OTLP backends.
#[derive(Debug)]
pub enum OtlpError {
    Http(reqwest::Error),
    ApiError { status: u16, message: String },
    Deserialization(serde_json::Error),
    ConnectionFailed(String),
    AuthenticationFailed(String),
    InvalidQuery(String),
    Backend(String),
}

impl fmt::Display for OtlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OtlpError::Http(e) => write!(f, "HTTP error: {}", e),
            OtlpError::ApiError { status, message } => {
                write!(f, "API error (status {}): {}", status, message)
            }
            OtlpError::Deserialization(e) => write!(f, "deserialization error: {}", e),
            OtlpError::ConnectionFailed(msg) => write!(f, "connection failed: {}", msg),
            OtlpError::AuthenticationFailed(msg) => {
                write!(f, "authentication failed: {}", msg)
            }
            OtlpError::InvalidQuery(msg) => write!(f, "invalid query: {}", msg),
            OtlpError::Backend(msg) => write!(f, "backend error: {}", msg),
        }
    }
}

impl std::error::Error for OtlpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OtlpError::Http(e) => Some(e),
            OtlpError::Deserialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for OtlpError {
    fn from(err: reqwest::Error) -> Self {
        OtlpError::Http(err)
    }
}

impl From<serde_json::Error> for OtlpError {
    fn from(err: serde_json::Error) -> Self {
        OtlpError::Deserialization(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_api_error() {
        let err = OtlpError::ApiError {
            status: 404,
            message: "not found".to_string(),
        };
        assert_eq!(format!("{}", err), "API error (status 404): not found");
    }

    #[test]
    fn test_display_connection_failed() {
        let err = OtlpError::ConnectionFailed("timeout".to_string());
        assert_eq!(format!("{}", err), "connection failed: timeout");
    }

    #[test]
    fn test_display_authentication_failed() {
        let err = OtlpError::AuthenticationFailed("bad token".to_string());
        assert_eq!(format!("{}", err), "authentication failed: bad token");
    }

    #[test]
    fn test_display_invalid_query() {
        let err = OtlpError::InvalidQuery("missing time range".to_string());
        assert_eq!(format!("{}", err), "invalid query: missing time range");
    }

    #[test]
    fn test_display_backend() {
        let err = OtlpError::Backend("internal failure".to_string());
        assert_eq!(format!("{}", err), "backend error: internal failure");
    }

    #[test]
    fn test_from_serde_json_error() {
        let serde_err = serde_json::from_str::<String>("not json").unwrap_err();
        let err: OtlpError = serde_err.into();
        assert!(matches!(err, OtlpError::Deserialization(_)));
        let display = format!("{}", err);
        assert!(display.starts_with("deserialization error:"));
    }

    #[test]
    fn test_error_trait_source() {
        let serde_err = serde_json::from_str::<String>("not json").unwrap_err();
        let err: OtlpError = serde_err.into();
        assert!(std::error::Error::source(&err).is_some());

        let err = OtlpError::Backend("test".to_string());
        assert!(std::error::Error::source(&err).is_none());
    }
}
