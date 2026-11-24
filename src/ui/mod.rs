//! The user interface (egui / eframe): main window, dialogs, preferences, combo picker and input dialog.

mod dialogs;
mod emoji_list;
mod main_window;
mod picker;
mod prefs_window;
pub mod style;
mod title_bar;
pub mod tray;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_channel::Receiver;
use egui::ViewportCommand;
use uuid::Uuid;

use crate::app_state::{AppShared, UiMsg};
use crate::engine::EngineMsg;
use crate::i18n::tr;
use crate::platform::PlatformStatus;
use crate::shortcut::Shortcut;

pub use dialogs::{ComboEditor, Dialog};
pub use main_window::{GroupFilter, SortColumn};
pub use picker::{InputPrompt, PickerState, SharedPopups};
pub use prefs_window::PrefsWindow;

/// What a shortcut being recorded will be assigned to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordTarget {
    Picker,
    Trigger,
    AppToggle,
    /// Insert a `#{shortcut:}` variable in the combo editor.
    SnippetVariable,
}

pub struct ExpanderApp {
    pub shared: Arc<AppShared>,
    ui_rx: Receiver<UiMsg>,
    tray: Option<tray::Tray>,
    pub main_visible: bool,
    quitting: bool,
    pass: u64,
    start_hidden: bool,
    positioned: bool,
    applied_theme: Option<crate::prefs::Theme>,
    /// Language generation of the texts computed once (platform status), see `i18n::generation`.
    applied_language: u64,

    // main window
    pub group_filter: GroupFilter,
    pub selection: Vec<Uuid>,
    pub anchor: Option<Uuid>,
    pub search: String,
    pub focus_search: bool,
    pub sort: (SortColumn, bool),

    // dialogs and secondary windows
    pub dialog: Option<Dialog>,
    pub prefs: Option<PrefsWindow>,
    pub messages: Vec<(String, String)>,
    pub recording: Option<RecordTarget>,
    pub recorded_snippet_shortcut: Option<Shortcut>,
    pub platform_status: PlatformStatus,

    // popups shown even when the main window is hidden
    pub popups: SharedPopups,
    /// Were popups shown during the last ui pass? A ui pass is needed to close them.
    popups_shown: bool,
}

impl ExpanderApp {
    fn new(cc: &eframe::CreationContext<'_>, shared: Arc<AppShared>, ui_rx: Receiver<UiMsg>, start_hidden: bool) -> Self {
        let ctx = cc.egui_ctx.clone();
        shared.ui.set_context(ctx.clone());
        style::setup(&ctx);

        let tray = tray::Tray::create(shared.clone());

        let mut messages: Vec<(String, String)> =
            shared.startup_errors.lock().drain(..).map(|e| (tr("app-error").to_string(), e)).collect();
        let platform_status = crate::platform::platform_status();
        if let Some(m) = &platform_status.message
            && !platform_status.input_ok
        {
            messages.push((tr("app-warning").to_string(), m.clone()));
        }

        let mut app = Self {
            shared: shared.clone(),
            ui_rx,
            tray,
            main_visible: !start_hidden,
            quitting: false,
            pass: 0,
            start_hidden,
            positioned: !start_hidden,
            applied_theme: None,
            applied_language: crate::i18n::generation(),
            group_filter: GroupFilter::All,
            selection: Vec::new(),
            anchor: None,
            search: String::new(),
            focus_search: false,
            sort: (SortColumn::Name, true),
            dialog: None,
            prefs: None,
            messages,
            recording: None,
            recorded_snippet_shortcut: None,
            platform_status,
            popups: SharedPopups::default(),
            popups_shown: false,
        };

        // first launch: offer to import an existing Beeftext library
        let first_launch = !shared.prefs.read().already_launched;
        if first_launch {
            if shared.library.read().combos.is_empty()
                && let Some(path) = crate::paths::beeftext_library_candidates().into_iter().next()
            {
                // the number of combos and groups, read once (the dialog is drawn many times)
                let summary = crate::library::Library::load(&path)
                    .map(|p| (p.library.combos.len(), p.library.groups.len()))
                    .map_err(|e| format!("{e:#}"));
                app.dialog = Some(Dialog::BeeftextMigration(path, summary));
            }
            shared.prefs.write().already_launched = true;
            shared.save_prefs();
        }
        app
    }

    // ---- window management ------------------------------------------------------------------------------------------

    pub fn show_main(&mut self, ctx: &egui::Context) {
        if !self.positioned {
            if let Some(pos) = centered_position(ctx, egui::vec2(980.0, 640.0)) {
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
            }
            self.positioned = true;
        }
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        self.main_visible = true;
        ctx.request_repaint();
    }

    pub fn hide_main(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        self.main_visible = false;
    }

    pub fn quit(&mut self, ctx: &egui::Context) {
        self.quitting = true;
        let _ = self.shared.engine_tx.send(EngineMsg::Quit);
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    pub fn error(&mut self, text: impl Into<String>) {
        self.messages.push((tr("app-error").to_string(), text.into()));
    }

    pub fn info(&mut self, text: impl Into<String>) {
        self.messages.push((tr("app-information").to_string(), text.into()));
    }

    /// Save the library, reporting errors to the user.
    pub fn commit_library(&mut self) {
        if let Err(e) = self.shared.save_library() {
            self.error(format!("{}\n{e:#}", tr("msg-library-save-failed")));
        }
    }

    pub fn commit_prefs(&mut self) {
        self.shared.refresh_hook_config();
        self.shared.save_prefs();
    }

    pub fn start_recording(&mut self, target: RecordTarget) {
        self.recording = Some(target);
        self.shared.recording_shortcut.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn stop_recording(&mut self) {
        self.recording = None;
        self.shared.recording_shortcut.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn assign_recorded(&mut self, sc: Shortcut) {
        let Some(target) = self.recording else { return };
        self.stop_recording();
        if !sc.is_valid() {
            return;
        }
        {
            let mut p = self.shared.prefs.write();
            match target {
                RecordTarget::Picker => p.combo_picker_shortcut = sc,
                RecordTarget::Trigger => p.combo_trigger_shortcut = sc,
                RecordTarget::AppToggle => p.app_toggle_shortcut = sc,
                RecordTarget::SnippetVariable => {}
            }
        }
        if target == RecordTarget::SnippetVariable {
            self.recorded_snippet_shortcut = Some(sc);
        } else {
            self.commit_prefs();
        }
    }

    pub fn last_dir(&self) -> Option<PathBuf> {
        let p = self.shared.prefs.read().last_import_export_path.clone();
        (!p.is_empty()).then(|| PathBuf::from(p)).filter(|p| p.is_dir())
    }

    pub fn remember_dir(&mut self, file: &std::path::Path) {
        if let Some(dir) = file.parent() {
            self.shared.prefs.write().last_import_export_path = dir.display().to_string();
            self.shared.save_prefs();
        }
    }

    fn handle_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.ui_rx.try_recv() {
            match msg {
                UiMsg::ShowMainWindow => self.show_main(ctx),
                UiMsg::ShowPicker => {
                    log::debug!("Opening the combo picker");
                    self.popups.lock().picker = Some(PickerState::new(&self.shared));
                }
                UiMsg::NewCombo => {
                    self.show_main(ctx);
                    let group = self.default_group_for_new_combo();
                    self.dialog = Some(Dialog::Combo(ComboEditor::new_combo(group)));
                }
                UiMsg::ShowPreferences => {
                    self.show_main(ctx);
                    self.prefs = Some(PrefsWindow::default());
                }
                UiMsg::PromptInput { description, reply } => {
                    let mut popups = self.popups.lock();
                    if let Some(previous) = popups.input.take() {
                        let _ = previous.reply.send(None);
                    }
                    popups.input = Some(InputPrompt::new(description, reply));
                    popups.input_linger_until = None;
                }
                UiMsg::ShortcutRecorded(sc) => self.assign_recorded(sc),
                UiMsg::Quit => self.quit(ctx),
            }
        }
    }

    pub fn default_group_for_new_combo(&self) -> Option<Uuid> {
        match self.group_filter {
            GroupFilter::Group(g) => Some(g),
            GroupFilter::All => self.shared.library.read().groups.first().map(|g| g.uuid),
        }
    }

    pub fn selected_set(&self) -> HashSet<Uuid> {
        self.selection.iter().copied().collect()
    }
}

/// Position that centers a window of the given size on the monitor of the root viewport.
pub fn centered_position(ctx: &egui::Context, size: egui::Vec2) -> Option<egui::Pos2> {
    let monitor = ctx.input(|i| i.viewport().monitor_size)?;
    if monitor.x < 100.0 || monitor.y < 100.0 {
        return None;
    }
    Some(egui::pos2(((monitor.x - size.x) / 2.0).max(0.0), ((monitor.y - size.y) / 2.5).max(0.0)))
}

impl eframe::App for ExpanderApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.pass += 1;
        if self.start_hidden && self.pass <= 3 {
            // eframe shows the window after the first frame: hide it again (it was created off-screen)
            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
            ctx.request_repaint();
        }
        self.handle_messages(ctx);

        if let Some(t) = &mut self.tray {
            t.update(self.shared.is_enabled());
        }

        let theme = self.shared.prefs.read().theme;
        if self.applied_theme != Some(theme) {
            style::apply_theme(ctx, theme);
            self.applied_theme = Some(theme);
        }
        if self.applied_language != crate::i18n::generation() {
            self.applied_language = crate::i18n::generation();
            self.platform_status = crate::platform::platform_status();
            self.shared.localize_default_group_names();
        }

        let popups_open = {
            let mut p = self.popups.lock();
            p.picker.is_some() || p.input_window_needed()
        };
        if popups_open || self.popups_shown {
            // run the ui pass even if the main window is hidden (or occluded), so that the popups can be shown
            // (or closed: a popup is closed by a ui pass that does not show it anymore)
            ctx.send_viewport_cmd(ViewportCommand::RequestPaintWhileHidden);
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.viewport().close_requested()) && !self.quitting {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.hide_main(&ctx);
        }

        if self.main_visible {
            self.main_window_ui(ui);
            self.dialogs_ui(&ctx);
            self.prefs_ui(&ctx);
            self.messages_ui(&ctx);
        }

        let (picker_open, input_open) = {
            let mut p = self.popups.lock();
            (p.picker.is_some(), p.input_window_needed())
        };
        if picker_open {
            self.picker_viewport(&ctx);
        }
        if input_open {
            self.input_viewport(&ctx);
        }
        self.popups_shown = picker_open || input_open;
        if self.recording.is_some() {
            // the hook reports shortcuts, but also accept them from egui (e.g. when the hook is not available)
            self.record_from_egui(&ctx);
        }
    }

    fn on_exit(&mut self) {
        let _ = self.shared.engine_tx.send(EngineMsg::Quit);
        if let Some(p) = self.popups.lock().input.take() {
            let _ = p.reply.send(None);
        }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
}

impl ExpanderApp {
    fn record_from_egui(&mut self, ctx: &egui::Context) {
        let pressed = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Key { key, pressed: true, modifiers, .. } => Some((*key, *modifiers)),
                _ => None,
            })
        });
        let Some((key, mods)) = pressed else { return };
        if key == egui::Key::Escape && !mods.any() {
            self.stop_recording();
            return;
        }
        if let Some(k) = egui_key_to_key(key) {
            let sc = Shortcut { ctrl: mods.ctrl, alt: mods.alt, shift: mods.shift, meta: mods.mac_cmd, key: k };
            if sc.is_valid() {
                self.assign_recorded(sc);
            }
        }
    }
}

fn egui_key_to_key(key: egui::Key) -> Option<crate::shortcut::Key> {
    use crate::shortcut::Key as K;
    use egui::Key as E;
    let name = key.name();
    Some(match key {
        E::Enter => K::Enter,
        E::Space => K::Space,
        E::Tab => K::Tab,
        E::Backspace => K::Backspace,
        E::Escape => K::Escape,
        E::Insert => K::Insert,
        E::Delete => K::Delete,
        E::Home => K::Home,
        E::End => K::End,
        E::PageUp => K::PageUp,
        E::PageDown => K::PageDown,
        E::ArrowLeft => K::Left,
        E::ArrowRight => K::Right,
        E::ArrowUp => K::Up,
        E::ArrowDown => K::Down,
        _ => {
            if let Some(n) = name.strip_prefix('F')
                && let Ok(i) = n.parse::<u8>()
            {
                return Some(K::F(i));
            }
            let mut chars = name.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => K::Char(c.to_ascii_uppercase()),
                _ => return None,
            }
        }
    })
}

/// The icon of the main window and of the combo editor window, drawn once.
fn window_icon() -> Arc<egui::IconData> {
    static ICON: std::sync::LazyLock<Arc<egui::IconData>> =
        std::sync::LazyLock::new(|| Arc::new(egui::IconData { rgba: crate::icon::render_rgba(64, true), width: 64, height: 64 }));
    ICON.clone()
}

/// Start the user interface. Blocks until the application quits.
pub fn run(shared: Arc<AppShared>, ui_rx: Receiver<UiMsg>, start_hidden: bool) -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Mantice")
        .with_app_id("mantice")
        .with_decorations(!title_bar::CUSTOM)
        .with_inner_size(title_bar::outer_size(egui::vec2(980.0, 640.0)))
        .with_min_inner_size(title_bar::outer_size(egui::vec2(680.0, 420.0)))
        .with_icon(window_icon());
    if start_hidden {
        viewport = viewport.with_position([-20000.0, -20000.0]);
    }
    let options = eframe::NativeOptions { viewport, centered: !start_hidden, ..Default::default() };
    eframe::run_native("Mantice", options, Box::new(move |cc| Ok(Box::new(ExpanderApp::new(cc, shared, ui_rx, start_hidden)))))
}
