//! 零宽字符隐写术 MCP Server
//!
//! 一个用于 CTF 解题的 MCP (Model Context Protocol) 服务器，
//! 将零宽字符隐写术的分析、编码、解码功能暴露为工具，
//! 供大模型 (Claude, GPT, etc.) 通过 MCP 协议调用。
//!
//! ## 提供的工具
//!
//! - `zw_analyze`      - 分析文本中的零宽字符分布
//! - `zw_decode`       - 自动解码零宽字符隐写信息（支持多种方案）
//! - `zw_encode`       - 将消息编码为零宽字符隐写文本
//! - `zw_dump_raw`     - 导出原始零宽字符序列（调试用）
//! - `zw_list_chars`   - 列出所有已知零宽/不可见字符
//! - `zw_list_presets` - 列出所有编码预设方案
//!
//! ## 运行
//!
//! ```bash
//! # 直接启动 MCP Server（stdio 模式）
//! zw-mcp-server
//! ```

mod mcp;
mod zw_core;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // 日志输出到 stderr，避免干扰 stdio MCP 通信
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();

    if let Err(e) = mcp::server::run().await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
