//! MCP (Model Context Protocol) JSON-RPC 2.0 协议类型定义

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================
// JSON-RPC 2.0 核心类型
// ============================================================

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

// ============================================================
// MCP 协议类型
// ============================================================

#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    pub tools: ToolsCapability,
}

#[derive(Debug, Serialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

#[derive(Debug, Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Serialize)]
pub struct ToolsListResult {
    pub tools: Vec<Tool>,
}

#[derive(Debug, Serialize)]
pub struct ToolCallResult {
    pub content: Vec<Content>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl Content {
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            content_type: "text".to_string(),
            text: s.into(),
        }
    }
}

impl ToolCallResult {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(text)],
            is_error: None,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(text)],
            is_error: Some(true),
        }
    }
}
