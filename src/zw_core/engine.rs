//! 零宽字符分析与解码引擎

use std::collections::{BTreeMap, HashMap};

use super::chars::{all_zero_width_chars, is_unicode_tag, is_zero_width, UNICODE_TAGS_START};

// ============================================================
// 分析
// ============================================================

/// 文本分析结果
#[derive(Debug, Clone)]
pub struct Analysis {
    pub total_chars: usize,
    pub visible_chars: usize,
    pub zero_width_count: usize,
    pub unique_zw_chars: usize,
    /// codepoint -> count
    pub distribution: BTreeMap<u32, usize>,
    pub has_unicode_tags: bool,
}

/// 分析文本中的零宽字符分布
pub fn analyze(text: &str) -> Analysis {
    let mut distribution: BTreeMap<u32, usize> = BTreeMap::new();
    let mut visible = 0usize;
    let mut has_tags = false;

    for ch in text.chars() {
        if is_zero_width(ch) {
            *distribution.entry(ch as u32).or_insert(0) += 1;
            if is_unicode_tag(ch) {
                has_tags = true;
            }
        } else if !ch.is_control() {
            visible += 1;
        }
    }

    let zw_count: usize = distribution.values().sum();
    Analysis {
        total_chars: text.chars().count(),
        visible_chars: visible,
        zero_width_count: zw_count,
        unique_zw_chars: distribution.len(),
        distribution,
        has_unicode_tags: has_tags,
    }
}

/// 格式化分析报告
pub fn format_analysis(analysis: &Analysis) -> String {
    let mut out = String::new();
    out.push_str(&format!("总字符数: {}\n", analysis.total_chars));
    out.push_str(&format!("可见字符数: {}\n", analysis.visible_chars));
    out.push_str(&format!("零宽字符数: {}\n", analysis.zero_width_count));
    out.push_str(&format!("零宽字符种类: {}\n", analysis.unique_zw_chars));

    if analysis.zero_width_count == 0 {
        out.push_str("未检测到零宽字符!\n");
        return out;
    }

    let name_map: HashMap<u32, &str> = all_zero_width_chars()
        .iter()
        .map(|z| (z.codepoint, z.name))
        .collect();

    out.push_str("\n字符分布:\n");
    let mut sorted: Vec<_> = analysis.distribution.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (&cp, &count) in &sorted {
        let name = name_map
            .get(&cp)
            .copied()
            .unwrap_or("UNICODE TAG");
        out.push_str(&format!("  U+{:04X} {}: {} 次\n", cp, name, count));
    }
    out
}

// ============================================================
// 提取
// ============================================================

/// 提取所有零宽字符
pub fn extract_all(text: &str) -> Vec<char> {
    text.chars().filter(|&ch| is_zero_width(ch)).collect()
}

/// 提取零宽字符段（按可见字符分割）
pub fn extract_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if is_zero_width(ch) {
            current.push(ch);
        } else if !current.is_empty() {
            segments.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

// ============================================================
// 解码方案
// ============================================================

/// 单条解码结果
#[derive(Debug, Clone)]
pub struct DecodeResult {
    pub method: String,
    pub decoded: String,
    pub score: f64,
}

/// 判断解码结果是否可能有效
fn is_printable(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let printable = text.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t').count();
    let ratio = printable as f64 / text.chars().count() as f64;
    ratio > 0.5
}

/// 为解码结果打分
fn score(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }
    let len = text.chars().count();
    let mut s = 0.0;

    // 可打印比例
    let printable = text.chars().filter(|c| !c.is_control()).count();
    s += (printable as f64 / len as f64) * 30.0;

    // ASCII 字母数字比例
    let alnum = text.chars().filter(|c| c.is_ascii_alphanumeric()).count();
    s += (alnum as f64 / len as f64) * 30.0;

    // 长度奖励
    if len >= 3 {
        s += (len.min(20)) as f64;
    }

    // 空格
    if text.contains(' ') {
        s += 10.0;
    }

    // CTF flag 格式
    let flags = ["flag{", "ctf{", "FLAG{", "CTF{", "key{", "KEY{"];
    if flags.iter().any(|f| text.contains(f)) {
        s += 50.0;
    }

    // 连续不可打印惩罚
    let mut max_unp = 0usize;
    let mut cur_unp = 0usize;
    for c in text.chars() {
        if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
            cur_unp += 1;
            max_unp = max_unp.max(cur_unp);
        } else {
            cur_unp = 0;
        }
    }
    s -= (max_unp as f64) * 5.0;

    s.max(0.0)
}

// --- 方案1: Unicode Tags ---
pub fn decode_unicode_tags(text: &str) -> Option<DecodeResult> {
    let mut result = String::new();
    for ch in text.chars() {
        let cp = ch as u32;
        if cp >= UNICODE_TAGS_START && cp <= super::chars::UNICODE_TAGS_END {
            let ascii = cp - UNICODE_TAGS_START;
            if ascii > 0 && ascii < 128 {
                if let Some(c) = char::from_u32(ascii) {
                    result.push(c);
                }
            }
        }
    }
    if result.is_empty() || !is_printable(&result) {
        return None;
    }
    let s = score(&result);
    Some(DecodeResult {
        method: "Unicode Tags (U+E0000-U+E007F)".to_string(),
        decoded: result,
        score: s,
    })
}

// --- 方案2: Steganographr (neatnik.net) ---
pub fn decode_steganographr(text: &str) -> Option<DecodeResult> {
    const WJ: char = '\u{2060}';
    const ZWSP: char = '\u{200B}';
    const ZWNJ: char = '\u{200C}';

    let zw_only: String = text.chars().filter(|&c| c == WJ || c == ZWSP || c == ZWNJ).collect();
    if zw_only.is_empty() {
        return None;
    }

    let parts: Vec<&str> = zw_only.split(WJ).collect();
    let mut result = String::new();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        let binary: String = part
            .chars()
            .filter_map(|c| match c {
                ZWSP => Some('0'),
                ZWNJ => Some('1'),
                _ => None,
            })
            .collect();
        if !binary.is_empty() {
            if let Ok(value) = u32::from_str_radix(&binary, 2) {
                if value > 0 {
                    if let Some(c) = char::from_u32(value) {
                        result.push(c);
                    }
                }
            }
        }
    }

    if result.is_empty() || !is_printable(&result) {
        return None;
    }
    let s = score(&result);
    Some(DecodeResult {
        method: "Steganographr (WJ+ZWSP+ZWNJ)".to_string(),
        decoded: result,
        score: s,
    })
}

// --- 方案3: 直接二进制 ---
pub fn decode_direct_binary(
    zw_seq: &[char],
    zero_char: char,
    one_char: char,
    bits: usize,
) -> Option<DecodeResult> {
    let binary: String = zw_seq
        .iter()
        .filter_map(|&c| {
            if c == zero_char {
                Some('0')
            } else if c == one_char {
                Some('1')
            } else {
                None
            }
        })
        .collect();

    if binary.len() < bits {
        return None;
    }

    let mut result = String::new();
    for chunk in binary.as_bytes().chunks(bits) {
        if chunk.len() < bits {
            break;
        }
        let s = std::str::from_utf8(chunk).unwrap_or("");
        if let Ok(value) = u32::from_str_radix(s, 2) {
            if value > 0 && value < 128 {
                result.push(char::from_u32(value).unwrap_or('?'));
            }
        }
    }

    if result.is_empty() || !is_printable(&result) {
        return None;
    }

    let s = score(&result);
    let z_code = format!("U+{:04X}", zero_char as u32);
    let o_code = format!("U+{:04X}", one_char as u32);
    Some(DecodeResult {
        method: format!("二进制 ({}=0, {}=1, {}bit)", z_code, o_code, bits),
        decoded: result,
        score: s,
    })
}

// --- 方案4: N进制映射 (330k 风格) ---
pub fn decode_nary(zw_seq: &[char], charset: &[char]) -> Vec<DecodeResult> {
    let base = charset.len();
    if base < 2 {
        return vec![];
    }

    let char_to_digit: HashMap<char, usize> = charset.iter().enumerate().map(|(i, &c)| (c, i)).collect();
    let digits: Vec<usize> = zw_seq.iter().filter_map(|c| char_to_digit.get(c).copied()).collect();

    if digits.is_empty() {
        return vec![];
    }

    // 计算每个字符需要多少个零宽字符
    let chars_per_unicode = ((16.0f64) / (base as f64).log2()).ceil() as usize;

    let mut results = Vec::new();
    let try_sizes: Vec<usize> = {
        let mut s = vec![chars_per_unicode];
        if chars_per_unicode > 1 { s.push(chars_per_unicode - 1); }
        s.push(chars_per_unicode + 1);
        for extra in [4, 5, 6, 7, 8] {
            if !s.contains(&extra) { s.push(extra); }
        }
        s
    };

    for group_size in try_sizes {
        if group_size < 1 || group_size > 16 {
            continue;
        }
        let mut text = String::new();
        let mut ok = true;
        for chunk in digits.chunks(group_size) {
            if chunk.len() < group_size {
                break;
            }
            let mut value = 0u32;
            for &d in chunk {
                value = value.saturating_mul(base as u32).saturating_add(d as u32);
            }
            if value > 0 && value < 0x110000 {
                if let Some(c) = char::from_u32(value) {
                    text.push(c);
                } else {
                    ok = false;
                    break;
                }
            } else if value == 0 {
                continue;
            } else {
                ok = false;
                break;
            }
        }
        if ok && !text.is_empty() && is_printable(&text) {
            let s = score(&text);
            if s > 15.0 {
                let chars_desc: Vec<String> = charset.iter().map(|c| format!("U+{:04X}", *c as u32)).collect();
                results.push(DecodeResult {
                    method: format!("{}进制 ({}, 分组={})", base, chars_desc.join("+"), group_size),
                    decoded: text,
                    score: s,
                });
            }
        }
    }
    results
}

// --- 方案5: 分段二进制 ---
pub fn decode_segmented_binary(
    segments: &[String],
    zero_char: char,
    one_char: char,
    bits: usize,
) -> Option<DecodeResult> {
    let mut result = String::new();
    for seg in segments {
        let binary: String = seg
            .chars()
            .filter_map(|c| {
                if c == zero_char {
                    Some('0')
                } else if c == one_char {
                    Some('1')
                } else {
                    None
                }
            })
            .collect();
        if binary.len() != bits {
            return None;
        }
        if let Ok(value) = u32::from_str_radix(&binary, 2) {
            if value > 0 && value < 128 {
                result.push(char::from_u32(value).unwrap_or('?'));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    if result.is_empty() || !is_printable(&result) {
        return None;
    }

    let s = score(&result);
    let z_code = format!("U+{:04X}", zero_char as u32);
    let o_code = format!("U+{:04X}", one_char as u32);
    Some(DecodeResult {
        method: format!("分段二进制 ({}=0, {}=1, {}bit/段)", z_code, o_code, bits),
        decoded: result,
        score: s,
    })
}

// ============================================================
// 编码
// ============================================================

/// 二进制编码
pub fn encode_binary(message: &str, zero_char: char, one_char: char, bits: usize) -> String {
    let mut result = String::new();
    for ch in message.chars() {
        let val = ch as u32;
        for i in (0..bits).rev() {
            if (val >> i) & 1 == 1 {
                result.push(one_char);
            } else {
                result.push(zero_char);
            }
        }
    }
    result
}

/// Steganographr 编码
pub fn encode_steganographr(message: &str, cover: &str) -> String {
    const WJ: char = '\u{2060}';
    const ZWSP: char = '\u{200B}';
    const ZWNJ: char = '\u{200C}';

    let mut encoded = String::new();
    for ch in message.chars() {
        let val = ch as u32;
        for i in (0..8).rev() {
            if (val >> i) & 1 == 1 {
                encoded.push(ZWNJ);
            } else {
                encoded.push(ZWSP);
            }
        }
        encoded.push(WJ);
    }

    if cover.len() > 1 {
        let mid = cover.chars().count() / 2;
        let prefix: String = cover.chars().take(mid).collect();
        let suffix: String = cover.chars().skip(mid).collect();
        format!("{}{}{}", prefix, encoded, suffix)
    } else {
        encoded
    }
}

/// Unicode Tags 编码
pub fn encode_tags(message: &str, cover: &str) -> String {
    let mut encoded = String::new();
    for ch in message.chars() {
        let cp = ch as u32;
        if cp < 128 {
            if let Some(c) = char::from_u32(UNICODE_TAGS_START + cp) {
                encoded.push(c);
            }
        }
    }
    if cover.len() > 1 {
        let mid = cover.chars().count() / 2;
        let prefix: String = cover.chars().take(mid).collect();
        let suffix: String = cover.chars().skip(mid).collect();
        format!("{}{}{}", prefix, encoded, suffix)
    } else {
        encoded
    }
}

/// 330k 方案编码
pub fn encode_330k(message: &str, cover: &str, charset: &[char]) -> String {
    let base = charset.len() as u32;
    let chars_per_unicode = ((16.0f64) / (base as f64).log2()).ceil() as usize;

    let mut encoded = String::new();
    for ch in message.chars() {
        let mut val = ch as u32;
        let mut digits = Vec::new();
        for _ in 0..chars_per_unicode {
            digits.push((val % base) as usize);
            val /= base;
        }
        digits.reverse();
        for d in digits {
            encoded.push(charset[d]);
        }
    }

    if cover.len() > 1 {
        let mid = cover.chars().count() / 2;
        let prefix: String = cover.chars().take(mid).collect();
        let suffix: String = cover.chars().skip(mid).collect();
        format!("{}{}{}", prefix, encoded, suffix)
    } else {
        encoded
    }
}

// ============================================================
// 自动解码引擎
// ============================================================

/// 预设编码方案
pub struct Preset {
    pub name: &'static str,
    pub chars: Vec<char>,
    pub description: &'static str,
}

pub fn encoding_presets() -> Vec<(&'static str, Preset)> {
    vec![
        ("330k_default", Preset {
            name: "330k Unicode Steganography (默认4字符)",
            chars: vec!['\u{200C}', '\u{200D}', '\u{202C}', '\u{FEFF}'],
            description: "330k.github.io 默认方案: 4字符=2bit编码",
        }),
        ("steganographr", Preset {
            name: "Steganographr (neatnik.net)",
            chars: vec!['\u{2060}', '\u{200B}', '\u{200C}'],
            description: "WJ=分隔符, ZWSP=0, ZWNJ=1",
        }),
        ("stegcloak", Preset {
            name: "StegCloak",
            chars: vec!['\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}'],
            description: "StegCloak 字符集",
        }),
        ("zwsp_binary", Preset {
            name: "ZWSP Binary (基础二进制)",
            chars: vec!['\u{200B}', '\u{200C}'],
            description: "ZWSP=0, ZWNJ=1",
        }),
        ("common_3char", Preset {
            name: "常见三字符方案",
            chars: vec!['\u{200B}', '\u{200C}', '\u{200D}'],
            description: "ZWSP/ZWNJ/ZWJ 三字符方案",
        }),
        ("irongeek_zw", Preset {
            name: "Irongeek Zero-Width",
            chars: vec!['\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}'],
            description: "Irongeek 推荐的最兼容零宽字符组合",
        }),
    ]
}

/// 自动尝试所有方案解码
pub fn auto_decode(text: &str) -> Vec<DecodeResult> {
    let analysis = analyze(text);
    if analysis.zero_width_count == 0 {
        return vec![];
    }

    let mut results = Vec::new();
    let zw_all = extract_all(text);
    let segments = extract_segments(text);

    // 按出现次数排序的唯一零宽字符
    let mut freq: Vec<(u32, usize)> = analysis.distribution.iter().map(|(&k, &v)| (k, v)).collect();
    freq.sort_by(|a, b| b.1.cmp(&a.1));
    let top_chars: Vec<char> = freq.iter().filter_map(|(cp, _)| char::from_u32(*cp)).collect();

    // 方案1: Unicode Tags
    if let Some(r) = decode_unicode_tags(text) {
        results.push(r);
    }

    // 方案2: Steganographr
    if let Some(r) = decode_steganographr(text) {
        results.push(r);
    }

    // 方案3: 预设 N进制
    for (_, preset) in encoding_presets() {
        let preset_in_text: Vec<char> = preset.chars.iter().copied()
            .filter(|c| analysis.distribution.contains_key(&(*c as u32)))
            .collect();
        if preset_in_text.len() >= 2 {
            let mut nary = decode_nary(&zw_all, &preset.chars);
            results.append(&mut nary);
        }
    }

    // 方案4: 暴力二进制
    if top_chars.len() >= 2 {
        let limit = top_chars.len().min(6);
        for i in 0..limit {
            for j in 0..limit {
                if i == j { continue; }
                for bits in [8, 7] {
                    if let Some(r) = decode_direct_binary(&zw_all, top_chars[i], top_chars[j], bits) {
                        if r.score > 15.0 {
                            results.push(r);
                        }
                    }
                }
            }
        }
    }

    // 方案5: N进制 (使用实际出现的字符)
    if top_chars.len() >= 3 {
        for n in 3..top_chars.len().min(9) {
            let charset: Vec<char> = top_chars[..n].to_vec();
            let mut nary = decode_nary(&zw_all, &charset);
            results.append(&mut nary);
        }
    }

    // 方案6: 分段二进制
    if !segments.is_empty() && top_chars.len() >= 2 {
        let limit = top_chars.len().min(4);
        for i in 0..limit {
            for j in 0..limit {
                if i == j { continue; }
                for bits in [8, 7] {
                    if let Some(r) = decode_segmented_binary(&segments, top_chars[i], top_chars[j], bits) {
                        if r.score > 15.0 {
                            results.push(r);
                        }
                    }
                }
            }
        }
    }

    // 去重并排序
    let mut seen = std::collections::HashSet::new();
    results.retain(|r| seen.insert(r.decoded.clone()));
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// 导出原始零宽字符序列
pub fn dump_raw(text: &str) -> String {
    let name_map: HashMap<u32, &str> = all_zero_width_chars()
        .iter()
        .map(|z| (z.codepoint, z.name))
        .collect();

    let mut out = String::from("原始零宽字符序列:\n");
    for (i, ch) in text.chars().enumerate() {
        let cp = ch as u32;
        if let Some(name) = name_map.get(&cp) {
            out.push_str(&format!("[{:4}] U+{:04X} {}\n", i, cp, name));
        } else if is_unicode_tag(ch) {
            let ascii = cp - UNICODE_TAGS_START;
            let display = if (32..127).contains(&ascii) {
                char::from_u32(ascii).unwrap_or('?')
            } else {
                '?'
            };
            out.push_str(&format!("[{:4}] U+{:05X} UNICODE TAG (ASCII {} = '{}')\n", i, cp, ascii, display));
        }
    }
    out
}
