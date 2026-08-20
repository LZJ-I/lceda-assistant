use super::theme::{self, ACCENT, LABEL, SECONDARY};
use crate::i18n::{self, Lang};
use eframe::egui::{self, Color32, ColorImage, TextureHandle};
use lceda_core::client::LcedaClient;
use lceda_core::export::{ExportRequest, export};
use lceda_core::mesh::{self, Mesh};
use lceda_core::models::SearchItem;
use poll_promise::Promise;
use std::path::PathBuf;

type SearchPromise = Promise<lceda_core::Result<Vec<SearchItem>>>;
type ExportPromise = Promise<lceda_core::Result<String>>;
type PreviewPromise = Promise<PreviewData>;

struct PreviewData {
    image: Option<Vec<u8>>,
    mesh: Option<Mesh>,
}

pub fn run(lang: Lang) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(i18n::t(lang, "app_title"))
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 580.0])
            .with_transparent(true),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "lceda",
        options,
        Box::new(move |cc| {
            apply_vibrancy(cc);
            theme::apply(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(lang)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}

fn apply_vibrancy(_cc: &eframe::CreationContext<'_>) {
    // eframe 0.32 创建回调里拿不到稳定窗口句柄；外观靠半透明卡片。
}

struct App {
    lang: Lang,
    keyword: String,
    out_dir: String,
    force: bool,
    items: Vec<SearchItem>,
    selected: Option<usize>,
    status: String,
    logs: Vec<String>,
    search: Option<SearchPromise>,
    job: Option<ExportPromise>,
    preview: Option<PreviewPromise>,
    image_tex: Option<TextureHandle>,
    mesh: Option<Mesh>,
    orbit: f32,
}

impl App {
    fn new(lang: Lang) -> Self {
        Self {
            lang,
            keyword: String::new(),
            out_dir: default_out_dir(),
            force: false,
            items: Vec::new(),
            selected: None,
            status: i18n::t(lang, "ready").into(),
            logs: vec![i18n::t(lang, "no_dotnet").into()],
            search: None,
            job: None,
            preview: None,
            image_tex: None,
            mesh: None,
            orbit: 0.35,
        }
    }

    fn log(&mut self, msg: impl Into<String>) {
        self.logs.push(msg.into());
        if self.logs.len() > 200 {
            self.logs.remove(0);
        }
    }

    fn busy(&self) -> bool {
        self.search.is_some() || self.job.is_some()
    }

    fn selected_item(&self) -> Option<&SearchItem> {
        self.selected.and_then(|i| self.items.get(i))
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.96, 0.96, 0.97, 0.86]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_jobs(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(16.0).fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new(i18n::t(self.lang, "app_title")).color(LABEL));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let label = if self.lang == Lang::Zh { "中文 / EN" } else { "EN / 中文" };
                        if theme::pill_button(ui, label, true, false).clicked() {
                            self.lang = self.lang.toggle();
                        }
                    });
                });
                ui.add_space(6.0);
                ui.label(egui::RichText::new(i18n::t(self.lang, "no_dotnet")).color(SECONDARY).small());
                ui.add_space(10.0);

                ui.columns(2, |cols| {
                    self.left_pane(&mut cols[0], ctx);
                    self.right_pane(&mut cols[1], ctx);
                });
            });
    }
}

impl App {
    fn poll_jobs(&mut self, ctx: &egui::Context) {
        if let Some(p) = &self.search {
            if p.ready().is_some() {
                match self.search.take().unwrap().block_and_take() {
                    Ok(items) => {
                        self.log(format!("{}: {}", i18n::t(self.lang, "search"), items.len()));
                        self.items = items;
                        self.selected = if self.items.is_empty() { None } else { Some(0) };
                        if self.items.is_empty() {
                            self.log(i18n::t(self.lang, "no_results"));
                        } else {
                            self.queue_preview(ctx);
                        }
                    }
                    Err(e) => self.log(format!("{}: {e}", i18n::t(self.lang, "error"))),
                }
                self.status = i18n::t(self.lang, "ready").into();
            } else {
                ctx.request_repaint();
            }
        }
        if let Some(p) = &self.job {
            if p.ready().is_some() {
                match self.job.take().unwrap().block_and_take() {
                    Ok(msg) => {
                        self.log(msg);
                        self.log(i18n::t(self.lang, "json_kept"));
                    }
                    Err(e) => {
                        self.log(format!("{}: {e}", i18n::t(self.lang, "error")));
                        self.log(i18n::t(self.lang, "json_kept"));
                    }
                }
                self.status = i18n::t(self.lang, "ready").into();
            } else {
                ctx.request_repaint();
            }
        }
        if let Some(p) = &self.preview {
            if p.ready().is_some() {
                let data = self.preview.take().unwrap().block_and_take();
                self.image_tex = data.image.as_ref().and_then(|b| load_texture(ctx, b));
                self.mesh = data.mesh;
            } else {
                ctx.request_repaint();
            }
        }
    }

    fn left_pane(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        theme::card_frame().show(ui, |ui| {
            ui.set_min_height(ui.available_height());
            ui.label(egui::RichText::new(i18n::t(self.lang, "keyword")).strong().color(LABEL));
            ui.horizontal(|ui| {
                let edit = egui::TextEdit::singleline(&mut self.keyword)
                    .hint_text(i18n::t(self.lang, "keyword_hint"))
                    .desired_width(ui.available_width() - 88.0);
                let resp = ui.add(edit);
                let go = theme::pill_button(ui, i18n::t(self.lang, "search"), !self.busy(), true)
                    .clicked();
                if (go || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                    && !self.busy()
                {
                    self.do_search();
                }
            });
            ui.add_space(8.0);
            ui.label(egui::RichText::new(i18n::t(self.lang, "components")).strong());
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut clicked = None;
                for (idx, item) in self.items.iter().enumerate() {
                    let selected = self.selected == Some(idx);
                    let mut text = format!("{}  {}", item.index, item.name());
                    if let Some(id) = item.lcsc_id() {
                        text.push_str("  ");
                        text.push_str(&id);
                    }
                    if item.model_uuid.is_some() {
                        text.push_str("  · 3D");
                    }
                    let resp = ui.selectable_label(selected, text);
                    if resp.clicked() {
                        clicked = Some(idx);
                    }
                }
                if let Some(idx) = clicked {
                    self.selected = Some(idx);
                    self.queue_preview(ctx);
                }
            });
        });
    }

    fn right_pane(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| {
            theme::card_frame().show(ui, |ui| {
                ui.set_min_height(220.0);
                ui.label(egui::RichText::new(i18n::t(self.lang, "preview")).strong());
                if let Some(tex) = &self.image_tex {
                    let size = tex.size_vec2();
                    let scale = (ui.available_width() / size.x).min(200.0 / size.y).min(1.0);
                    ui.image((tex.id(), size * scale));
                } else {
                    ui.colored_label(SECONDARY, "—");
                }
                if let Some(item) = self.selected_item() {
                    ui.label(egui::RichText::new(item.name()).color(LABEL).size(16.0));
                    if let Some(id) = item.lcsc_id() {
                        ui.label(egui::RichText::new(id).color(ACCENT).small());
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            "{}  {}",
                            i18n::t(self.lang, "manufacturer"),
                            if item.manufacturer.is_empty() {
                                "—"
                            } else {
                                &item.manufacturer
                            }
                        ))
                        .color(SECONDARY)
                        .small(),
                    );
                }
            });

            ui.add_space(8.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_min_height(180.0);
                ui.label(egui::RichText::new(i18n::t(self.lang, "model3d")).strong());
                let (resp, painter) =
                    ui.allocate_painter(egui::vec2(ui.available_width(), 160.0), egui::Sense::drag());
                if resp.dragged() {
                    self.orbit += resp.drag_delta().x * 0.01;
                }
                if let Some(mesh) = &self.mesh {
                    paint_mesh(&painter, resp.rect, mesh, self.orbit);
                } else {
                    painter.text(
                        resp.rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "—",
                        egui::FontId::proportional(14.0),
                        SECONDARY,
                    );
                }
            });

            ui.add_space(8.0);
            theme::card_frame().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let has3d = self.selected_item().map(|i| i.model_uuid.is_some()).unwrap_or(false);
                    let has_ad = self
                        .selected_item()
                        .map(|i| i.has_symbol_or_footprint())
                        .unwrap_or(false);
                    let has_item = self.selected_item().is_some();
                    let en = !self.busy() && has_item;
                    if theme::pill_button(ui, i18n::t(self.lang, "download_step"), en && has3d, true).clicked()
                    {
                        self.run_export(|req| req.step = true);
                    }
                    if theme::pill_button(ui, i18n::t(self.lang, "download_obj"), en && has3d, false).clicked()
                    {
                        self.run_export(|req| req.obj = true);
                    }
                    if theme::pill_button(ui, i18n::t(self.lang, "export_ad"), en && has_ad, true).clicked() {
                        self.run_export(|req| {
                            req.ad = true;
                            req.source_json = true;
                        });
                    }
                    if theme::pill_button(ui, i18n::t(self.lang, "export_kicad"), en && has_ad, true).clicked() {
                        self.run_export(|req| {
                            req.kicad = true;
                            req.source_json = true;
                        });
                    }
                    if theme::pill_button(ui, i18n::t(self.lang, "export_source"), en && has_ad, false).clicked()
                    {
                        self.run_export(|req| req.source_json = true);
                    }
                    if theme::pill_button(ui, i18n::t(self.lang, "datasheet"), en, false).clicked() {
                        self.run_export(|req| req.datasheet = true);
                    }
                    if theme::pill_button(ui, i18n::t(self.lang, "open_page"), has_item, false).clicked() {
                        if let Some(it) = self.selected_item() {
                            let url = it.product_url();
                            if webbrowser::open(&url).is_ok() {
                                self.log(format!("{} {url}", i18n::t(self.lang, "opened")));
                            }
                        }
                    }
                    if theme::pill_button(ui, i18n::t(self.lang, "batch"), !self.busy(), false).clicked() {
                        self.run_batch();
                    }
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(i18n::t(self.lang, "output"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.out_dir).desired_width(ui.available_width() - 90.0),
                    );
                    if ui.button(i18n::t(self.lang, "browse")).clicked() {
                        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                            self.out_dir = dir.display().to_string();
                        }
                    }
                });
                ui.checkbox(&mut self.force, i18n::t(self.lang, "overwrite"));
                ui.colored_label(ACCENT, &self.status);
            });

            ui.add_space(8.0);
            theme::card_frame().show(ui, |ui| {
                ui.label(egui::RichText::new(i18n::t(self.lang, "log")).strong());
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(140.0)
                    .show(ui, |ui| {
                        for line in &self.logs {
                            ui.monospace(line);
                        }
                    });
            });
        });
        let _ = ctx;
    }

    fn do_search(&mut self) {
        let kw = self.keyword.trim().to_string();
        if kw.is_empty() {
            self.log(i18n::t(self.lang, "empty_keyword"));
            return;
        }
        self.status = i18n::t(self.lang, "searching").into();
        self.search = Some(Promise::spawn_thread("search", move || {
            LcedaClient::new().search(&kw)
        }));
    }

    fn queue_preview(&mut self, _ctx: &egui::Context) {
        let Some(item) = self.selected_item().cloned() else {
            return;
        };
        self.image_tex = None;
        self.mesh = None;
        self.preview = Some(Promise::spawn_thread("preview", move || {
            let client = LcedaClient::new();
            let mut data = PreviewData {
                image: None,
                mesh: None,
            };
            if let Some(url) = item.image_url() {
                data.image = client.get_bytes(&url).ok();
            }
            if item.model_uuid.is_some() {
                if let Ok(bytes) = client.download_obj_bytes(&item) {
                    let text = String::from_utf8_lossy(&bytes);
                    let mesh = mesh::parse_obj(&text, 80_000);
                    let faces = mesh::decimate(&mesh, 4_000);
                    data.mesh = Some(Mesh {
                        vertices: mesh.vertices,
                        triangles: faces,
                    });
                }
            }
            data
        }));
    }

    fn run_export(&mut self, mutate: impl FnOnce(&mut ExportRequest)) {
        let Some(item) = self.selected_item().cloned() else {
            self.log(i18n::t(self.lang, "select_first"));
            return;
        };
        let mut req = ExportRequest {
            force: self.force,
            out_dir: PathBuf::from(self.out_dir.trim()),
            ..Default::default()
        };
        mutate(&mut req);
        self.status = i18n::t(self.lang, "working").into();
        self.job = Some(Promise::spawn_thread("export", move || {
            let client = LcedaClient::new();
            let paths = export(&client, &item, &req)?;
            Ok(format_paths(&paths))
        }));
    }

    fn run_batch(&mut self) {
        let Some(file) = rfd::FileDialog::new()
            .add_filter("text", &["txt", "csv", "list"])
            .pick_file()
        else {
            return;
        };
        let force = self.force;
        let out_dir = PathBuf::from(self.out_dir.trim());
        self.status = i18n::t(self.lang, "working").into();
        self.job = Some(Promise::spawn_thread("batch", move || {
            let text = std::fs::read_to_string(&file).map_err(|e| {
                lceda_core::Error::msg(format!("read {}: {e}", file.display()))
            })?;
            let ids = lceda_core::models::parse_id_list(&text);
            let req = ExportRequest {
                step: true,
                ad: true,
                kicad: true,
                source_json: true,
                force,
                out_dir,
                ..Default::default()
            };
            let client = LcedaClient::new();
            let mut lines = Vec::new();
            let mut fail = 0;
            for (kw, result) in lceda_core::export::export_batch(&client, &ids, &req) {
                match result {
                    Ok(paths) => lines.push(format!("OK {kw}\n{}", format_paths(&paths))),
                    Err(e) => {
                        fail += 1;
                        lines.push(format!("FAIL {kw}: {e}"));
                    }
                }
            }
            lines.push(format!("done, failed={fail}"));
            Ok(lines.join("\n"))
        }));
    }
}

fn format_paths(paths: &lceda_core::models::DownloadPaths) -> String {
    let mut lines = Vec::new();
    if let Some(p) = &paths.step {
        lines.push(format!("STEP {}", p.display()));
    }
    if let Some(p) = &paths.obj {
        lines.push(format!("OBJ {}", p.display()));
    }
    if let Some(p) = &paths.datasheet {
        lines.push(format!("PDF {}", p.display()));
    }
    if let Some(p) = &paths.schlib {
        lines.push(format!("SchLib {}", p.display()));
    }
    if let Some(p) = &paths.pcblib {
        lines.push(format!("PcbLib {}", p.display()));
    }
    if let Some(p) = &paths.kicad_sym {
        lines.push(format!("KiCad symbol {}", p.display()));
    }
    if let Some(p) = &paths.kicad_mod {
        lines.push(format!("KiCad footprint {}", p.display()));
    }
    if let Some(p) = &paths.symbol_json {
        lines.push(format!("symbol JSON {}", p.display()));
    }
    if let Some(p) = &paths.footprint_json {
        lines.push(format!("footprint JSON {}", p.display()));
    }
    if lines.is_empty() {
        "done".into()
    } else {
        lines.join("\n")
    }
}

fn default_out_dir() -> String {
    directories::UserDirs::new()
        .and_then(|u| u.document_dir().map(|p| p.join("lceda-out").display().to_string()))
        .unwrap_or_else(|| "out".into())
}

fn load_texture(ctx: &egui::Context, bytes: &[u8]) -> Option<TextureHandle> {
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture("part", color, Default::default()))
}

fn paint_mesh(painter: &egui::Painter, rect: egui::Rect, mesh: &Mesh, orbit: f32) {
    if mesh.vertices.is_empty() || mesh.triangles.is_empty() {
        return;
    }
    let (min, max) = mesh.vertices.iter().fold(
        ([f32::MAX; 3], [f32::MIN; 3]),
        |(mut min, mut max), v| {
            for i in 0..3 {
                min[i] = min[i].min(v[i]);
                max[i] = max[i].max(v[i]);
            }
            (min, max)
        },
    );
    let cx = (min[0] + max[0]) * 0.5;
    let cy = (min[1] + max[1]) * 0.5;
    let cz = (min[2] + max[2]) * 0.5;
    let span = (max[0] - min[0]).max(max[1] - min[1]).max(max[2] - min[2]).max(1.0);
    let cos = orbit.cos();
    let sin = orbit.sin();
    let origin = rect.center();
    let scale = rect.height().min(rect.width()) * 0.42;
    for tri in &mesh.triangles {
        let mut pts = [egui::Pos2::ZERO; 3];
        for (i, idx) in tri.iter().enumerate() {
            let v = mesh.vertices.get(*idx as usize).copied().unwrap_or([0.0; 3]);
            let x = (v[0] - cx) / span;
            let y = (v[1] - cy) / span;
            let z = (v[2] - cz) / span;
            let xr = x * cos - z * sin;
            let zr = x * sin + z * cos;
            pts[i] = egui::pos2(origin.x + xr * scale, origin.y - (y * 0.85 + zr * 0.35) * scale);
        }
        painter.line_segment([pts[0], pts[1]], egui::Stroke::new(0.6_f32, Color32::from_rgb(90, 110, 140)));
        painter.line_segment([pts[1], pts[2]], egui::Stroke::new(0.6_f32, Color32::from_rgb(90, 110, 140)));
        painter.line_segment([pts[2], pts[0]], egui::Stroke::new(0.6_f32, Color32::from_rgb(90, 110, 140)));
    }
}
