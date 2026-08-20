use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SearchItem {
    pub index: usize,
    pub display_title: String,
    pub title: String,
    pub manufacturer: String,
    pub model_uuid: Option<String>,
    pub raw: Value,
}

impl SearchItem {
    pub fn name(&self) -> &str {
        if !self.display_title.is_empty() {
            &self.display_title
        } else if !self.title.is_empty() {
            &self.title
        } else {
            "component"
        }
    }

    pub fn uuid(&self) -> Option<&str> {
        self.raw.get("uuid").and_then(Value::as_str)
    }

    pub fn image_url(&self) -> Option<String> {
        if let Some(images) = self.raw.get("images").and_then(Value::as_array) {
            if let Some(first) = images.first().and_then(Value::as_str) {
                return normalize_url(first);
            }
        }
        self.raw
            .get("creator")
            .and_then(|c| c.get("avatar"))
            .and_then(Value::as_str)
            .and_then(normalize_url)
    }

    pub fn symbol_uuid(&self) -> Option<String> {
        nested_uuid(&self.raw, "symbol").or_else(|| attr_string(&self.raw, "Symbol"))
    }

    pub fn footprint_uuid(&self) -> Option<String> {
        nested_uuid(&self.raw, "footprint").or_else(|| attr_string(&self.raw, "Footprint"))
    }

    pub fn has_symbol_or_footprint(&self) -> bool {
        self.symbol_uuid().is_some() || self.footprint_uuid().is_some()
    }

    pub fn lcsc_id(&self) -> Option<String> {
        const KEYS: &[&str] = &[
            "LCSC Part",
            "LCSC Part Number",
            "LCSC",
            "Supplier Part",
            "Supplier Part Number",
            "JLCPCB Part Number",
            "JLCPCB Part",
        ];
        for key in KEYS {
            if let Some(id) = attr_string(&self.raw, key).and_then(|s| extract_lcsc(&s)) {
                return Some(id);
            }
        }
        for key in ["number", "code", "szlcsc"] {
            if let Some(id) = string_or_num(&self.raw, key).and_then(|s| extract_lcsc(&s)) {
                return Some(id);
            }
        }
        if let Some(attrs) = self.raw.get("attributes").and_then(Value::as_object) {
            for v in attrs.values() {
                let s = match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    _ => continue,
                };
                if looks_like_lcsc_token(&s) {
                    if let Some(id) = extract_lcsc(&s) {
                        return Some(id);
                    }
                }
            }
        }
        extract_lcsc(self.name()).or_else(|| extract_lcsc(&self.title))
    }

    pub fn datasheet_url(&self) -> Option<String> {
        const KEYS: &[&str] = &["Datasheet", "datasheet", "PDF", "Supplier Datasheet"];
        for key in KEYS {
            if let Some(url) = attr_string(&self.raw, key).and_then(|s| normalize_url(&s)) {
                if url.starts_with("http") {
                    return Some(url);
                }
            }
        }
        None
    }

    pub fn product_url(&self) -> String {
        if let Some(id) = self.lcsc_id() {
            format!("https://www.szlcsc.com/search?keyword={id}")
        } else {
            format!(
                "https://www.szlcsc.com/search?keyword={}",
                urlencoding::encode(self.name())
            )
        }
    }

    pub fn meta(&self) -> crate::ir::PartMeta {
        crate::ir::PartMeta {
            lcsc: self.lcsc_id().unwrap_or_default(),
            mpn: self.name().to_string(),
            manufacturer: self.manufacturer.clone(),
            datasheet: self.datasheet_url().unwrap_or_default(),
            footprint_lib: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DownloadPaths {
    pub step: Option<PathBuf>,
    pub obj: Option<PathBuf>,
    pub mtl: Option<PathBuf>,
    pub symbol_json: Option<PathBuf>,
    pub footprint_json: Option<PathBuf>,
    pub schlib: Option<PathBuf>,
    pub pcblib: Option<PathBuf>,
    pub kicad_sym: Option<PathBuf>,
    pub kicad_mod: Option<PathBuf>,
    pub datasheet: Option<PathBuf>,
}

fn nested_uuid(raw: &Value, key: &str) -> Option<String> {
    raw.get(key)
        .and_then(|v| v.get("uuid"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn attr_string(raw: &Value, key: &str) -> Option<String> {
    raw.get("attributes")
        .and_then(|a| a.get(key))
        .and_then(json_to_string)
}

fn string_or_num(raw: &Value, key: &str) -> Option<String> {
    raw.get(key).and_then(json_to_string)
}

fn json_to_string(v: &Value) -> Option<String> {
    v.as_str()
        .map(str::to_string)
        .or_else(|| v.as_i64().map(|n| n.to_string()))
        .filter(|s| !s.is_empty())
}

pub fn normalize_url(url: &str) -> Option<String> {
    let value = url.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(rest) = value.strip_prefix("//") {
        return Some(format!("https://{rest}"));
    }
    Some(value.to_string())
}

pub fn join_dir(dir: impl AsRef<Path>, name: &str) -> PathBuf {
    dir.as_ref().join(name)
}

pub fn extract_lcsc(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'C' || c == b'c' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j - i >= 4 {
                let digits = &text[i + 1..j];
                return Some(format!("C{digits}"));
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    None
}

pub fn looks_like_lcsc_token(s: &str) -> bool {
    let t = s.trim();
    if t.len() < 4 {
        return false;
    }
    let b = t.as_bytes();
    (b[0] == b'C' || b[0] == b'c') && t[1..].bytes().all(|c| c.is_ascii_digit())
}

pub fn parse_id_list(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("//"))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_lcsc_numbers() {
        assert_eq!(extract_lcsc("C2040").as_deref(), Some("C2040"));
        assert_eq!(extract_lcsc("part C8755 SOP").as_deref(), Some("C8755"));
        assert!(extract_lcsc("TYPE-C").is_none());
    }
}
