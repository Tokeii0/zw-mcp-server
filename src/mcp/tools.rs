//! MCP 工具定义 - 将零宽字符功能暴露为 MCP tools

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use super::protocol::{Tool, ToolCallResult};
use crate::zw_core::{chars, engine};

/// 注册所有可用工具
pub fn all_tools() -> Vec<Tool> {
    vec![
        tool_analyze(),
        tool_decode(),
        tool_encode(),
        tool_dump_raw(),
        tool_list_chars(),
        tool_list_presets(),
    ]
}

// ============================================================
// 工具定义
// ============================================================

fn tool_analyze() -> Tool {
    Tool {
        name: "zw_analyze".to_string(),
        description: "分析文本中的零宽/不可见字符分布。输入可能包含隐写术隐藏信息的文本，返回零宽字符的种类、数量、分布等统计信息。支持直接传入文本或指定文件路径。".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "要分析的文本内容（可能包含不可见的零宽字符）。与 file_path 二选一"
                },
                "file_path": {
                    "type": "string",
                    "description": "要分析的文件路径（支持绝对路径和相对路径）。与 text 二选一"
                }
            }
        }),
    }
}

fn tool_decode() -> Tool {
    Tool {
        name: "zw_decode".to_string(),
        description: "自动解码文本中隐藏的零宽字符隐写信息。支持多种编码方案：二进制映射、N进制映射(330k)、Steganographr、Unicode Tags、莫尔斯码等。会自动尝试所有方案并按置信度排序返回结果。适用于CTF解题。支持直接传入文本或指定文件路径。".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "包含零宽字符隐写信息的文本。与 file_path 二选一"
                },
                "file_path": {
                    "type": "string",
                    "description": "包含零宽字符隐写信息的文件路径。与 text 二选一"
                },
                "method": {
                    "type": "string",
                    "description": "可选：指定解码方案。留空则自动尝试所有方案。可选值: auto, unicode_tags, steganographr, binary, 330k",
                    "enum": ["auto", "unicode_tags", "steganographr", "binary", "330k"]
                }
            }
        }),
    }
}

fn tool_encode() -> Tool {
    Tool {
        name: "zw_encode".to_string(),
        description: "将消息编码为零宽字符隐写文本。可选择不同编码方案，可指定载体文本将隐写信息嵌入其中。支持从文件读取载体文本，支持将编码结果写入文件。".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "要隐藏的秘密消息"
                },
                "method": {
                    "type": "string",
                    "description": "编码方法: binary, steganographr, tags, 330k",
                    "enum": ["binary", "steganographr", "tags", "330k"],
                    "default": "binary"
                },
                "cover_text": {
                    "type": "string",
                    "description": "可选：载体文本，隐写信息会嵌入其中。与 cover_file 二选一",
                    "default": ""
                },
                "cover_file": {
                    "type": "string",
                    "description": "可选：载体文本的文件路径。与 cover_text 二选一"
                },
                "output_path": {
                    "type": "string",
                    "description": "可选：将编码结果写入指定文件路径"
                }
            },
            "required": ["message"]
        }),
    }
}

fn tool_dump_raw() -> Tool {
    Tool {
        name: "zw_dump_raw".to_string(),
        description: "导出文本中所有零宽字符的原始序列，显示每个字符的位置、Unicode码点和名称。用于调试和手动分析。支持直接传入文本或指定文件路径。".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "要分析的文本。与 file_path 二选一"
                },
                "file_path": {
                    "type": "string",
                    "description": "要分析的文件路径。与 text 二选一"
                }
            }
        }),
    }
}

fn tool_list_chars() -> Tool {
    Tool {
        name: "zw_list_chars".to_string(),
        description: "列出所有已知的零宽/不可见Unicode字符（共182个），包括码点、名称、分类。".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

fn tool_list_presets() -> Tool {
    Tool {
        name: "zw_list_presets".to_string(),
        description: "列出所有支持的编码预设方案，包括330k、Steganographr、StegCloak、Binary等。".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

// ============================================================
// 工具执行
// ============================================================

/// 根据工具名称和参数执行工具
pub fn call_tool(name: &str, args: &Value) -> ToolCallResult {
    match name {
        "zw_analyze" => exec_analyze(args),
        "zw_decode" => exec_decode(args),
        "zw_encode" => exec_encode(args),
        "zw_dump_raw" => exec_dump_raw(args),
        "zw_list_chars" => exec_list_chars(),
        "zw_list_presets" => exec_list_presets(),
        _ => ToolCallResult::error(format!("未知工具: {}", name)),
    }
}

fn get_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str())
}

/// 从参数中获取文本，支持 text 直传 或 file_path 文件导入
/// 自动尝试多种编码: UTF-8, UTF-8 BOM, UTF-16 LE/BE, GBK, Latin-1
fn resolve_text(args: &Value) -> Result<String, ToolCallResult> {
    // 优先使用 file_path
    if let Some(path_str) = get_str(args, "file_path") {
        return read_file_auto(path_str);
    }
    // 其次使用 text
    if let Some(t) = get_str(args, "text") {
        return Ok(t.to_string());
    }
    Err(ToolCallResult::error("缺少参数: 请提供 text 或 file_path"))
}

/// 自动检测编码读取文件
fn read_file_auto(path_str: &str) -> Result<String, ToolCallResult> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(ToolCallResult::error(format!("文件不存在: {}", path_str)));
    }

    // 先读取原始字节
    let raw = match fs::read(path) {
        Ok(b) => b,
        Err(e) => return Err(ToolCallResult::error(format!("读取文件失败: {}", e))),
    };

    // 检测 BOM 并尝试对应编码
    if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // UTF-8 BOM
        if let Ok(s) = String::from_utf8(raw[3..].to_vec()) {
            return Ok(s);
        }
    }
    if raw.starts_with(&[0xFF, 0xFE]) {
        // UTF-16 LE BOM
        let iter = raw[2..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]]));
        let text: String = char::decode_utf16(iter)
            .map(|r| r.unwrap_or('\u{FFFD}'))
            .collect();
        return Ok(text);
    }
    if raw.starts_with(&[0xFE, 0xFF]) {
        // UTF-16 BE BOM
        let iter = raw[2..].chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]]));
        let text: String = char::decode_utf16(iter)
            .map(|r| r.unwrap_or('\u{FFFD}'))
            .collect();
        return Ok(text);
    }

    // 尝试 UTF-8
    if let Ok(s) = String::from_utf8(raw.clone()) {
        return Ok(s);
    }

    // 尝试 UTF-16 LE (无BOM)
    if raw.len() % 2 == 0 {
        let iter = raw.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]]));
        let text: String = char::decode_utf16(iter)
            .map(|r| r.unwrap_or('\u{FFFD}'))
            .collect();
        // 如果解码后大部分是可打印字符，认为成功
        let printable = text.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t').count();
        if text.chars().count() > 0 && printable as f64 / text.chars().count() as f64 > 0.7 {
            return Ok(text);
        }
    }

    // 最后降级: 使用 Latin-1 (ISO-8859-1, 不会失败)
    let text: String = raw.iter().map(|&b| b as char).collect();
    Ok(text)
}

/// 将内容写入文件
fn write_file(path_str: &str, content: &str) -> Result<(), ToolCallResult> {
    let path = Path::new(path_str);
    // 自动创建父目录
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(ToolCallResult::error(format!("创建目录失败: {}", e)));
            }
        }
    }
    match fs::write(path, content.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(ToolCallResult::error(format!("写入文件失败: {}", e))),
    }
}

fn exec_analyze(args: &Value) -> ToolCallResult {
    let text = match resolve_text(args) {
        Ok(t) => t,
        Err(e) => return e,
    };

    let mut report = String::new();
    if let Some(fp) = get_str(args, "file_path") {
        report.push_str(&format!("文件: {}\n", fp));
    }
    let analysis = engine::analyze(&text);
    report.push_str(&engine::format_analysis(&analysis));
    ToolCallResult::success(report)
}

fn exec_decode(args: &Value) -> ToolCallResult {
    let text = match resolve_text(args) {
        Ok(t) => t,
        Err(e) => return e,
    };

    let method = get_str(args, "method").unwrap_or("auto");

    let results = match method {
        "unicode_tags" => {
            engine::decode_unicode_tags(&text).into_iter().collect::<Vec<_>>()
        }
        "steganographr" => {
            engine::decode_steganographr(&text).into_iter().collect::<Vec<_>>()
        }
        "binary" => {
            // 暴力尝试二进制
            let analysis = engine::analyze(&text);
            let zw_all = engine::extract_all(&text);
            let mut freq: Vec<(u32, usize)> = analysis.distribution.iter().map(|(&k, &v)| (k, v)).collect();
            freq.sort_by(|a, b| b.1.cmp(&a.1));
            let top: Vec<char> = freq.iter().filter_map(|(cp, _)| char::from_u32(*cp)).collect();
            let limit = top.len().min(6);
            let mut results = Vec::new();
            for i in 0..limit {
                for j in 0..limit {
                    if i == j { continue; }
                    for bits in [8, 7] {
                        if let Some(r) = engine::decode_direct_binary(&zw_all, top[i], top[j], bits) {
                            if r.score > 15.0 {
                                results.push(r);
                            }
                        }
                    }
                }
            }
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            results
        }
        "330k" => {
            let zw_all = engine::extract_all(&text);
            let charset = vec!['\u{200C}', '\u{200D}', '\u{202C}', '\u{FEFF}'];
            engine::decode_nary(&zw_all, &charset)
        }
        _ => engine::auto_decode(&text),
    };

    if results.is_empty() {
        return ToolCallResult::success("未找到有效解码结果。请确认文本中包含零宽字符隐写信息。");
    }

    let mut output = String::new();
    if let Some(fp) = get_str(args, "file_path") {
        output.push_str(&format!("文件: {}\n", fp));
    }
    output.push_str(&format!("找到 {} 个可能的解码结果（按置信度排序）:\n\n", results.len()));
    for (i, r) in results.iter().enumerate().take(10) {
        output.push_str(&format!(
            "[{}] 方案: {}\n    得分: {:.1}\n    结果: {}\n\n",
            i + 1, r.method, r.score, r.decoded
        ));
    }
    if let Some(best) = results.first() {
        output.push_str(&format!("★ 最佳结果: {}\n", best.decoded));
    }
    ToolCallResult::success(output)
}

fn exec_encode(args: &Value) -> ToolCallResult {
    let message = match get_str(args, "message") {
        Some(m) => m,
        None => return ToolCallResult::error("缺少参数: message"),
    };
    let method = get_str(args, "method").unwrap_or("binary");

    // 载体文本: cover_file 优先于 cover_text
    let cover: String = if let Some(cover_path) = get_str(args, "cover_file") {
        match read_file_auto(cover_path) {
            Ok(t) => t,
            Err(e) => return e,
        }
    } else {
        get_str(args, "cover_text").unwrap_or("").to_string()
    };
    let cover = cover.as_str();

    let encoded = match method {
        "binary" => {
            let zw = engine::encode_binary(message, '\u{200B}', '\u{200C}', 8);
            if !cover.is_empty() {
                let mid = cover.chars().count() / 2;
                let prefix: String = cover.chars().take(mid).collect();
                let suffix: String = cover.chars().skip(mid).collect();
                format!("{}{}{}", prefix, zw, suffix)
            } else {
                zw
            }
        }
        "steganographr" => engine::encode_steganographr(message, cover),
        "tags" => engine::encode_tags(message, cover),
        "330k" => {
            let charset = vec!['\u{200C}', '\u{200D}', '\u{202C}', '\u{FEFF}'];
            engine::encode_330k(message, cover, &charset)
        }
        _ => return ToolCallResult::error(format!("未知编码方法: {}", method)),
    };

    let mut output = String::new();
    output.push_str(&format!("编码方法: {}\n", method));
    output.push_str(&format!("消息: {}\n", message));
    output.push_str(&format!("编码后长度: {} 字符\n", encoded.chars().count()));
    output.push_str(&format!("编码结果（repr）: {:?}\n", encoded));
    output.push_str(&format!("\n编码文本:\n{}\n", encoded));

    // 如果指定了输出文件，写入
    if let Some(out_path) = get_str(args, "output_path") {
        match write_file(out_path, &encoded) {
            Ok(_) => output.push_str(&format!("\n✓ 已写入文件: {}\n", out_path)),
            Err(e) => return e,
        }
    }

    ToolCallResult::success(output)
}

fn exec_dump_raw(args: &Value) -> ToolCallResult {
    let text = match resolve_text(args) {
        Ok(t) => t,
        Err(e) => return e,
    };
    let mut prefix = String::new();
    if let Some(fp) = get_str(args, "file_path") {
        prefix.push_str(&format!("文件: {}\n", fp));
    }
    let raw = engine::dump_raw(&text);
    if raw.lines().count() <= 1 {
        return ToolCallResult::success(format!("{}文本中未发现零宽字符。", prefix));
    }
    ToolCallResult::success(format!("{}{}", prefix, raw))
}

fn exec_list_chars() -> ToolCallResult {
    let all = chars::all_zero_width_chars();
    let mut output = String::from("零宽/不可见 Unicode 字符大全:\n\n");
    let mut current_cat = "";
    for zw in &all {
        if zw.category != current_cat {
            output.push_str(&format!("\n[{}]\n", zw.category));
            current_cat = zw.category;
        }
        output.push_str(&format!("  U+{:04X}  {}\n", zw.codepoint, zw.name));
    }
    output.push_str(&format!(
        "\nUnicode Tags: U+E0001 - U+E007F (128个，映射到 ASCII)\n\n共计 {} + 128 = {} 个字符\n",
        all.len(),
        all.len() + 128
    ));
    ToolCallResult::success(output)
}

fn exec_list_presets() -> ToolCallResult {
    let presets = engine::encoding_presets();
    let mut output = String::from("编码预设方案:\n\n");
    for (key, preset) in &presets {
        output.push_str(&format!("[{}]\n", key));
        output.push_str(&format!("  名称: {}\n", preset.name));
        output.push_str(&format!("  说明: {}\n", preset.description));
        let chars_str: Vec<String> = preset.chars.iter().map(|c| format!("U+{:04X}", *c as u32)).collect();
        output.push_str(&format!("  字符: {}\n\n", chars_str.join(" ")));
    }
    ToolCallResult::success(output)
}
