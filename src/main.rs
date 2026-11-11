use std::time::{Duration, Instant};

use chrono::{Local, Utc};
use chrono_tz::Tz;
use eframe::egui::{self, Color32, Frame, Margin, RichText, Rounding, Stroke, Visuals};

// ------------------------------------------------------------------
// Palette (Material 3 – AMOLED seed #6750A4)
// ------------------------------------------------------------------
const PRIMARY:           Color32 = Color32::from_rgb(0xD0, 0xBC, 0xFF);
const ON_PRIMARY:        Color32 = Color32::from_rgb(0x38, 0x1E, 0x72);
const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0x4F, 0x37, 0x8B);
const SURFACE:           Color32 = Color32::from_rgb(0x1C, 0x1B, 0x1F);
const SURFACE_HOVER:     Color32 = Color32::from_rgb(0x2B, 0x29, 0x30);
const ON_SURFACE:        Color32 = Color32::from_rgb(0xE6, 0xE1, 0xE5);
const ON_SURFACE_VAR:    Color32 = Color32::from_rgb(0xCA, 0xC4, 0xD0);
const BLACK:             Color32 = Color32::BLACK;

// Base reference size — UI scales proportionally with the window
const BASE_W: f32 = 480.0;
const BASE_H: f32 = 720.0;

// ------------------------------------------------------------------
// Data
// ------------------------------------------------------------------
struct Lap {
    name: String,
    total: Duration,   // time since timer start
    split: Duration,   // time since previous lap
}

struct TimerState {
    started_at: Option<Instant>,
    elapsed_before_pause: Duration,
    laps: Vec<Lap>,
    new_lap_name: String,
    running: bool,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            started_at: None,
            elapsed_before_pause: Duration::ZERO,
            laps: Vec::new(),
            new_lap_name: String::new(),
            running: false,
        }
    }
}

impl TimerState {
    fn elapsed(&self) -> Duration {
        if let Some(start) = self.started_at {
            start.elapsed() + self.elapsed_before_pause
        } else {
            self.elapsed_before_pause
        }
    }
    fn start(&mut self) {
        if !self.running {
            self.started_at = Some(Instant::now());
            self.running = true;
        }
    }
    fn pause(&mut self) {
        if self.running {
            self.elapsed_before_pause += self.started_at.take().unwrap().elapsed();
            self.running = false;
        }
    }
    fn reset(&mut self) {
        self.started_at = None;
        self.elapsed_before_pause = Duration::ZERO;
        self.laps.clear();
        self.running = false;
        self.new_lap_name.clear();
    }
    fn add_lap(&mut self) {
        if self.new_lap_name.trim().is_empty() {
            return;
        }
        let name = self.new_lap_name.trim().to_string();
        let total = self.elapsed();
        let split = if let Some(last) = self.laps.last() {
            total.saturating_sub(last.total)
        } else {
            total
        };
        self.laps.push(Lap { name, total, split });
        self.new_lap_name.clear();
    }
}

struct ClockState {
    start_time: Option<chrono::DateTime<Utc>>,
    time_zones: Vec<Tz>,
    new_zone: Tz,
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            start_time: None,
            time_zones: vec![
                chrono_tz::UTC,
                chrono_tz::Asia::Kolkata,
                chrono_tz::America::New_York,
            ],
            new_zone: chrono_tz::Europe::London,
        }
    }
}

// ------------------------------------------------------------------
// App
// ------------------------------------------------------------------
struct CuteTimerApp {
    timer: TimerState,
    clock: ClockState,
    active_tab: usize,
    ui_scale: f32,
}

impl Default for CuteTimerApp {
    fn default() -> Self {
        Self {
            timer: TimerState::default(),
            clock: ClockState::default(),
            active_tab: 0,
            ui_scale: 1.0,
        }
    }
}

// ------------------------------------------------------------------
// Scaled Material-style helpers
// ------------------------------------------------------------------
fn filled_button(ui: &mut egui::Ui, label: &str, width: f32, scale: f32) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(16.0 * scale).color(ON_PRIMARY))
            .fill(PRIMARY)
            .rounding(Rounding::same(26.0 * scale))
            .min_size(egui::vec2(width * scale, 52.0 * scale)),
    )
}

fn outlined_button(ui: &mut egui::Ui, label: &str, width: f32, scale: f32) -> egui::Response {
    let btn = egui::Button::new(RichText::new(label).size(16.0 * scale).color(PRIMARY))
        .fill(Color32::TRANSPARENT)
        .rounding(Rounding::same(26.0 * scale))
        .min_size(egui::vec2(width * scale, 52.0 * scale));
    let resp = ui.add(btn);
    ui.painter().rect_stroke(
        resp.rect,
        Rounding::same(26.0 * scale),
        Stroke::new((1.0 * scale).max(1.0), PRIMARY),
    );
    resp
}

fn material_card(ui: &mut egui::Ui, scale: f32, add_contents: impl FnOnce(&mut egui::Ui)) {
    Frame::none()
        .fill(SURFACE)
        .rounding(Rounding::same(14.0 * scale))
        .inner_margin(Margin::same(14.0 * scale))
        .show(ui, add_contents);
}

fn compute_scale(screen: egui::Rect) -> f32 {
    (screen.width() / BASE_W)
        .min(screen.height() / BASE_H)
        .clamp(0.8, 3.5)
}

// ------------------------------------------------------------------
// App impl
// ------------------------------------------------------------------
impl eframe::App for CuteTimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.timer.running {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else {
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        // Scale everything proportionally to the window size
        self.ui_scale = compute_scale(ctx.screen_rect());
        let scale = self.ui_scale;

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(BLACK)
                    .inner_margin(Margin::same(12.0 * scale)),
            )
            .show(ctx, |ui| {
                // --- Pinned tab bar ---
                ui.horizontal(|ui| {
                    let tab = |ui: &mut egui::Ui, label: &str, idx: usize, active: usize| {
                        let bg = if idx == active { PRIMARY_CONTAINER } else { SURFACE };
                        let fg = if idx == active { ON_SURFACE } else { ON_SURFACE_VAR };
                        ui.add(
                            egui::Button::new(RichText::new(label).size(18.0 * scale).color(fg))
                                .fill(bg)
                                .rounding(Rounding::same(26.0 * scale))
                                .min_size(egui::vec2(160.0 * scale, 52.0 * scale)),
                        )
                        .clicked()
                    };
                    if tab(ui, "⏱ Timer", 0, self.active_tab) {
                        self.active_tab = 0;
                    }
                    if tab(ui, "🌍 Clock", 1, self.active_tab) {
                        self.active_tab = 1;
                    }
                });

                ui.add_space(14.0 * scale);

                // --- Scrollable content ---
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match self.active_tab {
                            0 => self.timer_tab(ui),
                            1 => self.clock_tab(ui),
                            _ => unreachable!(),
                        }
                    });
            });
    }
}

impl CuteTimerApp {
    fn timer_tab(&mut self, ui: &mut egui::Ui) {
        let scale = self.ui_scale;
        let display = format_duration(self.timer.elapsed());

        ui.vertical_centered(|ui| {
            ui.add_space(8.0 * scale);
            ui.label(
                RichText::new(display)
                    .size(96.0 * scale)
                    .strong()
                    .color(PRIMARY),
            );
        });

        ui.add_space(12.0 * scale);

        ui.horizontal(|ui| {
            if self.timer.running {
                if filled_button(ui, "⏸ Pause", 140.0, scale).clicked() {
                    self.timer.pause();
                }
            } else if filled_button(ui, "▶ Start", 140.0, scale).clicked() {
                self.timer.start();
            }
            if outlined_button(ui, "⏹ Reset", 140.0, scale).clicked() {
                self.timer.reset();
            }
        });

        ui.add_space(16.0 * scale);

        // --- New lap row (Enter adds the lap) ---
        ui.horizontal(|ui| {
            ui.label(RichText::new("New lap:").size(16.0 * scale).color(ON_SURFACE_VAR));
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.timer.new_lap_name)
                    .desired_width(260.0 * scale)
                    .font(egui::FontId::proportional(18.0 * scale))
                    .text_color(ON_SURFACE),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.timer.add_lap();
                resp.request_focus();
            }
            if filled_button(ui, "➕ Add", 110.0, scale).clicked() {
                self.timer.add_lap();
            }
        });

        ui.add_space(16.0 * scale);

        // --- Lap cards: name + Total (since start) + Split (since previous lap) ---
        for (i, lap) in self.timer.laps.iter_mut().enumerate() {
            material_card(ui, scale, |ui| {
                // Name on its own full-width line
                ui.add(
                    egui::TextEdit::singleline(&mut lap.name)
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::proportional(18.0 * scale))
                        .text_color(ON_SURFACE),
                );
                ui.add_space(8.0 * scale);

                // Two time readouts side-by-side
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Total").size(13.0 * scale).color(ON_SURFACE_VAR));
                    ui.label(
                        RichText::new(format_duration(lap.total))
                            .size(24.0 * scale)
                            .strong()
                            .color(PRIMARY),
                    );

                    ui.add_space(24.0 * scale);

                    ui.label(RichText::new("Split").size(13.0 * scale).color(ON_SURFACE_VAR));
                    let split_str = if i == 0 {
                        format_duration(lap.split)
                    } else {
                        format!("+{}", format_duration(lap.split))
                    };
                    ui.label(RichText::new(split_str).size(18.0 * scale).color(ON_SURFACE));
                });
            });
            ui.add_space(8.0 * scale);
        }
    }

    fn clock_tab(&mut self, ui: &mut egui::Ui) {
        let scale = self.ui_scale;

        if self.timer.running && self.clock.start_time.is_none() {
            self.clock.start_time = Some(Utc::now());
        }

        let now = Local::now();
        ui.label(
            RichText::new(format!("Now: {}", now.format("%H:%M:%S")))
                .size(36.0 * scale)
                .strong()
                .color(PRIMARY),
        );

        if let Some(start) = self.clock.start_time {
            let start_local = start.with_timezone(&Local);
            ui.label(
                RichText::new(format!("Started at: {}", start_local.format("%H:%M:%S")))
                    .size(16.0 * scale)
                    .color(ON_SURFACE_VAR),
            );
            ui.label(
                RichText::new(format!("Elapsed: {}", format_duration(self.timer.elapsed())))
                    .size(16.0 * scale)
                    .color(ON_SURFACE_VAR),
            );
        } else {
            ui.label(
                RichText::new("Timer not started yet.")
                    .size(16.0 * scale)
                    .color(ON_SURFACE_VAR),
            );
        }

        ui.add_space(16.0 * scale);
        ui.separator();
        ui.add_space(16.0 * scale);

        ui.label(RichText::new("Time zones").size(22.0 * scale).color(ON_SURFACE));

        ui.horizontal(|ui| {
            egui::ComboBox::from_label("")
                .selected_text(self.clock.new_zone.name())
                .show_ui(ui, |ui| {
                    for tz in chrono_tz::TZ_VARIANTS.iter() {
                        ui.selectable_value(&mut self.clock.new_zone, *tz, tz.name());
                    }
                });
            if filled_button(ui, "➕ Add", 90.0, scale).clicked() {
                if !self.clock.time_zones.contains(&self.clock.new_zone) {
                    self.clock.time_zones.push(self.clock.new_zone);
                }
            }
        });

        ui.add_space(16.0 * scale);

        for tz in &self.clock.time_zones {
            let local_in_zone = Utc::now().with_timezone(tz);
            material_card(ui, scale, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{}:", tz.name()))
                            .size(16.0 * scale)
                            .color(ON_SURFACE_VAR),
                    );
                    ui.add_space(8.0 * scale);
                    ui.label(
                        RichText::new(local_in_zone.format("%H:%M:%S").to_string())
                            .size(22.0 * scale)
                            .strong()
                            .color(PRIMARY),
                    );
                });
            });
            ui.add_space(8.0 * scale);
        }
    }
}

// ------------------------------------------------------------------
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let ms = d.subsec_millis();
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}.{ms:03}")
    } else {
        format!("{m:02}:{s:02}.{ms:03}")
    }
}

fn amoled_visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.override_text_color = Some(ON_SURFACE);
    v.panel_fill = BLACK;
    v.window_fill = BLACK;
    v.extreme_bg_color = SURFACE;
    v.widgets.noninteractive.bg_fill = BLACK;
    v.widgets.inactive.bg_fill = SURFACE;
    v.widgets.inactive.weak_bg_fill = SURFACE;
    v.widgets.inactive.rounding = Rounding::same(20.0);
    v.widgets.hovered.bg_fill = SURFACE_HOVER;
    v.widgets.hovered.weak_bg_fill = SURFACE_HOVER;
    v.widgets.hovered.rounding = Rounding::same(20.0);
    v.widgets.active.bg_fill = PRIMARY_CONTAINER;
    v.widgets.active.weak_bg_fill = PRIMARY_CONTAINER;
    v.widgets.active.rounding = Rounding::same(20.0);
    v.selection.bg_fill = PRIMARY_CONTAINER;
    v.selection.stroke = Stroke::new(1.0_f32, PRIMARY);
    v
}

// ------------------------------------------------------------------
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 720.0])
            .with_title("Cute AMOLED Timer"),
        ..Default::default()
    };

    eframe::run_native(
        "Cute AMOLED Timer",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(amoled_visuals());
            Box::<CuteTimerApp>::default() as Box<dyn eframe::App>
        }),
    )
}