# zw-mcp-server

零宽字符隐写术 MCP Server — 专为 CTF 解题设计，供大模型通过 MCP 协议调用。

## 功能

| 工具 | 说明 |
|------|------|
| `zw_analyze` | 分析文本中的零宽/不可见字符分布 |
| `zw_decode` | 自动解码零宽字符隐写信息（支持8种方案，自动暴力尝试） |
| `zw_encode` | 将消息编码为零宽字符隐写文本 |
| `zw_dump_raw` | 导出原始零宽字符序列（调试用） |
| `zw_list_chars` | 列出全部 182 个已知零宽/不可见字符 |
| `zw_list_presets` | 列出所有编码预设方案 |

## 支持的编码方案

- **二进制映射** — 2字符→0/1 (7/8bit)，暴力尝试所有组合
- **N进制映射** — 330k.github.io 方案，支持2~8进制
- **Steganographr** — neatnik.net 方案 (WJ分隔+ZWSP/ZWNJ)
- **Unicode Tags** — U+E0000 偏移映射到 ASCII
- **StegCloak** — 4字符集方案
- **分段编码** — 按可见字符分割的段内二进制
- 自动暴力遍历所有字符排列组合

## 编译

```bash
cd zw-mcp-server
cargo build --release
```

编译后的可执行文件在 `target/release/zw-mcp-server.exe`

## 配置 MCP

### VS Code (GitHub Copilot)

在 VS Code 的 `.vscode/mcp.json` 或用户设置中添加：

```json
{
  "mcpServers": {
    "zw-steg": {
      "command": "d:\\AI\\CTFTOOLS\\zw-mcp-server\\target\\release\\zw-mcp-server.exe",
      "args": []
    }
  }
}
```

### Claude Desktop

在 `claude_desktop_config.json` 中添加：

```json
{
  "mcpServers": {
    "zw-steg": {
      "command": "d:\\AI\\CTFTOOLS\\zw-mcp-server\\target\\release\\zw-mcp-server.exe"
    }
  }
}
```

### Cursor / Continue / 其他 MCP 客户端

同理配置 stdio 模式即可。

## 使用示例

大模型可以直接调用：

> "请分析这段文本中是否包含零宽字符隐写信息并解码"

MCP 会自动路由到 `zw_analyze` + `zw_decode` 工具。

## 协议

- 传输: stdio (stdin/stdout)
- 协议: JSON-RPC 2.0
- MCP 版本: 2024-11-05
