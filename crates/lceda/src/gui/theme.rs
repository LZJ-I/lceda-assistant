//! 苹果风格浅色磨砂主题：系统蓝、大圆角、半透明卡片。

use eframe::egui::{self, Color32, CornerRadius, FontDefinitions, FontFamily, FontId, Stroke, Visuals};

pub const ACCENT: Color32 = Color32::from_rgb(0, 122, 255);
pub const LABEL: Color32 = Color32::from_rgb(29, 29, 31);
pub const SECONDARY: Color32 = Color32::from_rgb(134, 134, 139);

pub fn fill() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 168)
}
pub fn fill_strong() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 210)
}
pub fn hairline() -> Color32 {
    Color32::from_rgba_unmultiplied(0, 0, 0, 22)
}

pub fn apply(ctx: &egui::Context) {
    install_cjk_fonts(ctx);

    let mut visuals = Visuals::light();
    visuals.window_fill = Color32::TRANSPARENT;
    visuals.panel_fill = Color32::from_rgba_unmultiplied(245, 245, 247, 80);
    visuals.extreme_bg_color = Color32::from_rgba_unmultiplied(255, 255, 255, 140);
    visuals.faint_bg_color = Color32::from_rgba_unmultiplied(255, 255, 255, 90);
    visuals.widgets.inactive.bg_fill = fill();
    visuals.widgets.inactive.weak_bg_fill = fill();
    visuals.widgets.hovered.bg_fill = fill_strong();
    visuals.widgets.active.bg_fill = Color32::from_rgb(0, 122, 255);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, LABEL);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(10);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(10);
    visuals.widgets.active.corner_radius = CornerRadius::same(10);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(12);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, hairline());
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 122, 255, 40);
    visuals.hyperlink_color = ACCENT;
    visuals.window_corner_radius = CornerRadius::same(16);
    visuals.window_stroke = Stroke::new(1.0_f32, hairline());
    visuals.window_shadow.blur = 24;
    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(14);
        style.text_styles.insert(
            egui::TextStyle::Heading,
            FontId::new(22.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(14.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
    });
}

fn install_cjk_fonts(ctx: &egui::Context) {
    let candidates = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\msyh.ttf",
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert(
                "cjk".into(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "cjk".into());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("cjk".into());
            ctx.set_fonts(fonts);
            break;
        }
    }
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(fill())
        .stroke(Stroke::new(1.0_f32, hairline()))
        .corner_radius(14)
        .inner_margin(egui::Margin::same(12))
        .shadow(egui::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(18),
        })
}

pub fn pill_button(ui: &mut egui::Ui, text: &str, enabled: bool, filled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(egui::RichText::new(text).color(if filled {
            Color32::WHITE
        } else {
            LABEL
        }))
        .fill(if filled { ACCENT } else { fill_strong() })
        .stroke(Stroke::new(1.0_f32, if filled { ACCENT } else { hairline() }))
        .corner_radius(20)
        .min_size(egui::vec2(0.0, 34.0)),
    )
}
