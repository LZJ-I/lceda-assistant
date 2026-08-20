use std::path::Path;

pub fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim_matches([' ', '.'].as_slice());
    if trimmed.is_empty() {
        "component".into()
    } else {
        trimmed.to_string()
    }
}

/// Altium compound storage names are 31 chars, ASCII-ish.
pub fn altium_section_key(name: &str) -> String {
    let ascii: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else if c == '/' {
                '_'
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = ascii.trim_matches('_');
    let base = if trimmed.is_empty() { "component" } else { trimmed };
    base.chars().take(31).collect()
}

pub fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

pub fn looks_like_step(bytes: &[u8]) -> bool {
    let head = std::str::from_utf8(bytes.get(..64).unwrap_or(bytes)).unwrap_or("");
    head.contains("ISO-10303") || head.contains("STEP")
}

pub fn unique_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(1);
    let mut x = nanos as u64 ^ 0x9E37_79B9_7F4A_7C15;
    let mut out = String::with_capacity(8);
    for _ in 0..8 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        let c = b'A' + (x % 26) as u8;
        out.push(c as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_bad_chars() {
        assert_eq!(sanitize_filename(r#"A<>:"/\|?*B"#), "A_________B");
        assert_eq!(sanitize_filename("   "), "component");
    }
}
