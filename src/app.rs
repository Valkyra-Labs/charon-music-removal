//! The window.

use crate::engine::{Engine, Event, Preview, PEAKS_PER_SECOND};
use crate::media::MediaInfo;
use crate::playback::{Cut, Listen, Player, Tracks};
use charon_audio::AudioBuffer;
use egui::{Color32, Rect, Sense, Stroke, TextureHandle, TextureOptions, Vec2};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const MUSIC: Color32 = Color32::from_rgb(217, 130, 43);
const WAVE: Color32 = Color32::from_rgb(150, 146, 138);
const ACCENT: Color32 = Color32::from_rgb(64, 150, 120);

/// Removal depth of a region: the fraction of the music estimate that is
/// subtracted.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Depth {
    Full,
    Minus30,
    Minus20,
    Keep,
}

impl Depth {
    const ALL: [Depth; 4] = [Depth::Full, Depth::Minus30, Depth::Minus20, Depth::Keep];

    fn amount(self) -> f32 {
        match self {
            Depth::Full => 1.0,
            Depth::Minus30 => 1.0 - 10f32.powf(-30.0 / 20.0),
            Depth::Minus20 => 1.0 - 10f32.powf(-20.0 / 20.0),
            Depth::Keep => 0.0,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Depth::Full => "Remove",
            Depth::Minus30 => "-30 dB",
            Depth::Minus20 => "-20 dB",
            Depth::Keep => "Keep",
        }
    }
}

struct RegionUi {
    start: usize,
    end: usize,
    depth: Depth,
}

struct Loaded {
    path: PathBuf,
    info: MediaInfo,
    audio: Arc<AudioBuffer>,
    peaks: Vec<f32>,
    removed: Option<Arc<AudioBuffer>>,
    removed_peaks: Vec<f32>,
    regions: Vec<RegionUi>,
    selected: Option<usize>,
    preview: Option<Preview>,
    texture: Option<TextureHandle>,
    texture_gen: u64,
}

enum Busy {
    Idle,
    Working { stage: &'static str, fraction: f32 },
}

pub struct App {
    engine: Engine,
    player: Option<Player>,
    loaded: Option<Loaded>,
    busy: Busy,
    status: String,
    show_about: bool,
    /// Start the analysis as soon as a file is loaded.
    auto_analyze: bool,
    model_found: bool,
    model_path: PathBuf,
}

fn fmt_time(seconds: f64) -> String {
    let s = seconds.max(0.0);
    format!("{}:{:04.1}", (s / 60.0) as u64, s % 60.0)
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        model: PathBuf,
        open: Option<PathBuf>,
        auto_analyze: bool,
    ) -> Self {
        let ctx = cc.egui_ctx.clone();
        let engine = Engine::start(model.clone(), move || ctx.request_repaint());
        let player = match Player::new() {
            Ok(p) => Some(p),
            Err(e) => {
                log::warn!("no audio output: {e}");
                None
            }
        };
        let model_found = model.exists();
        let mut app = Self {
            engine,
            player,
            loaded: None,
            busy: Busy::Idle,
            status: if model_found {
                "Open a video (MP4 or MOV) or drop it on the window.".into()
            } else {
                format!("Model not found at {}. See the README.", model.display())
            },
            show_about: false,
            auto_analyze,
            model_found,
            model_path: model,
        };
        if let Some(path) = open {
            app.open(path);
        }
        app
    }

    fn open(&mut self, path: PathBuf) {
        if let Some(p) = &self.player {
            p.set_playing(false);
            p.set_tracks(None);
            p.seek(0);
        }
        self.loaded = None;
        self.busy = Busy::Working {
            stage: "Reading audio",
            fraction: 0.0,
        };
        self.status = format!("Opening {}", path.display());
        self.engine.load(path);
    }

    fn start_analysis(&mut self) {
        if let Some(l) = &self.loaded {
            self.busy = Busy::Working {
                stage: "Finding music",
                fraction: 0.0,
            };
            self.engine.analyze(l.audio.clone());
        }
    }

    fn rebuild_tracks(&self) {
        let (Some(player), Some(l)) = (&self.player, &self.loaded) else {
            return;
        };
        let cuts = l
            .regions
            .iter()
            .map(|r| Cut {
                start: r.start,
                end: r.end,
                amount: r.depth.amount(),
            })
            .collect();
        player.set_tracks(Some(Arc::new(Tracks {
            input: l.audio.clone(),
            removed: l.removed.clone(),
            cuts,
        })));
    }

    fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.engine.events.try_recv() {
            match event {
                Event::Progress { stage, fraction } => {
                    self.busy = Busy::Working { stage, fraction };
                }
                Event::Loaded {
                    path,
                    info,
                    audio,
                    peaks,
                    seconds,
                } => {
                    let preview = info.video.map(|size| {
                        let ctx = ctx.clone();
                        Preview::start(&path, size, move || ctx.request_repaint())
                    });
                    self.status = format!(
                        "Read {:.0} s of audio in {:.1} s. Next: Find and remove music.",
                        audio.duration(),
                        seconds
                    );
                    self.loaded = Some(Loaded {
                        path,
                        info,
                        audio,
                        peaks,
                        removed: None,
                        removed_peaks: Vec::new(),
                        regions: Vec::new(),
                        selected: None,
                        preview,
                        texture: None,
                        texture_gen: 0,
                    });
                    self.busy = Busy::Idle;
                    self.rebuild_tracks();
                    if self.auto_analyze && self.model_found {
                        self.auto_analyze = false;
                        self.start_analysis();
                    }
                }
                Event::Detected { regions } => {
                    if let Some(l) = &mut self.loaded {
                        l.regions = regions
                            .iter()
                            .map(|r| RegionUi {
                                start: r.start,
                                end: r.end,
                                depth: Depth::Full,
                            })
                            .collect();
                        l.selected = None;
                    }
                }
                Event::Analyzed {
                    regions,
                    removed,
                    removed_peaks,
                    seconds,
                } => {
                    if let Some(l) = &mut self.loaded {
                        let rate = l.audio.sample_rate as f64;
                        let music: f64 = regions
                            .iter()
                            .map(|r| (r.end - r.start) as f64 / rate)
                            .sum();
                        self.status = if regions.is_empty() {
                            format!("No music found ({seconds:.0} s).")
                        } else {
                            format!(
                                "{} music region(s), {:.0} s of music, processed in {:.0} s. Listen, adjust, export.",
                                regions.len(),
                                music,
                                seconds
                            )
                        };
                        // Depths set while separation ran are kept.
                        let same = l.regions.len() == regions.len()
                            && l.regions
                                .iter()
                                .zip(&regions)
                                .all(|(u, r)| u.start == r.start && u.end == r.end);
                        if !same {
                            l.regions = regions
                                .iter()
                                .map(|r| RegionUi {
                                    start: r.start,
                                    end: r.end,
                                    depth: Depth::Full,
                                })
                                .collect();
                            l.selected = None;
                        }
                        l.removed = Some(removed);
                        l.removed_peaks = removed_peaks;
                    }
                    self.busy = Busy::Idle;
                    self.rebuild_tracks();
                    if let Some(p) = &self.player {
                        p.set_listen(Listen::Result);
                    }
                }
                Event::Exported { path, seconds } => {
                    self.busy = Busy::Idle;
                    self.status = format!("Saved {} in {:.1} s.", path.display(), seconds);
                }
                Event::Cancelled => {
                    self.busy = Busy::Idle;
                    self.status = "Cancelled.".into();
                }
                Event::Failed(e) => {
                    self.busy = Busy::Idle;
                    self.status = format!("Error: {e}");
                }
            }
        }
    }

    fn export(&mut self) {
        let Some(l) = &self.loaded else { return };
        let stem = l
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video")
            .to_string();
        let ext = l
            .path
            .extension()
            .and_then(|s| s.to_str())
            .map(str::to_ascii_lowercase)
            .filter(|e| e == "mov" || e == "mp4" || e == "m4v")
            .unwrap_or_else(|| "mp4".into());
        let Some(dest) = rfd::FileDialog::new()
            .set_file_name(format!("{stem} (music removed).{ext}"))
            .set_directory(l.path.parent().unwrap_or(Path::new(".")))
            .save_file()
        else {
            return;
        };
        if dest == l.path {
            self.status = "Choose a new file name; the original is never overwritten.".into();
            return;
        }
        let tracks = Tracks {
            input: l.audio.clone(),
            removed: l.removed.clone(),
            cuts: l
                .regions
                .iter()
                .map(|r| Cut {
                    start: r.start,
                    end: r.end,
                    amount: r.depth.amount(),
                })
                .collect(),
        };
        self.busy = Busy::Working {
            stage: "Writing the video",
            fraction: 0.0,
        };
        self.engine
            .export(l.path.clone(), tracks.render_result(), dest);
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let idle = matches!(self.busy, Busy::Idle);
        ui.horizontal(|ui| {
            if ui.add_enabled(idle, egui::Button::new("Open...")).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Video", &["mp4", "mov", "m4v"])
                    .pick_file()
                {
                    self.open(path);
                }
            }
            let can_analyze = idle && self.loaded.is_some() && self.model_found;
            if ui
                .add_enabled(can_analyze, egui::Button::new("Find and remove music"))
                .clicked()
            {
                self.start_analysis();
            }
            let can_export = idle && self.loaded.as_ref().is_some_and(|l| l.removed.is_some());
            if ui
                .add_enabled(can_export, egui::Button::new("Export video..."))
                .clicked()
            {
                self.export();
            }
            ui.separator();
            if let Some(l) = &self.loaded {
                let name = l.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let video = l
                    .info
                    .video
                    .map(|(w, h)| format!("{w}x{h}"))
                    .unwrap_or_else(|| "no video".into());
                ui.label(egui::RichText::new(name).strong());
                ui.label(format!("{}, {video}", fmt_time(l.info.duration)));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("About").clicked() {
                    self.show_about = true;
                }
                ui.label(
                    egui::RichText::new("Runs on this computer. No network.")
                        .color(ACCENT)
                        .small(),
                );
            });
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| match self.busy {
            Busy::Working { stage, fraction } => {
                ui.label(stage);
                ui.add(
                    egui::ProgressBar::new(fraction)
                        .desired_width(240.0)
                        .show_percentage(),
                );
                if ui.button("Cancel").clicked() {
                    self.engine.cancel();
                }
            }
            Busy::Idle => {
                ui.label(&self.status);
            }
        });
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Listen");
        if let Some(p) = &self.player {
            let mut listen = p.listen();
            ui.horizontal(|ui| {
                ui.selectable_value(&mut listen, Listen::Original, "Original [1]");
                ui.selectable_value(&mut listen, Listen::Result, "Result [2]");
                ui.selectable_value(&mut listen, Listen::Removed, "Removed [3]");
            });
            if listen != p.listen() {
                p.set_listen(listen);
            }
            ui.label(
                egui::RichText::new("\"Removed\" plays what was taken out, to check that no speech or effects went with the music.")
                    .small()
                    .weak(),
            );
        } else {
            ui.label("No audio output device.");
        }
        ui.separator();
        ui.heading("Music regions");
        let mut changed = false;
        let mut seek_to = None;
        if let Some(l) = &mut self.loaded {
            let rate = l.audio.sample_rate as f64;
            if l.regions.is_empty() {
                ui.label(if l.removed.is_some() {
                    "No music found."
                } else {
                    "Run \"Find and remove music\"."
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("All:");
                    for d in Depth::ALL {
                        if ui.small_button(d.label()).clicked() {
                            for r in &mut l.regions {
                                r.depth = d;
                            }
                            changed = true;
                        }
                    }
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, r) in l.regions.iter_mut().enumerate() {
                        let selected = l.selected == Some(i);
                        let title = format!(
                            "{} - {}",
                            fmt_time(r.start as f64 / rate),
                            fmt_time(r.end as f64 / rate)
                        );
                        egui::Frame::group(ui.style())
                            .stroke(if selected {
                                Stroke::new(1.5, MUSIC)
                            } else {
                                ui.visuals().widgets.noninteractive.bg_stroke
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if ui.link(title).clicked() {
                                        l.selected = Some(i);
                                        seek_to = Some(r.start);
                                    }
                                });
                                ui.horizontal(|ui| {
                                    for d in Depth::ALL {
                                        if ui.selectable_label(r.depth == d, d.label()).clicked() {
                                            r.depth = d;
                                            changed = true;
                                        }
                                    }
                                });
                            });
                    }
                });
            }
            ui.separator();
            ui.label(
                egui::RichText::new(
                    "Removing music lowers the chance of a claim; it does not guarantee anything. Use it for music you do not have rights to.",
                )
                .small()
                .weak(),
            );
        }
        if changed {
            self.rebuild_tracks();
        }
        if let (Some(s), Some(p)) = (seek_to, &self.player) {
            p.seek(s);
        }
    }

    fn preview(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, height: f32) {
        let pos = self.player.as_ref().map(|p| p.position()).unwrap_or(0);
        let playing = self.player.as_ref().is_some_and(|p| p.playing());
        let Some(l) = &mut self.loaded else {
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
            ui.painter().rect_filled(rect, 6.0, Color32::from_gray(18));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Drop a video here",
                egui::FontId::proportional(18.0),
                Color32::from_gray(150),
            );
            return;
        };
        let seconds = pos as f64 / l.audio.sample_rate as f64;
        if let Some(preview) = &l.preview {
            preview.follow(seconds, playing);
            if let Ok(mut slot) = preview.slot.try_lock() {
                if let Some((generation, frame)) = slot.take() {
                    if generation != l.texture_gen {
                        let image = egui::ColorImage::from_rgba_premultiplied(
                            [frame.width, frame.height],
                            &frame.rgba,
                        );
                        match &mut l.texture {
                            Some(t) => t.set(image, TextureOptions::LINEAR),
                            None => {
                                l.texture =
                                    Some(ctx.load_texture("preview", image, TextureOptions::LINEAR))
                            }
                        }
                        l.texture_gen = generation;
                    }
                }
            }
        }
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
        ui.painter().rect_filled(rect, 6.0, Color32::BLACK);
        if let Some(t) = &l.texture {
            let size = t.size_vec2();
            let scale = (rect.width() / size.x).min(rect.height() / size.y);
            let draw = Rect::from_center_size(rect.center(), size * scale);
            ui.painter().image(
                t.id(),
                draw,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else if l.info.video.is_none() {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Audio only",
                egui::FontId::proportional(16.0),
                Color32::from_gray(150),
            );
        }
        let rate = l.audio.sample_rate as f64;
        if let Some(r) = l.regions.iter().find(|r| pos >= r.start && pos < r.end) {
            let text = match r.depth {
                Depth::Keep => "Music kept",
                _ => "Music removed",
            };
            let tag = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(120.0, 22.0));
            ui.painter().rect_filled(tag, 4.0, MUSIC);
            ui.painter().text(
                tag.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(13.0),
                Color32::WHITE,
            );
        }
        ui.painter().text(
            rect.left_bottom() + Vec2::new(10.0, -10.0),
            egui::Align2::LEFT_BOTTOM,
            fmt_time(pos as f64 / rate),
            egui::FontId::monospace(13.0),
            Color32::WHITE,
        );
    }

    fn timeline(&mut self, ui: &mut egui::Ui) {
        let Some(l) = &mut self.loaded else { return };
        let player = self.player.as_ref();
        let pos = player.map(|p| p.position()).unwrap_or(0);
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), 130.0),
            Sense::click_and_drag(),
        );
        let rect = response.rect;
        painter.rect_filled(rect, 6.0, ui.visuals().extreme_bg_color);
        let samples = l.audio.samples().max(1);
        let rate = l.audio.sample_rate as f64;
        let x_of = |t: usize| rect.left() + rect.width() * (t as f32 / samples as f32);
        for r in &l.regions {
            let area = Rect::from_min_max(
                egui::pos2(x_of(r.start), rect.top()),
                egui::pos2(x_of(r.end), rect.bottom()),
            );
            let alpha = if r.depth == Depth::Keep { 20 } else { 45 };
            painter.rect_filled(
                area,
                0.0,
                Color32::from_rgba_unmultiplied(217, 130, 43, alpha),
            );
        }
        if let Some(i) = l.selected {
            if let Some(r) = l.regions.get(i) {
                painter.rect_stroke(
                    Rect::from_min_max(
                        egui::pos2(x_of(r.start), rect.top()),
                        egui::pos2(x_of(r.end), rect.bottom()),
                    ),
                    0.0,
                    Stroke::new(1.5, MUSIC),
                    egui::StrokeKind::Inside,
                );
            }
        }
        let mid = rect.center().y;
        let half = rect.height() * 0.45;
        let width = rect.width().max(1.0) as usize;
        let per_px = l.peaks.len() as f32 / width as f32;
        let amount_at = |peak_index: usize| {
            let t = peak_index * (rate as usize / PEAKS_PER_SECOND);
            l.regions
                .iter()
                .find(|r| t >= r.start && t < r.end)
                .map(|r| r.depth.amount())
                .unwrap_or(0.0)
        };
        for px in 0..width {
            let a = (px as f32 * per_px) as usize;
            let b = (((px + 1) as f32 * per_px) as usize)
                .max(a + 1)
                .min(l.peaks.len());
            if a >= b {
                continue;
            }
            let peak = l.peaks[a..b]
                .iter()
                .copied()
                .fold(0.0f32, f32::max)
                .min(1.0);
            let x = rect.left() + px as f32 + 0.5;
            painter.line_segment(
                [
                    egui::pos2(x, mid - peak * half),
                    egui::pos2(x, mid + peak * half),
                ],
                Stroke::new(1.0, WAVE),
            );
            if !l.removed_peaks.is_empty() {
                let removed = l.removed_peaks[a..b.min(l.removed_peaks.len())]
                    .iter()
                    .copied()
                    .fold(0.0f32, f32::max)
                    .min(1.0)
                    * amount_at(a);
                if removed > 0.0 {
                    painter.line_segment(
                        [
                            egui::pos2(x, mid - removed * half),
                            egui::pos2(x, mid + removed * half),
                        ],
                        Stroke::new(1.0, MUSIC),
                    );
                }
            }
        }
        let x = x_of(pos);
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            Stroke::new(2.0, Color32::from_rgb(220, 80, 60)),
        );
        if let Some(p) = response.interact_pointer_pos() {
            if response.clicked() || response.dragged() {
                let frac = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                let t = (frac as f64 * samples as f64) as usize;
                if let Some(player) = player {
                    player.seek(t);
                }
                if response.clicked() {
                    l.selected = l.regions.iter().position(|r| t >= r.start && t < r.end);
                }
            }
        }
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("0:00").small().weak());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(fmt_time(samples as f64 / rate))
                        .small()
                        .weak(),
                );
            });
        });
    }

    fn transport(&mut self, ui: &mut egui::Ui) {
        let Some(player) = &self.player else { return };
        let Some(l) = &self.loaded else { return };
        ui.horizontal(|ui| {
            let label = if player.playing() { "Pause" } else { "Play" };
            if ui.button(label).clicked() {
                log::info!("play button clicked");
                if player.position() >= l.audio.samples() {
                    player.seek(0);
                }
                player.set_playing(!player.playing());
            }
            ui.label(format!(
                "{} / {}",
                fmt_time(player.position() as f64 / l.audio.sample_rate as f64),
                fmt_time(l.audio.duration())
            ));
            ui.label(
                egui::RichText::new("Space: play/pause. 1/2/3: original/result/removed.")
                    .small()
                    .weak(),
            );
        });
    }

    fn keys(&mut self, ctx: &egui::Context) {
        let Some(player) = &self.player else { return };
        if self.loaded.is_none() || ctx.egui_wants_keyboard_input() {
            return;
        }
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                log::info!("space pressed");
                player.set_playing(!player.playing());
            }
            if i.key_pressed(egui::Key::Num1) {
                player.set_listen(Listen::Original);
            }
            if i.key_pressed(egui::Key::Num2) {
                player.set_listen(Listen::Result);
            }
            if i.key_pressed(egui::Key::Num3) {
                player.set_listen(Listen::Removed);
            }
        });
    }

    fn about(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .open(&mut self.show_about)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Charon Music Removal (demo)").strong());
                ui.label("Removes music from a video and keeps speech and sound effects. Everything runs on this computer; the app makes no network connections. The video stream is copied without re-encoding.");
                ui.separator();
                ui.label("Engine: charon-audio (MIT OR Apache-2.0). Runtime: ONNX Runtime (MIT). Media: Apple AVFoundation.");
                ui.label("Model: TIGER-DnR, music branch (Xu, Li, Chen, Hu; JusperLee/TIGER-DnR, weights Apache-2.0), converted to ONNX; changes listed in licenses/TIGER-DnR-NOTICE.txt. Trained on Divide and Remaster v1, whose music and effects include non-commercial clips: this demo is free, with no paid features, for that reason.");
                ui.label(format!("Model file: {}", self.model_path.display()));
                ui.label("UI: egui (MIT OR Apache-2.0).");
            });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll(&ctx);
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .filter(|p| !p.as_os_str().is_empty())
                .collect()
        });
        if let Some(path) = dropped.into_iter().next() {
            if matches!(self.busy, Busy::Idle) {
                self.open(path);
            }
        }
        self.keys(&ctx);

        egui::Panel::top("top").show(ui, |ui| {
            ui.add_space(4.0);
            self.top_bar(ui);
            ui.add_space(4.0);
        });
        egui::Panel::bottom("status").show(ui, |ui| {
            ui.add_space(3.0);
            self.status_bar(ui);
            ui.add_space(3.0);
        });
        egui::Panel::right("inspector")
            .default_size(300.0)
            .resizable(true)
            .show(ui, |ui| {
                ui.add_space(6.0);
                self.inspector(ui);
            });
        egui::CentralPanel::default_margins().show(ui, |ui| {
            let height = (ui.available_height() - 200.0).max(160.0);
            self.preview(ui, &ctx, height);
            ui.add_space(8.0);
            self.transport(ui);
            ui.add_space(4.0);
            self.timeline(ui);
        });
        self.about(&ctx);
        if self.player.as_ref().is_some_and(|p| p.playing()) {
            ctx.request_repaint();
        }
    }
}
