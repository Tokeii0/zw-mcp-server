//! 全网收集的零宽/不可见 Unicode 字符大全

/// 零宽字符信息
#[derive(Debug, Clone)]
pub struct ZeroWidthChar {
    pub ch: char,
    pub codepoint: u32,
    pub name: &'static str,
    pub category: &'static str,
}

/// Unicode Tags 范围
pub const UNICODE_TAGS_START: u32 = 0xE0000;
pub const UNICODE_TAGS_END: u32 = 0xE007F;

/// 所有已知的零宽/不可见字符
pub fn all_zero_width_chars() -> Vec<ZeroWidthChar> {
    vec![
        // --- 核心零宽字符 ---
        ZeroWidthChar { ch: '\u{200B}', codepoint: 0x200B, name: "ZERO WIDTH SPACE (ZWSP)", category: "核心零宽" },
        ZeroWidthChar { ch: '\u{200C}', codepoint: 0x200C, name: "ZERO WIDTH NON-JOINER (ZWNJ)", category: "核心零宽" },
        ZeroWidthChar { ch: '\u{200D}', codepoint: 0x200D, name: "ZERO WIDTH JOINER (ZWJ)", category: "核心零宽" },
        ZeroWidthChar { ch: '\u{FEFF}', codepoint: 0xFEFF, name: "ZERO WIDTH NO-BREAK SPACE / BOM", category: "核心零宽" },
        ZeroWidthChar { ch: '\u{2060}', codepoint: 0x2060, name: "WORD JOINER (WJ)", category: "核心零宽" },

        // --- 方向控制标记 ---
        ZeroWidthChar { ch: '\u{200E}', codepoint: 0x200E, name: "LEFT-TO-RIGHT MARK (LRM)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{200F}', codepoint: 0x200F, name: "RIGHT-TO-LEFT MARK (RLM)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{202A}', codepoint: 0x202A, name: "LEFT-TO-RIGHT EMBEDDING (LRE)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{202B}', codepoint: 0x202B, name: "RIGHT-TO-LEFT EMBEDDING (RLE)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{202C}', codepoint: 0x202C, name: "POP DIRECTIONAL FORMATTING (PDF)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{202D}', codepoint: 0x202D, name: "LEFT-TO-RIGHT OVERRIDE (LRO)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{202E}', codepoint: 0x202E, name: "RIGHT-TO-LEFT OVERRIDE (RLO)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{2066}', codepoint: 0x2066, name: "LEFT-TO-RIGHT ISOLATE (LRI)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{2067}', codepoint: 0x2067, name: "RIGHT-TO-LEFT ISOLATE (RLI)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{2068}', codepoint: 0x2068, name: "FIRST STRONG ISOLATE (FSI)", category: "方向控制" },
        ZeroWidthChar { ch: '\u{2069}', codepoint: 0x2069, name: "POP DIRECTIONAL ISOLATE (PDI)", category: "方向控制" },

        // --- 不可见数学运算符 ---
        ZeroWidthChar { ch: '\u{2061}', codepoint: 0x2061, name: "FUNCTION APPLICATION", category: "不可见数学" },
        ZeroWidthChar { ch: '\u{2062}', codepoint: 0x2062, name: "INVISIBLE TIMES", category: "不可见数学" },
        ZeroWidthChar { ch: '\u{2063}', codepoint: 0x2063, name: "INVISIBLE SEPARATOR", category: "不可见数学" },
        ZeroWidthChar { ch: '\u{2064}', codepoint: 0x2064, name: "INVISIBLE PLUS", category: "不可见数学" },

        // --- 蒙古文元音分隔符 ---
        ZeroWidthChar { ch: '\u{180E}', codepoint: 0x180E, name: "MONGOLIAN VOWEL SEPARATOR", category: "蒙古文" },

        // --- 其他不可见/格式字符 ---
        ZeroWidthChar { ch: '\u{00AD}', codepoint: 0x00AD, name: "SOFT HYPHEN (SHY)", category: "格式字符" },
        ZeroWidthChar { ch: '\u{034F}', codepoint: 0x034F, name: "COMBINING GRAPHEME JOINER (CGJ)", category: "格式字符" },
        ZeroWidthChar { ch: '\u{061C}', codepoint: 0x061C, name: "ARABIC LETTER MARK (ALM)", category: "格式字符" },
        ZeroWidthChar { ch: '\u{115F}', codepoint: 0x115F, name: "HANGUL CHOSEONG FILLER", category: "格式字符" },
        ZeroWidthChar { ch: '\u{1160}', codepoint: 0x1160, name: "HANGUL JUNGSEONG FILLER", category: "格式字符" },
        ZeroWidthChar { ch: '\u{17B4}', codepoint: 0x17B4, name: "KHMER VOWEL INHERENT AQ", category: "格式字符" },
        ZeroWidthChar { ch: '\u{17B5}', codepoint: 0x17B5, name: "KHMER VOWEL INHERENT AA", category: "格式字符" },
        ZeroWidthChar { ch: '\u{3164}', codepoint: 0x3164, name: "HANGUL FILLER", category: "格式字符" },
        ZeroWidthChar { ch: '\u{FFA0}', codepoint: 0xFFA0, name: "HALFWIDTH HANGUL FILLER", category: "格式字符" },

        // --- 变体选择器 ---
        ZeroWidthChar { ch: '\u{FE00}', codepoint: 0xFE00, name: "VARIATION SELECTOR-1", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE01}', codepoint: 0xFE01, name: "VARIATION SELECTOR-2", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE02}', codepoint: 0xFE02, name: "VARIATION SELECTOR-3", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE03}', codepoint: 0xFE03, name: "VARIATION SELECTOR-4", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE04}', codepoint: 0xFE04, name: "VARIATION SELECTOR-5", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE05}', codepoint: 0xFE05, name: "VARIATION SELECTOR-6", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE06}', codepoint: 0xFE06, name: "VARIATION SELECTOR-7", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE07}', codepoint: 0xFE07, name: "VARIATION SELECTOR-8", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE08}', codepoint: 0xFE08, name: "VARIATION SELECTOR-9", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE09}', codepoint: 0xFE09, name: "VARIATION SELECTOR-10", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE0A}', codepoint: 0xFE0A, name: "VARIATION SELECTOR-11", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE0B}', codepoint: 0xFE0B, name: "VARIATION SELECTOR-12", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE0C}', codepoint: 0xFE0C, name: "VARIATION SELECTOR-13", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE0D}', codepoint: 0xFE0D, name: "VARIATION SELECTOR-14", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE0E}', codepoint: 0xFE0E, name: "VARIATION SELECTOR-15", category: "变体选择器" },
        ZeroWidthChar { ch: '\u{FE0F}', codepoint: 0xFE0F, name: "VARIATION SELECTOR-16", category: "变体选择器" },

        // --- 已弃用的格式字符 ---
        ZeroWidthChar { ch: '\u{206A}', codepoint: 0x206A, name: "INHIBIT SYMMETRIC SWAPPING", category: "已弃用格式" },
        ZeroWidthChar { ch: '\u{206B}', codepoint: 0x206B, name: "ACTIVATE SYMMETRIC SWAPPING", category: "已弃用格式" },
        ZeroWidthChar { ch: '\u{206C}', codepoint: 0x206C, name: "INHIBIT ARABIC FORM SHAPING", category: "已弃用格式" },
        ZeroWidthChar { ch: '\u{206D}', codepoint: 0x206D, name: "ACTIVATE ARABIC FORM SHAPING", category: "已弃用格式" },
        ZeroWidthChar { ch: '\u{206E}', codepoint: 0x206E, name: "NATIONAL DIGIT SHAPES", category: "已弃用格式" },
        ZeroWidthChar { ch: '\u{206F}', codepoint: 0x206F, name: "NOMINAL DIGIT SHAPES", category: "已弃用格式" },

        // --- 行/段落分隔符 ---
        ZeroWidthChar { ch: '\u{2028}', codepoint: 0x2028, name: "LINE SEPARATOR", category: "分隔符" },
        ZeroWidthChar { ch: '\u{2029}', codepoint: 0x2029, name: "PARAGRAPH SEPARATOR", category: "分隔符" },
    ]
}

/// 判断字符是否是已知的零宽/不可见字符
pub fn is_zero_width(ch: char) -> bool {
    let cp = ch as u32;
    matches!(cp,
        0x200B..=0x200F |
        0x202A..=0x202E |
        0x2060..=0x2064 |
        0x2066..=0x2069 |
        0xFEFF |
        0x180E |
        0x00AD |
        0x034F |
        0x061C |
        0x115F..=0x1160 |
        0x17B4..=0x17B5 |
        0x3164 |
        0xFFA0 |
        0xFE00..=0xFE0F |
        0x206A..=0x206F |
        0x2028..=0x2029
    ) || is_unicode_tag(ch)
}

/// 判断字符是否是 Unicode Tag
pub fn is_unicode_tag(ch: char) -> bool {
    let cp = ch as u32;
    cp >= UNICODE_TAGS_START && cp <= UNICODE_TAGS_END
}
