use std::env;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

impl Lang {
    pub fn from_code(code: &str) -> Self {
        if code.to_ascii_lowercase().starts_with("zh") {
            Self::Zh
        } else {
            Self::En
        }
    }

    pub fn detect() -> Self {
        for key in ["LCEDA_LANG", "LANG", "LC_ALL"] {
            if let Ok(v) = env::var(key) {
                if !v.is_empty() {
                    return Self::from_code(&v);
                }
            }
        }
        Self::Zh
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Zh => Self::En,
            Self::En => Self::Zh,
        }
    }
}

pub fn t(lang: Lang, key: &str) -> &'static str {
    match (lang, key) {
        (Lang::Zh, "app_title") => "立创封装助手",
        (Lang::En, "app_title") => "LCSC Parts",
        (Lang::Zh, "search") => "搜索",
        (Lang::En, "search") => "Search",
        (Lang::Zh, "keyword") => "型号 / 立创编号",
        (Lang::En, "keyword") => "MPN or LCSC ID",
        (Lang::Zh, "keyword_hint") => "例如 C8755 或 TYPE-C",
        (Lang::En, "keyword_hint") => "e.g. C8755 or TYPE-C",
        (Lang::Zh, "components") => "器件",
        (Lang::En, "components") => "Parts",
        (Lang::Zh, "preview") => "预览",
        (Lang::En, "preview") => "Preview",
        (Lang::Zh, "model3d") => "三维外形",
        (Lang::En, "model3d") => "3D outline",
        (Lang::Zh, "download_step") => "下载 STEP",
        (Lang::En, "download_step") => "Download STEP",
        (Lang::Zh, "download_obj") => "下载 OBJ",
        (Lang::En, "download_obj") => "Download OBJ",
        (Lang::Zh, "export_ad") => "导出 AD 库",
        (Lang::En, "export_ad") => "Export Altium",
        (Lang::Zh, "export_kicad") => "导出 KiCad",
        (Lang::En, "export_kicad") => "Export KiCad",
        (Lang::Zh, "datasheet") => "数据手册",
        (Lang::En, "datasheet") => "Datasheet",
        (Lang::Zh, "open_page") => "打开立创页",
        (Lang::En, "open_page") => "Open LCSC",
        (Lang::Zh, "batch") => "批量文件…",
        (Lang::En, "batch") => "Batch file…",
        (Lang::Zh, "export_source") => "仅源文件",
        (Lang::En, "export_source") => "JSON only",
        (Lang::Zh, "output") => "输出目录",
        (Lang::En, "output") => "Output folder",
        (Lang::Zh, "browse") => "选择…",
        (Lang::En, "browse") => "Browse…",
        (Lang::Zh, "overwrite") => "覆盖已有文件",
        (Lang::En, "overwrite") => "Overwrite",
        (Lang::Zh, "ready") => "就绪",
        (Lang::En, "ready") => "Ready",
        (Lang::Zh, "searching") => "正在搜索…",
        (Lang::En, "searching") => "Searching…",
        (Lang::Zh, "working") => "处理中…",
        (Lang::En, "working") => "Working…",
        (Lang::Zh, "log") => "日志",
        (Lang::En, "log") => "Log",
        (Lang::Zh, "manufacturer") => "厂牌",
        (Lang::En, "manufacturer") => "Mfr",
        (Lang::Zh, "has3d") => "3D",
        (Lang::En, "has3d") => "3D",
        (Lang::Zh, "no_results") => "没有找到器件",
        (Lang::En, "no_results") => "No parts found",
        (Lang::Zh, "empty_keyword") => "请输入关键字",
        (Lang::En, "empty_keyword") => "Please enter a keyword",
        (Lang::Zh, "select_first") => "请先选择器件",
        (Lang::En, "select_first") => "Select a part first",
        (Lang::Zh, "done") => "完成",
        (Lang::En, "done") => "Done",
        (Lang::Zh, "error") => "出错",
        (Lang::En, "error") => "Error",
        (Lang::Zh, "no_dotnet") => "进程内写出 Altium / KiCad 库，无需 .NET。",
        (Lang::En, "no_dotnet") => "Writes Altium and KiCad libraries natively — no .NET SDK.",
        (Lang::Zh, "frost_hint") => "磨砂玻璃外观（Windows Acrylic / macOS Vibrancy）",
        (Lang::En, "frost_hint") => "Frosted glass (Windows Acrylic / macOS Vibrancy)",
        (Lang::Zh, "batch_done") => "批量完成",
        (Lang::En, "batch_done") => "Batch done",
        (Lang::Zh, "opened") => "已在浏览器打开",
        (Lang::En, "opened") => "Opened in browser",
        (Lang::Zh, "json_kept") => "已保留 EasyEDA 源 JSON，可对照检查。",
        (Lang::En, "json_kept") => "EasyEDA source JSON was kept for inspection.",
        _ => "???",
    }
}
