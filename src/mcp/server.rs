//! MCP Server - stdio 传输层实现

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::protocol::*;
use super::tools;

/// 运行 MCP Server（stdio 模式）
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    tracing::info!("MCP Server started (stdio mode)");

    while let Some(line) = lines.next_line().await? {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        tracing::debug!("Received: {}", &line);

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::error(
                    None,
                    -32700,
                    format!("Parse error: {}", e),
                );
                let msg = serde_json::to_string(&resp)?;
                stdout.write_all(msg.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
                continue;
            }
        };

        let response = handle_request(&request);

        if let Some(resp) = response {
            let msg = serde_json::to_string(&resp)?;
            tracing::debug!("Sending: {}", &msg);
            stdout.write_all(msg.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
    }

    tracing::info!("MCP Server shutting down");
    Ok(())
}

/// 处理单个 JSON-RPC 请求
fn handle_request(req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    match req.method.as_str() {
        // --- MCP 握手 ---
        "initialize" => {
            let result = InitializeResult {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ServerCapabilities {
                    tools: ToolsCapability { list_changed: false },
                },
                server_info: ServerInfo {
                    name: "zw-mcp-server".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            };
            Some(JsonRpcResponse::success(
                req.id.clone(),
                serde_json::to_value(result).unwrap(),
            ))
        }

        // 通知: 不需要响应
        "notifications/initialized" => None,

        // --- 工具列表 ---
        "tools/list" => {
            let result = ToolsListResult {
                tools: tools::all_tools(),
            };
            Some(JsonRpcResponse::success(
                req.id.clone(),
                serde_json::to_value(result).unwrap(),
            ))
        }

        // --- 工具调用 ---
        "tools/call" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            tracing::info!("Tool call: {} with args: {}", tool_name, &arguments);

            let result = tools::call_tool(tool_name, &arguments);
            Some(JsonRpcResponse::success(
                req.id.clone(),
                serde_json::to_value(result).unwrap(),
            ))
        }

        // --- Ping ---
        "ping" => Some(JsonRpcResponse::success(
            req.id.clone(),
            serde_json::json!({}),
        )),

        // --- 未知方法 ---
        _ => {
            tracing::warn!("Unknown method: {}", req.method);
            // 如果是通知（没有id），不要响应
            if req.id.is_none() {
                None
            } else {
                Some(JsonRpcResponse::error(
                    req.id.clone(),
                    -32601,
                    format!("Method not found: {}", req.method),
                ))
            }
        }
    }
}
