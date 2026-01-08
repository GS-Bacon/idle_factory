//! JSON-RPC 2.0 protocol definitions for Mod API
//!
//! This module defines the standard JSON-RPC 2.0 message types
//! used for communication between the game and external mods.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    /// Request ID (None for notifications)
    pub id: Option<u64>,
    /// Method name
    pub method: String,
    /// Parameters (default to null)
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request ID (matches the request)
    pub id: Option<u64>,
    /// Result (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Notification (request without id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Parameters
    pub params: serde_json::Value,
}

// Standard JSON-RPC 2.0 error codes
/// Parse error: Invalid JSON
pub const PARSE_ERROR: i32 = -32700;
/// Invalid Request: JSON is not a valid Request object
pub const INVALID_REQUEST: i32 = -32600;
/// Method not found
pub const METHOD_NOT_FOUND: i32 = -32601;
/// Invalid params
pub const INVALID_PARAMS: i32 = -32602;
/// Internal error
pub const INTERNAL_ERROR: i32 = -32603;

// Custom error codes (application-specific, -32000 to -32099)
/// Mod not found
pub const MOD_NOT_FOUND: i32 = -32000;
/// Permission denied
pub const PERMISSION_DENIED: i32 = -32001;
/// Rate limited
pub const RATE_LIMITED: i32 = -32002;

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(id: u64, method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: method.into(),
            params,
        }
    }

    /// Create a notification (request without id)
    pub fn notification(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.into(),
            params,
        }
    }

    /// Check if this is a valid JSON-RPC 2.0 request
    pub fn is_valid(&self) -> bool {
        self.jsonrpc == "2.0" && !self.method.is_empty()
    }

    /// Check if this is a notification (no id)
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

impl JsonRpcResponse {
    /// Create a success response
    pub fn success(id: Option<u64>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: Option<u64>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    /// Create an error response with additional data
    pub fn error_with_data(
        id: Option<u64>,
        code: i32,
        message: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: Some(data),
            }),
        }
    }

    /// Check if this response is successful
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Check if this response is an error
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

impl JsonRpcError {
    /// Create a new error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a parse error
    pub fn parse_error() -> Self {
        Self::new(PARSE_ERROR, "Parse error")
    }

    /// Create an invalid request error
    pub fn invalid_request() -> Self {
        Self::new(INVALID_REQUEST, "Invalid Request")
    }

    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::new(METHOD_NOT_FOUND, format!("Method not found: {}", method))
    }

    /// Create an invalid params error
    pub fn invalid_params(details: &str) -> Self {
        Self::new(INVALID_PARAMS, format!("Invalid params: {}", details))
    }

    /// Create an internal error
    pub fn internal_error(details: &str) -> Self {
        Self::new(INTERNAL_ERROR, format!("Internal error: {}", details))
    }
}

impl JsonRpcNotification {
    /// Create a new notification
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }

    /// Create a notification with no params
    pub fn empty(method: impl Into<String>) -> Self {
        Self::new(method, serde_json::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_new() {
        let req = JsonRpcRequest::new(1, "game.version", serde_json::json!({}));

        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.id, Some(1));
        assert_eq!(req.method, "game.version");
        assert!(req.is_valid());
        assert!(!req.is_notification());
    }

    #[test]
    fn test_request_notification() {
        let req = JsonRpcRequest::notification("event.tick", serde_json::json!({"tick": 100}));

        assert_eq!(req.jsonrpc, "2.0");
        assert!(req.id.is_none());
        assert!(req.is_notification());
    }

    #[test]
    fn test_response_success() {
        let resp = JsonRpcResponse::success(Some(1), serde_json::json!({"version": "0.3.0"}));

        assert!(resp.is_success());
        assert!(!resp.is_error());
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_response_error() {
        let resp = JsonRpcResponse::error(Some(1), METHOD_NOT_FOUND, "Method not found: foo");

        assert!(!resp.is_success());
        assert!(resp.is_error());
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());

        let error = resp.error.unwrap();
        assert_eq!(error.code, METHOD_NOT_FOUND);
    }

    #[test]
    fn test_error_factories() {
        assert_eq!(JsonRpcError::parse_error().code, PARSE_ERROR);
        assert_eq!(JsonRpcError::invalid_request().code, INVALID_REQUEST);
        assert_eq!(
            JsonRpcError::method_not_found("test").code,
            METHOD_NOT_FOUND
        );
        assert_eq!(JsonRpcError::invalid_params("test").code, INVALID_PARAMS);
        assert_eq!(JsonRpcError::internal_error("test").code, INTERNAL_ERROR);
    }

    #[test]
    fn test_notification_new() {
        let notif = JsonRpcNotification::new("event.update", serde_json::json!({"data": 123}));

        assert_eq!(notif.jsonrpc, "2.0");
        assert_eq!(notif.method, "event.update");
    }

    #[test]
    fn test_notification_empty() {
        let notif = JsonRpcNotification::empty("ping");

        assert_eq!(notif.params, serde_json::Value::Null);
    }

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest::new(1, "test", serde_json::json!({"foo": "bar"}));
        let json = serde_json::to_string(&req).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, Some(1));
        assert_eq!(parsed.method, "test");
    }

    #[test]
    fn test_response_serialization_success() {
        let resp = JsonRpcResponse::success(Some(1), serde_json::json!("ok"));
        let json = serde_json::to_string(&resp).unwrap();

        // error should not be present in JSON
        assert!(!json.contains("error"));
        assert!(json.contains("result"));
    }

    #[test]
    fn test_response_serialization_error() {
        let resp = JsonRpcResponse::error(Some(1), -32600, "Invalid");
        let json = serde_json::to_string(&resp).unwrap();

        // result should not be present in JSON
        assert!(!json.contains("result"));
        assert!(json.contains("error"));
    }

    #[test]
    fn test_request_with_null_params() {
        // Test default params behavior
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.params, serde_json::Value::Null);
    }
}
