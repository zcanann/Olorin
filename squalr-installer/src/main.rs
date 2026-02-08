#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::NativeOptions;
use eframe::egui;
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{
    Align, Color32, ColorImage, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame, IconData, Layout, Margin, RichText, ScrollArea, Sense,
    Stroke, TextureHandle, ViewportBuilder,
};
use log::{Level, Log, Metadata, Record, SetLoggerError};
use squalr_engine::app_provisioner::app_provisioner_config::AppProvisionerConfig;
use squalr_engine::app_provisioner::installer::app_installer::AppInstaller;
use squalr_engine::app_provisioner::installer::install_phase::InstallPhase;
use squalr_engine::app_provisioner::installer::install_progress::InstallProgress;
use squalr_engine::app_provisioner::operations::launch::update_operation_launch::UpdateOperationLaunch;
use squalr_engine::app_provisioner::progress_tracker::ProgressTracker;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const APP_NAME: &str = "Squalr Installer";
const MAX_LOG_BUFFER_BYTES: usize = 256 * 1024;
static ICON_APP: &[u8] = include_bytes!("../../squalr/images/app/app_icon.png");
static ICON_CLOSE: &[u8] = include_bytes!("../../squalr/images/app/close.png");
static ICON_MAXIMIZE: &[u8] = include_bytes!("../../squalr/images/app/maximize.png");
static ICON_MINIMIZE: &[u8] = include_bytes!("../../squalr/images/app/minimize.png");
static FONT_NOTO_SANS: &[u8] = include_bytes!("../../squalr/fonts/NotoSans.ttf");
static FONT_UBUNTU_MONO_BOLD: &[u8] = include_bytes!("../../squalr/fonts/UbuntuMonoBold.ttf");

#[derive(Clone)]
struct InstallerFontLibrary {
    font_normal: FontId,
    font_window_title: FontId,
    font_ubuntu_mono_normal: FontId,
}

impl InstallerFontLibrary {
    fn new() -> Self {
        Self {
            font_normal: FontId::new(13.0, FontFamily::Name("noto_sans".into())),
            font_window_title: FontId::new(14.0, FontFamily::Name("noto_sans".into())),
            font_ubuntu_mono_normal: FontId::new(15.0, FontFamily::Name("ubuntu_mono_bold".into())),
        }
    }
}

#[derive(Clone)]
struct InstallerIconLibrary {
    app: TextureHandle,
    close: TextureHandle,
    maximize: TextureHandle,
    minimize: TextureHandle,
}

#[derive(Clone)]
struct InstallerUiState {
    installer_phase: InstallPhase,
    installer_progress: f32,
    installer_progress_string: String,
    install_complete: bool,
    installer_logs: String,
}

impl InstallerUiState {
    fn new() -> Self {
        Self {
            installer_phase: InstallPhase::Download,
            installer_progress: 0.0,
            installer_progress_string: "0%".to_string(),
            install_complete: false,
            installer_logs: String::new(),
        }
    }

    fn set_progress(
        &mut self,
        install_progress: InstallProgress,
    ) {
        self.installer_phase = install_progress.phase;
        self.installer_progress = install_progress.progress_percent.clamp(0.0, 1.0);
        self.installer_progress_string = format!("{:.0}%", self.installer_progress * 100.0);
        self.install_complete = install_progress.phase == InstallPhase::Complete;
    }

    fn append_log(
        &mut self,
        log_message: &str,
    ) {
        self.installer_logs.push_str(log_message);
        trim_log_buffer(&mut self.installer_logs, MAX_LOG_BUFFER_BYTES);
    }
}

struct InstallerLogger {
    ui_state: Arc<Mutex<InstallerUiState>>,
}

impl InstallerLogger {
    fn new(ui_state: Arc<Mutex<InstallerUiState>>) -> Self {
        Self { ui_state }
    }
}

impl Log for InstallerLogger {
    fn enabled(
        &self,
        metadata: &Metadata,
    ) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(
        &self,
        record: &Record,
    ) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let log_message = format!("[{}] {}\n", record.level(), record.args());

        if let Ok(mut ui_state) = self.ui_state.lock() {
            ui_state.append_log(&log_message);
        }
    }

    fn flush(&self) {}
}

struct InstallerApp {
    ui_state: Arc<Mutex<InstallerUiState>>,
    installer_theme: InstallerTheme,
    installer_icon_library: Option<InstallerIconLibrary>,
}

impl InstallerApp {
    fn new(
        context: &egui::Context,
        ui_state: Arc<Mutex<InstallerUiState>>,
    ) -> Self {
        let installer_theme = InstallerTheme::default();
        installer_theme.apply(context);
        install_fonts(context);
        let installer_icon_library = load_installer_icon_library(context);
        start_installer(ui_state.clone());
        Self {
            ui_state,
            installer_theme,
            installer_icon_library,
        }
    }

    fn launch_app(&self) {
        match AppProvisionerConfig::get_default_install_dir() {
            Ok(app_install_directory) => {
                let executable_path = app_install_directory.join("squalr.exe");
                UpdateOperationLaunch::launch_app(&executable_path);
            }
            Err(error) => {
                log::error!("Failed to resolve install directory: {}", error);
            }
        }
    }

    fn render_installer_title_bar(
        &self,
        context: &egui::Context,
    ) {
        egui::TopBottomPanel::top("title_bar")
            .exact_height(self.installer_theme.title_bar_height)
            .resizable(false)
            .frame(Frame::new().fill(Color32::TRANSPARENT))
            .show(context, |ui| {
                let title_bar_rectangle = ui.max_rect();
                ui.painter().rect_filled(
                    title_bar_rectangle,
                    CornerRadius {
                        nw: self.installer_theme.corner_radius_panel,
                        ne: self.installer_theme.corner_radius_panel,
                        sw: 0,
                        se: 0,
                    },
                    self.installer_theme.color_background_primary,
                );

                let mut button_cluster_rectangle: Option<egui::Rect> = None;
                ui.allocate_ui_with_layout(title_bar_rectangle.size(), Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(8.0);

                    if let Some(installer_icon_library) = &self.installer_icon_library {
                        ui.add(egui::Image::new((installer_icon_library.app.id(), egui::vec2(18.0, 18.0))));
                        ui.add_space(4.0);
                    }

                    ui.label(
                        RichText::new(APP_NAME)
                            .font(self.installer_theme.fonts.font_window_title.clone())
                            .color(self.installer_theme.color_foreground),
                    );

                    ui.add_space(ui.available_width());

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let button_size = egui::vec2(36.0, self.installer_theme.title_bar_height);

                        let close_button_response = ui.add_sized(button_size, egui::Button::new("").frame(false));
                        if let Some(installer_icon_library) = &self.installer_icon_library {
                            draw_icon(ui, close_button_response.rect, &installer_icon_library.close);
                        }
                        if close_button_response.clicked() {
                            context.send_viewport_cmd(ViewportCommand::Close);
                        }

                        let maximize_button_response = ui.add_sized(button_size, egui::Button::new("").frame(false));
                        if let Some(installer_icon_library) = &self.installer_icon_library {
                            draw_icon(ui, maximize_button_response.rect, &installer_icon_library.maximize);
                        }
                        if maximize_button_response.clicked() {
                            let currently_maximized = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
                            context.send_viewport_cmd(ViewportCommand::Maximized(!currently_maximized));
                        }

                        let minimize_button_response = ui.add_sized(button_size, egui::Button::new("").frame(false));
                        if let Some(installer_icon_library) = &self.installer_icon_library {
                            draw_icon(ui, minimize_button_response.rect, &installer_icon_library.minimize);
                        }
                        if minimize_button_response.clicked() {
                            context.send_viewport_cmd(ViewportCommand::Minimized(true));
                        }

                        button_cluster_rectangle = Some(
                            close_button_response
                                .rect
                                .union(maximize_button_response.rect)
                                .union(minimize_button_response.rect),
                        );
                    });
                });

                let drag_region_right_edge = button_cluster_rectangle
                    .map(|rectangle| rectangle.min.x)
                    .unwrap_or(title_bar_rectangle.max.x);
                let drag_region = egui::Rect::from_min_max(title_bar_rectangle.min, egui::pos2(drag_region_right_edge, title_bar_rectangle.max.y));
                let drag_response = ui.interact(drag_region, ui.id().with("installer_title_bar_drag"), Sense::click_and_drag());

                if drag_response.drag_started() {
                    context.send_viewport_cmd(ViewportCommand::StartDrag);
                }

                if drag_response.double_clicked() {
                    let currently_maximized = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
                    context.send_viewport_cmd(ViewportCommand::Maximized(!currently_maximized));
                }
            });
    }

    fn render_installer_footer(
        &self,
        context: &egui::Context,
        install_complete: bool,
    ) {
        egui::TopBottomPanel::bottom("footer")
            .exact_height(self.installer_theme.footer_height)
            .resizable(false)
            .frame(Frame::new().fill(Color32::TRANSPARENT))
            .show(context, |ui| {
                let footer_rectangle = ui.max_rect();
                ui.painter().rect_filled(
                    footer_rectangle,
                    CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: self.installer_theme.corner_radius_panel,
                        se: self.installer_theme.corner_radius_panel,
                    },
                    self.installer_theme.color_border_blue,
                );

                ui.allocate_ui_with_layout(footer_rectangle.size(), Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Install location: default user application directory.")
                            .font(self.installer_theme.fonts.font_normal.clone())
                            .color(self.installer_theme.color_foreground),
                    );
                    ui.add_space(ui.available_width());

                    if install_complete && ui.button("Launch Squalr").clicked() {
                        self.launch_app();
                    }
                });
            });
    }
}

#[derive(Clone)]
struct InstallerTheme {
    fonts: InstallerFontLibrary,
    color_background_primary: Color32,
    color_background_panel: Color32,
    color_background_control: Color32,
    color_background_control_primary: Color32,
    color_background_control_primary_dark: Color32,
    color_background_control_success: Color32,
    color_background_control_success_dark: Color32,
    color_foreground: Color32,
    color_foreground_preview: Color32,
    color_foreground_warning: Color32,
    color_foreground_error: Color32,
    color_foreground_info: Color32,
    color_foreground_debug: Color32,
    color_foreground_trace: Color32,
    color_border_blue: Color32,
    color_border_panel: Color32,
    color_log_background: Color32,
    corner_radius_panel: u8,
    title_bar_height: f32,
    footer_height: f32,
}

impl Default for InstallerTheme {
    fn default() -> Self {
        Self {
            fonts: InstallerFontLibrary::new(),
            color_background_primary: Color32::from_rgb(0x33, 0x33, 0x33),
            color_background_panel: Color32::from_rgb(0x27, 0x27, 0x27),
            color_background_control: Color32::from_rgb(0x44, 0x44, 0x44),
            color_background_control_primary: Color32::from_rgb(0x1E, 0x54, 0x92),
            color_background_control_primary_dark: Color32::from_rgb(0x06, 0x1E, 0x3E),
            color_background_control_success: Color32::from_rgb(0x14, 0xA4, 0x4D),
            color_background_control_success_dark: Color32::from_rgb(0x0E, 0x72, 0x36),
            color_foreground: Color32::WHITE,
            color_foreground_preview: Color32::from_rgb(0xAF, 0xAF, 0xAF),
            color_foreground_warning: Color32::from_rgb(0xE4, 0xA1, 0x1B),
            color_foreground_error: Color32::from_rgb(0xDC, 0x4C, 0x64),
            color_foreground_info: Color32::from_rgb(0x32, 0xC4, 0xE6),
            color_foreground_debug: Color32::from_rgb(0x32, 0xC4, 0xE6),
            color_foreground_trace: Color32::from_rgb(0x14, 0xA4, 0x4D),
            color_border_blue: Color32::from_rgb(0x00, 0x7A, 0xCC),
            color_border_panel: Color32::from_rgb(0x20, 0x1C, 0x1C),
            color_log_background: Color32::from_rgb(0x1E, 0x1E, 0x1E),
            corner_radius_panel: 8,
            title_bar_height: 32.0,
            footer_height: 24.0,
        }
    }
}

impl InstallerTheme {
    fn apply(
        &self,
        context: &egui::Context,
    ) {
        let mut style = (*context.style()).clone();
        let visuals = &mut style.visuals;

        visuals.override_text_color = Some(self.color_foreground);
        visuals.panel_fill = self.color_background_primary;
        visuals.window_fill = self.color_background_panel;
        visuals.faint_bg_color = self.color_background_panel;
        visuals.extreme_bg_color = self.color_background_control;
        visuals.code_bg_color = self.color_log_background;
        visuals.window_stroke = Stroke::new(1.0, self.color_border_panel);
        visuals.selection.bg_fill = self.color_border_blue;
        visuals.selection.stroke = Stroke::new(1.0, self.color_border_blue);

        visuals.widgets.noninteractive.bg_fill = self.color_background_panel;
        visuals.widgets.noninteractive.weak_bg_fill = self.color_background_panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.color_border_panel);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.color_foreground_preview);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.inactive.bg_fill = self.color_background_control_primary;
        visuals.widgets.inactive.weak_bg_fill = self.color_background_control_primary_dark;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.color_border_blue);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.hovered.bg_fill = self.color_border_blue;
        visuals.widgets.hovered.weak_bg_fill = self.color_background_control_primary;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.color_border_blue);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.active.bg_fill = self.color_background_control_primary_dark;
        visuals.widgets.active.weak_bg_fill = self.color_background_control_primary_dark;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.color_border_blue);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.active.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.open.bg_fill = self.color_background_control;
        visuals.widgets.open.weak_bg_fill = self.color_background_control;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, self.color_border_panel);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.open.corner_radius = CornerRadius::same(self.corner_radius_panel);

        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.window_margin = Margin::same(8);
        context.set_style(style);
    }
}

fn install_fonts(context: &egui::Context) {
    let mut font_definitions = FontDefinitions::default();
    font_definitions
        .font_data
        .insert("noto_sans".to_owned(), FontData::from_static(FONT_NOTO_SANS).into());
    font_definitions
        .font_data
        .insert("ubuntu_mono_bold".to_owned(), FontData::from_static(FONT_UBUNTU_MONO_BOLD).into());

    font_definitions
        .families
        .insert(FontFamily::Name("noto_sans".into()), vec!["noto_sans".to_owned()]);
    font_definitions
        .families
        .insert(FontFamily::Name("ubuntu_mono_bold".into()), vec!["ubuntu_mono_bold".to_owned()]);

    context.set_fonts(font_definitions);
}

fn load_installer_icon(
    context: &egui::Context,
    identifier: &str,
    icon_bytes: &[u8],
) -> Option<TextureHandle> {
    let icon = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let icon_size = [icon.width() as usize, icon.height() as usize];
    let icon_color_image = ColorImage::from_rgba_unmultiplied(icon_size, icon.as_raw());
    Some(context.load_texture(identifier, icon_color_image, egui::TextureOptions::LINEAR))
}

fn load_installer_icon_library(context: &egui::Context) -> Option<InstallerIconLibrary> {
    Some(InstallerIconLibrary {
        app: load_installer_icon(context, "installer_app_icon", ICON_APP)?,
        close: load_installer_icon(context, "installer_close_icon", ICON_CLOSE)?,
        maximize: load_installer_icon(context, "installer_maximize_icon", ICON_MAXIMIZE)?,
        minimize: load_installer_icon(context, "installer_minimize_icon", ICON_MINIMIZE)?,
    })
}

fn draw_icon(
    ui: &egui::Ui,
    bounds_rectangle: egui::Rect,
    icon_texture: &TextureHandle,
) {
    let [texture_width, texture_height] = icon_texture.size();
    let icon_rectangle = egui::Rect::from_center_size(bounds_rectangle.center(), egui::vec2(texture_width as f32, texture_height as f32));
    ui.painter().image(
        icon_texture.id(),
        icon_rectangle,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}

fn log_color_for_line(
    log_line: &str,
    installer_theme: &InstallerTheme,
) -> Color32 {
    if log_line.starts_with("[ERROR]") {
        installer_theme.color_foreground_error
    } else if log_line.starts_with("[WARN]") {
        installer_theme.color_foreground_warning
    } else if log_line.starts_with("[DEBUG]") {
        installer_theme.color_foreground_debug
    } else if log_line.starts_with("[TRACE]") {
        installer_theme.color_foreground_trace
    } else if log_line.starts_with("[INFO]") {
        installer_theme.color_foreground_info
    } else {
        installer_theme.color_foreground
    }
}

fn install_phase_string(install_phase: InstallPhase) -> &'static str {
    match install_phase {
        InstallPhase::Download => "Downloading installer package.",
        InstallPhase::Extraction => "Extracting installation payload.",
        InstallPhase::Complete => "Installation complete.",
    }
}

fn installer_status_string(ui_state: &InstallerUiState) -> &'static str {
    if ui_state.install_complete {
        "Squalr installed successfully."
    } else {
        "Installing Squalr, please wait..."
    }
}

impl eframe::App for InstallerApp {
    fn update(
        &mut self,
        context: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        context.request_repaint_after(Duration::from_millis(50));

        let state_snapshot = match self.ui_state.lock() {
            Ok(ui_state) => ui_state.clone(),
            Err(_) => InstallerUiState::new(),
        };

        let progress_status_text = install_phase_string(state_snapshot.installer_phase);
        let header_status_text = installer_status_string(&state_snapshot);

        self.render_installer_title_bar(context);
        self.render_installer_footer(context, state_snapshot.install_complete);

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(self.installer_theme.color_background_panel)
                    .inner_margin(Margin::same(8)),
            )
            .show(context, |ui| {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.label(
                        RichText::new(header_status_text)
                            .font(self.installer_theme.fonts.font_window_title.clone())
                            .color(self.installer_theme.color_foreground),
                    );

                    Frame::new()
                        .fill(self.installer_theme.color_background_primary)
                        .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Installation Status")
                                    .font(self.installer_theme.fonts.font_window_title.clone())
                                    .color(self.installer_theme.color_foreground),
                            );
                            ui.label(
                                RichText::new(progress_status_text)
                                    .font(self.installer_theme.fonts.font_normal.clone())
                                    .color(self.installer_theme.color_foreground_preview),
                            );
                            ui.add_space(4.0);

                            ui.add(
                                egui::ProgressBar::new(state_snapshot.installer_progress)
                                    .fill(if state_snapshot.install_complete {
                                        self.installer_theme.color_background_control_success
                                    } else {
                                        self.installer_theme.color_background_control_primary
                                    })
                                    .text(state_snapshot.installer_progress_string),
                            );
                        });

                    Frame::new()
                        .fill(self.installer_theme.color_background_primary)
                        .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Installer Log")
                                        .font(self.installer_theme.fonts.font_window_title.clone())
                                        .color(self.installer_theme.color_foreground),
                                );
                                if state_snapshot.install_complete {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new("Ready")
                                                .font(self.installer_theme.fonts.font_normal.clone())
                                                .color(self.installer_theme.color_background_control_success_dark),
                                        );
                                    });
                                }
                            });

                            Frame::new()
                                .fill(self.installer_theme.color_log_background)
                                .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                                .inner_margin(Margin::same(8))
                                .show(ui, |ui| {
                                    ui.set_min_height(290.0);
                                    ui.set_min_width(ui.available_width());
                                    ScrollArea::vertical()
                                        .id_salt("installer_log_scroll")
                                        .auto_shrink([false, false])
                                        .stick_to_bottom(true)
                                        .show(ui, |ui| {
                                            if state_snapshot.installer_logs.is_empty() {
                                                ui.label(
                                                    RichText::new("Waiting for installer output.")
                                                        .font(self.installer_theme.fonts.font_ubuntu_mono_normal.clone())
                                                        .color(self.installer_theme.color_foreground_preview),
                                                );
                                            } else {
                                                for installer_log_line in state_snapshot.installer_logs.lines() {
                                                    ui.label(
                                                        RichText::new(installer_log_line)
                                                            .font(self.installer_theme.fonts.font_normal.clone())
                                                            .color(log_color_for_line(installer_log_line, &self.installer_theme)),
                                                    );
                                                }
                                            }
                                        });
                                });
                        });
                });
            });
    }
}

fn trim_log_buffer(
    log_buffer: &mut String,
    max_buffer_bytes: usize,
) {
    if log_buffer.len() <= max_buffer_bytes {
        return;
    }

    let mut trim_start_index = log_buffer.len().saturating_sub(max_buffer_bytes);
    while trim_start_index < log_buffer.len() && !log_buffer.is_char_boundary(trim_start_index) {
        trim_start_index += 1;
    }

    log_buffer.drain(..trim_start_index);
}

fn initialize_logger(ui_state: Arc<Mutex<InstallerUiState>>) -> Result<(), SetLoggerError> {
    let logger = InstallerLogger::new(ui_state);
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}

fn start_installer(ui_state: Arc<Mutex<InstallerUiState>>) {
    let progress_tracker = ProgressTracker::new();
    let progress_receiver = progress_tracker.subscribe();
    let progress_ui_state = ui_state.clone();

    thread::spawn(move || {
        for install_progress in progress_receiver {
            if let Ok(mut state) = progress_ui_state.lock() {
                state.set_progress(install_progress);
            }
        }
    });

    match AppProvisionerConfig::get_default_install_dir() {
        Ok(install_directory) => {
            AppInstaller::run_installation(install_directory, progress_tracker);
        }
        Err(error) => {
            log::error!("Failed to resolve install directory: {}", error);
        }
    }
}

pub fn main() {
    let ui_state = Arc::new(Mutex::new(InstallerUiState::new()));

    if let Err(error) = initialize_logger(ui_state.clone()) {
        eprintln!("Failed to initialize installer logger: {}", error);
    }

    let icon = image::load_from_memory(ICON_APP)
        .unwrap_or_default()
        .into_rgba8();
    let icon_width = icon.width();
    let icon_height = icon.height();

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(IconData {
                rgba: icon.into_raw(),
                width: icon_width,
                height: icon_height,
            })
            .with_decorations(false)
            .with_inner_size([640.0, 640.0])
            .with_min_inner_size([480.0, 420.0]),
        ..NativeOptions::default()
    };

    let run_result = eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(move |creation_context| Ok(Box::new(InstallerApp::new(&creation_context.egui_ctx, ui_state.clone())))),
    );

    if let Err(error) = run_result {
        eprintln!("Fatal error starting installer GUI: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::trim_log_buffer;

    #[test]
    fn trim_log_buffer_keeps_recent_log_data() {
        let mut log_buffer = "line-1\nline-2\nline-3\n".to_string();
        trim_log_buffer(&mut log_buffer, 8);
        assert_eq!(log_buffer, "\nline-3\n");
    }

    #[test]
    fn trim_log_buffer_does_not_modify_when_under_limit() {
        let mut log_buffer = "line-1\n".to_string();
        trim_log_buffer(&mut log_buffer, 1024);
        assert_eq!(log_buffer, "line-1\n");
    }
}
