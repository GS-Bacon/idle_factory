//! Server and client message types

use crate::modding::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// Message from WebSocket server to Bevy main thread
#[derive(Debug)]
pub enum ServerMessage {
    /// New connection established
    Connected { conn_id: u64 },
    /// Connection closed
    Disconnected { conn_id: u64 },
    /// JSON-RPC request received
    Request {
        conn_id: u64,
        request: JsonRpcRequest,
    },
}

/// Message from Bevy main thread to WebSocket server
#[derive(Debug)]
pub enum ClientMessage {
    /// Send response to a specific connection
    Response {
        conn_id: u64,
        response: JsonRpcResponse,
    },
    /// Send notification to a specific connection
    Notify {
        conn_id: u64,
        notification: JsonRpcNotification,
    },
    /// Broadcast notification to all connections
    Broadcast { notification: JsonRpcNotification },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_message_variants() {
        // Ensure all variants can be created
        let _connected = ServerMessage::Connected { conn_id: 1 };
        let _disconnected = ServerMessage::Disconnected { conn_id: 1 };
        let _request = ServerMessage::Request {
            conn_id: 1,
            request: JsonRpcRequest::new(1, "test", serde_json::Value::Null),
        };
    }

    #[test]
    fn test_client_message_variants() {
        // Ensure all variants can be created
        let _response = ClientMessage::Response {
            conn_id: 1,
            response: JsonRpcResponse::success(Some(1), serde_json::json!({})),
        };
        let _notify = ClientMessage::Notify {
            conn_id: 1,
            notification: JsonRpcNotification::empty("test"),
        };
        let _broadcast = ClientMessage::Broadcast {
            notification: JsonRpcNotification::empty("test"),
        };
    }
}
