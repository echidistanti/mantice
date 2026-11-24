//! Modal dialogs: combo and group editors, confirmations, import, backups, about, help...

use std::path::{Path, PathBuf};

use egui::{Align, Id, Layout, RichText};
use uuid::Uuid;

use super::main_window::{case_labels, matching_labels};
use super::style::{self, muted};
use super::{ExpanderApp, RecordTarget};
use crate::app_state::AppShared;
use crate::i18n::tr;
use crate::library::{self, Combo, ConflictPolicy, Group, ImportAnalysis, Library};

pub enum Dialog {
    Combo(ComboEditor),
    Group(GroupEditor),
    Confirm(Confirm),
    Import(ImportDialog),
    RestoreBackup(RestoreBackupDialog),
    AppList(AppListEditor),
    /// The path of the Beeftext list, and its number of combos and groups, or why it cannot be read.
    BeeftextMigration(PathBuf, Result<(usize, usize), String>),
    About,
    Help,
    EmojiList(super::emoji_list::EmojiListDialog),
}

// ---------------------------------------------------------------------------------------------------------------------
// Combo editor
// ---------------------------------------------------------------------------------------------------------------------

pub struct ComboEditor {
    pub combo: Combo,
    pub is_new: bool,
    cursor: Option<usize>,
    focused: bool,
}

impl ComboEditor {
    pub fn new_combo(group: Option<Uuid>) -> Self {
        Self { combo: Combo { group, ..Default::default() }, is_new: true, cursor: None, focused: false }
    }

    pub fn edit(combo: Combo) -> Self {
        Self { combo, is_new: false, cursor: None, focused: false }
    }

    fn insert(&mut self, text: &str) {
        let s = &mut self.combo.snippet;
        let total = s.chars().count();
        let at = self.cursor.unwrap_or(total).min(total);
        let byte = s.char_indices().nth(at).map(|(b, _)| b).unwrap_or(s.len());
        s.insert_str(byte, text);
        self.cursor = Some(at + text.chars().count());
    }
}

/// The entries of the "Insert variable" menu: (label, text inserted).
fn variable_entries() -> Vec<(&'static str, &'static str)> {
    vec![
        (tr("var-clipboard"), "#{clipboard}"),
        (tr("var-cursor"), "#{cursor}"),
        (tr("var-date"), "#{date}"),
        (tr("var-time"), "#{time}"),
        (tr("var-date-time"), "#{dateTime}"),
        (tr("var-custom-date-time"), "#{dateTime:yyyy-MM-dd HH:mm}"),
        (tr("var-shifted-date-time"), "#{dateTime:+1d:yyyy-MM-dd}"),
        (tr("label-combo"), "#{combo:}"),
        (tr("var-combo-upper"), "#{upper:}"),
        (tr("var-combo-lower"), "#{lower:}"),
        (tr("var-combo-trim"), "#{trim:}"),
        (tr("var-input"), "#{input:}"),
        (tr("var-env"), "#{envVar:}"),
        (tr("var-powershell"), "#{powershell:}"),
        (tr("var-delay"), "#{delay:500}"),
    ]
}

// ---------------------------------------------------------------------------------------------------------------------
// Other dialogs
// ---------------------------------------------------------------------------------------------------------------------

pub struct GroupEditor {
    pub group: Group,
    pub is_new: bool,
    focused: bool,
}

impl GroupEditor {
    pub fn new_group() -> Self {
        Self { group: Group::new("", ""), is_new: true, focused: false }
    }
    pub fn edit(group: Group) -> Self {
        Self { group, is_new: false, focused: false }
    }
}

pub enum ConfirmAction {
    DeleteCombos(Vec<Uuid>),
    DeleteGroup(Uuid),
    RestoreFrom(PathBuf),
    ResetPreferences,
    RemoveBackups,
}

pub struct Confirm {
    text: String,
    action: ConfirmAction,
}

impl Confirm {
    pub fn new(text: String, action: ConfirmAction) -> Self {
        Self { text, action }
    }
}

pub struct ImportDialog {
    path: PathBuf,
    analysis: Option<ImportAnalysis>,
    policy: ConflictPolicy,
    group: Option<Uuid>,
}

impl ImportDialog {
    pub fn open(shared: &AppShared, path: &Path, group: Option<Uuid>) -> anyhow::Result<Self> {
        let combos = library::read_import_file(path)?;
        let analysis = library::analyze_import(&shared.library.read(), combos);
        Ok(Self { path: path.to_path_buf(), analysis: Some(analysis), policy: ConflictPolicy::Skip, group })
    }
}

pub struct RestoreBackupDialog {
    files: Vec<(PathBuf, String)>,
    selected: Option<usize>,
}

impl RestoreBackupDialog {
    pub fn new(dir: &Path) -> Self {
        let mut files: Vec<(PathBuf, String)> = crate::backup::list(dir)
            .into_iter()
            .map(|p| {
                let label = crate::backup::backup_date(&p).map(|d| d.format("%Y-%m-%d  %H:%M:%S").to_string()).unwrap_or_default();
                (p, label)
            })
            .collect();
        files.reverse();
        Self { files, selected: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppListKind {
    Excluded,
    Sensitive,
    EmojiExcluded,
}

pub struct AppListEditor {
    kind: AppListKind,
    text: String,
}

impl AppListEditor {
    pub fn new(shared: &AppShared, kind: AppListKind) -> Self {
        let list = match kind {
            AppListKind::Excluded => shared.excluded_apps.read().clone(),
            AppListKind::Sensitive => shared.sensitive_apps.read().clone(),
            AppListKind::EmojiExcluded => shared.emoji_excluded_apps.read().clone(),
        };
        Self { kind, text: list.patterns.join("\n") }
    }
}

/// The OK and Cancel buttons at the bottom of a dialog. Returns (OK clicked, Cancel clicked).
fn buttons_row(ui: &mut egui::Ui, ok_label: &str, ok_enabled: bool) -> (bool, bool) {
    ui.add_space(8.0);
    right_aligned(ui, |ui| {
        let accent = style::accent(ui);
        let b = egui::Button::new(RichText::new(ok_label).color(egui::Color32::WHITE)).fill(accent).min_size(egui::vec2(90.0, 0.0));
        let ok = ui.add_enabled(ok_enabled, b).clicked();
        (ok, ui.add(egui::Button::new(tr("app-cancel")).min_size(egui::vec2(90.0, 0.0))).clicked())
    })
}

pub(super) fn right_aligned<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 30.0), Layout::right_to_left(Align::Center), add).inner
}

pub(super) fn ok_button(ui: &mut egui::Ui) -> bool {
    ui.add(egui::Button::new(tr("app-ok")).min_size(egui::vec2(90.0, 0.0))).clicked()
}

pub(super) fn modal(id: &str) -> egui::Modal {
    egui::Modal::new(Id::unique(id))
}

/// Show a modal whose body returns (OK, Cancel), usually from [`buttons_row`]. Some(true) for OK, Some(false) when
/// cancelled or closed (Escape, click outside), None while it stays open.
fn ok_cancel(ctx: &egui::Context, id: &str, body: impl FnOnce(&mut egui::Ui) -> (bool, bool)) -> Option<bool> {
    let mut answer = (false, false);
    let r = modal(id).show(ctx, |ui| answer = body(ui));
    let (ok, cancel) = answer;
    if cancel || (r.should_close() && !ok) { Some(false) } else { ok.then_some(true) }
}

/// Like [`ok_cancel`], in a native window of its own. The main window stays dimmed and blocked as under a modal; a
/// click on it brings the window back to the front.
fn ok_cancel_window(
    ctx: &egui::Context,
    id: &str,
    title: &str,
    size: egui::Vec2,
    mut body: impl FnMut(&mut egui::Ui) -> (bool, bool),
) -> Option<bool> {
    let viewport = egui::ViewportId::from_hash_of(id);
    if modal(id).frame(egui::Frame::NONE).show(ctx, |_| {}).backdrop_response.clicked() {
        ctx.send_viewport_cmd_to(viewport, egui::ViewportCommand::Focus);
    }
    let size = super::title_bar::outer_size(size);
    let mut builder = egui::ViewportBuilder::default()
        .with_title(title)
        .with_decorations(!super::title_bar::CUSTOM)
        .with_inner_size(size)
        .with_min_inner_size(super::title_bar::outer_size(egui::vec2(520.0, 460.0)))
        .with_icon(super::window_icon())
        .with_active(true);
    if let Some(pos) = super::centered_position(ctx, size) {
        builder = builder.with_position(pos);
    }
    ctx.show_viewport_immediate(viewport, builder, |ui, _class| {
        super::title_bar::show(ui, title, super::title_bar::Buttons::Close, true);
        let (ok, cancel) = egui::CentralPanel::default().show(ui, |ui| body(ui)).inner;
        // Escape closes it like a modal: not while a menu or a list is open
        let escape = !egui::Popup::is_any_open(ui.ctx()) && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        let closed = escape || ui.input(|i| i.viewport().close_requested());
        if cancel || (closed && !ok) { Some(false) } else { ok.then_some(true) }
    })
}

// ---------------------------------------------------------------------------------------------------------------------

impl ExpanderApp {
    pub(super) fn dialogs_ui(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.dialog.take() else { return };
        let keep = match &mut dialog {
            Dialog::Combo(ed) => self.combo_editor_ui(ctx, ed),
            Dialog::Group(ed) => self.group_editor_ui(ctx, ed),
            Dialog::Confirm(c) => self.confirm_ui(ctx, c),
            Dialog::Import(d) => self.import_ui(ctx, d),
            Dialog::RestoreBackup(d) => self.restore_backup_ui(ctx, d),
            Dialog::AppList(d) => self.app_list_ui(ctx, d),
            Dialog::BeeftextMigration(p, summary) => self.migration_ui(ctx, p.clone(), summary),
            Dialog::About => about_ui(ctx, &self.shared),
            Dialog::Help => help_ui(ctx),
            Dialog::EmojiList(d) => self.emoji_list_ui(ctx, d),
        };
        if keep && self.dialog.is_none() {
            self.dialog = Some(dialog);
        }
    }

    pub(super) fn messages_ui(&mut self, ctx: &egui::Context) {
        if self.messages.is_empty() || self.dialog.is_some() {
            return;
        }
        let (title, text) = self.messages[0].clone();
        let mut close = false;
        let r = modal("message").show(ctx, |ui| {
            ui.set_max_width(460.0);
            ui.heading(&title);
            ui.add_space(6.0);
            ui.label(&text);
            ui.add_space(10.0);
            close = right_aligned(ui, ok_button) || ui.input(|i| i.key_pressed(egui::Key::Enter));
        });
        if close || r.should_close() {
            self.messages.remove(0);
        }
    }

    fn combo_editor_ui(&mut self, ctx: &egui::Context, ed: &mut ComboEditor) -> bool {
        if let Some(sc) = self.recorded_snippet_shortcut.take() {
            ed.insert(&format!("#{{shortcut:{sc}}}"));
        }
        let prefs = self.shared.prefs.read().clone();
        let (groups, duplicate): (Vec<(Uuid, String)>, Option<String>) = {
            let lib = self.shared.library.read();
            let dup = (!ed.combo.keyword.is_empty())
                .then(|| lib.combos.iter().find(|c| c.uuid != ed.combo.uuid && c.keyword == ed.combo.keyword).map(|c| c.display_name()))
                .flatten();
            (lib.groups.iter().map(|g| (g.uuid, g.name.clone())).collect(), dup)
        };
        let snippet_id = Id::unique("combo_snippet_editor");
        let title = if ed.is_new { tr("label-new-combo") } else { tr("editor-edit-title") };
        let answer = ok_cancel_window(ctx, "combo_editor", title, egui::vec2(860.0, 560.0), |ui| {
            egui::Grid::new("combo_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                ui.label(tr("label-name"));
                let r = ui.add(egui::TextEdit::singleline(&mut ed.combo.name).hint_text(tr("app-optional")).desired_width(f32::INFINITY));
                if !ed.focused {
                    r.request_focus();
                    ed.focused = true;
                }
                ui.end_row();

                ui.label(tr("label-keyword"));
                ui.vertical(|ui| {
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut ed.combo.keyword)
                            .font(egui::TextStyle::Monospace)
                            .hint_text(tr("editor-keyword-hint"))
                            .desired_width(f32::INFINITY),
                    );
                    if r.changed() {
                        ed.combo.keyword.retain(|c| !c.is_whitespace() && !c.is_control());
                    }
                    let kw_len = ed.combo.keyword.chars().count();
                    if kw_len == 0 && prefs.warn_about_empty_keyword {
                        ui.label(RichText::new(tr("editor-warn-empty-keyword")).color(style::WARNING).small());
                    } else if kw_len > 0 && kw_len < 3 && prefs.warn_about_short_keyword {
                        ui.label(RichText::new(tr("editor-warn-short-keyword")).color(style::WARNING).small());
                    }
                    if let Some(other) = &duplicate {
                        ui.label(RichText::new(tr!("editor-warn-duplicate-keyword", name = other.as_str())).color(style::WARNING).small());
                    }
                });
                ui.end_row();

                ui.label(tr("label-group"));
                let current = groups.iter().find(|(u, _)| Some(*u) == ed.combo.group).map(|(_, n)| n.clone()).unwrap_or_default();
                egui::ComboBox::from_id_salt("combo_group").selected_text(current).width(240.0).show_ui(ui, |ui| {
                    for (u, n) in &groups {
                        ui.selectable_value(&mut ed.combo.group, Some(*u), n);
                    }
                });
                ui.end_row();

                ui.label(tr("editor-matching"));
                ui.horizontal(|ui| {
                    let ml = matching_labels(&prefs);
                    let current = ml.iter().find(|(m, _)| *m == ed.combo.matching_mode).map(|(_, l)| l.clone()).unwrap_or_default();
                    egui::ComboBox::from_id_salt("combo_matching").selected_text(current).width(170.0).show_ui(ui, |ui| {
                        for (m, l) in &ml {
                            ui.selectable_value(&mut ed.combo.matching_mode, *m, l);
                        }
                    });
                    ui.add_space(8.0);
                    let cl = case_labels(&prefs);
                    let current = cl.iter().find(|(c, _)| *c == ed.combo.case_sensitivity).map(|(_, l)| l.clone()).unwrap_or_default();
                    egui::ComboBox::from_id_salt("combo_case").selected_text(current).width(220.0).show_ui(ui, |ui| {
                        for (c, l) in &cl {
                            ui.selectable_value(&mut ed.combo.case_sensitivity, *c, l);
                        }
                    });
                });
                ui.end_row();

                ui.label(tr("editor-description"));
                ui.add(egui::TextEdit::singleline(&mut ed.combo.description).hint_text(tr("app-optional")).desired_width(f32::INFINITY));
                ui.end_row();
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr("label-snippet")).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.menu_button(format!("{} ⏷", tr("editor-insert-variable")), |ui| {
                        for (label, text) in variable_entries() {
                            if ui.button(format!("{label}   {text}")).clicked() {
                                ed.insert(text);
                                ui.close();
                            }
                        }
                        ui.menu_button(tr("var-key"), |ui| {
                            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                                for k in crate::shortcut::Key::variable_names() {
                                    if ui.button(format!("#{{key:{k}}}")).clicked() {
                                        ed.insert(&format!("#{{key:{k}}}"));
                                        ui.close();
                                    }
                                }
                            });
                        });
                        if ui.button(tr("var-shortcut-record")).clicked() {
                            self.start_recording(RecordTarget::SnippetVariable);
                            ui.close();
                        }
                    });
                    if self.recording == Some(RecordTarget::SnippetVariable) {
                        ui.label(RichText::new(tr("editor-recording-shortcut")).color(style::accent(ui)));
                    }
                });
            });
            let out = egui::TextEdit::multiline(&mut ed.combo.snippet)
                .id(snippet_id)
                .font(egui::TextStyle::Monospace)
                .desired_rows(10)
                .desired_width(f32::INFINITY)
                .hint_text(tr("editor-snippet-hint"))
                .show(ui);
            if let Some(range) = out.cursor_range {
                ed.cursor = Some(range.primary.index.0);
            }
            ui.add_space(4.0);
            ui.checkbox(&mut ed.combo.enabled, tr("editor-combo-enabled"));
            if ed.combo.snippet.is_empty() {
                ui.label(muted(ui, tr("editor-snippet-empty")).small());
            }
            let (ok, cancel) = buttons_row(ui, tr("app-ok"), !ed.combo.snippet.is_empty());
            (ok || (ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter)) && !ed.combo.snippet.is_empty()), cancel)
        });
        if self.recording == Some(RecordTarget::SnippetVariable) {
            return true;
        }
        if answer != Some(true) {
            return answer.is_none();
        }
        let combo = ed.combo.clone();
        let uuid = combo.uuid;
        {
            let mut lib = self.shared.library.write();
            match lib.combo_mut(&uuid) {
                Some(existing) if !ed.is_new => {
                    if *existing != combo {
                        let mut c = combo;
                        c.touch();
                        c.last_use = existing.last_use;
                        *existing = c;
                    }
                }
                // new, or deleted meanwhile by another computer sharing the combo list: added
                _ => {
                    let mut c = combo;
                    let n = library::now();
                    c.created = n;
                    c.modified = n;
                    lib.combos.push(c);
                }
            }
        }
        self.selection = vec![uuid];
        self.commit_library();
        false
    }

    fn group_editor_ui(&mut self, ctx: &egui::Context, ed: &mut GroupEditor) -> bool {
        let answer = ok_cancel(ctx, "group_editor", |ui| {
            ui.set_width(420.0);
            ui.heading(if ed.is_new { tr("group-new-title") } else { tr("group-edit-title") });
            ui.add_space(8.0);
            egui::Grid::new("group_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                ui.label(tr("label-name"));
                let r = ui.add(egui::TextEdit::singleline(&mut ed.group.name).desired_width(f32::INFINITY));
                if !ed.focused {
                    r.request_focus();
                    ed.focused = true;
                }
                ui.end_row();
                ui.label(tr("editor-description"));
                ui.add(egui::TextEdit::singleline(&mut ed.group.description).desired_width(f32::INFINITY));
                ui.end_row();
            });
            ui.checkbox(&mut ed.group.enabled, tr("app-enabled"));
            let valid = !ed.group.name.trim().is_empty();
            let (ok, cancel) = buttons_row(ui, tr("app-ok"), valid);
            (ok || (valid && ui.input(|i| i.key_pressed(egui::Key::Enter))), cancel)
        });
        if answer != Some(true) {
            return answer.is_none();
        }
        let mut g = ed.group.clone();
        g.name = g.name.trim().to_string();
        let uuid = g.uuid;
        {
            let mut lib = self.shared.library.write();
            match lib.group_mut(&uuid) {
                Some(existing) if !ed.is_new => {
                    if *existing != g {
                        g.touch();
                        *existing = g;
                    }
                }
                // new, or deleted meanwhile by another computer sharing the combo list: added
                _ => lib.groups.push(g),
            }
        }
        if ed.is_new {
            self.group_filter = super::GroupFilter::Group(uuid);
        }
        self.commit_library();
        false
    }

    fn confirm_ui(&mut self, ctx: &egui::Context, c: &mut Confirm) -> bool {
        let answer = ok_cancel(ctx, "confirm", |ui| {
            ui.set_max_width(440.0);
            ui.heading(tr("app-confirmation"));
            ui.add_space(6.0);
            ui.label(&c.text);
            buttons_row(ui, tr("app-yes"), true)
        });
        if answer != Some(true) {
            return answer.is_none();
        }
        match &c.action {
            ConfirmAction::DeleteCombos(ids) => {
                self.shared.library.write().combos.retain(|x| !ids.contains(&x.uuid));
                self.selection.clear();
                self.commit_library();
            }
            ConfirmAction::DeleteGroup(g) => {
                {
                    let mut lib = self.shared.library.write();
                    lib.combos.retain(|x| x.group != Some(*g));
                    lib.groups.retain(|x| x.uuid != *g);
                    lib.ensure_correct_grouping(tr("app-default-group"));
                }
                self.group_filter = super::GroupFilter::All;
                self.selection.clear();
                self.commit_library();
            }
            ConfirmAction::RestoreFrom(path) => self.replace_library_from(path),
            ConfirmAction::ResetPreferences => {
                self.shared.prefs.write().reset_to_defaults();
                let p = self.shared.prefs.read().clone();
                crate::i18n::set_language(p.language);
                let _ = crate::platform::set_autostart(p.auto_start_at_login);
                self.commit_prefs();
            }
            ConfirmAction::RemoveBackups => {
                let n = crate::backup::remove_all(&self.shared.backup_dir());
                self.info(tr!("msg-backups-removed", count = n));
            }
        }
        false
    }

    pub(super) fn replace_library_from(&mut self, path: &Path) {
        match Library::load(path) {
            Ok(parsed) => {
                let mut lib = parsed.library;
                if let Ok(text) = std::fs::read_to_string(self.shared.paths.last_use_file()) {
                    lib.apply_last_use_json(&text);
                }
                lib.ensure_correct_grouping(tr("app-default-group"));
                let n = lib.combos.len();
                self.group_filter = super::GroupFilter::All;
                self.selection.clear();
                match self.shared.replace_library(lib) {
                    Ok(()) => self.info(tr!("msg-combos-restored", count = n)),
                    Err(e) => self.error(format!("{}\n{e:#}", tr("msg-library-save-failed"))),
                }
            }
            Err(e) => self.error(format!("{}\n{e:#}", tr("msg-load-failed"))),
        }
    }

    fn import_ui(&mut self, ctx: &egui::Context, d: &mut ImportDialog) -> bool {
        let groups: Vec<(Uuid, String)> = self.shared.library.read().groups.iter().map(|g| (g.uuid, g.name.clone())).collect();
        let Some(analysis) = d.analysis.as_ref() else { return false };
        let answer = ok_cancel(ctx, "import", |ui| {
            ui.set_width(480.0);
            ui.heading(tr("import-title"));
            ui.add_space(4.0);
            ui.label(muted(ui, d.path.display().to_string()));
            ui.add_space(8.0);
            ui.label(tr!("import-new-combos", count = analysis.new_combos.len()));
            if analysis.conflict_count() > 0 {
                ui.add_space(6.0);
                ui.label(RichText::new(tr!("import-conflicts", count = analysis.conflict_count())).color(style::WARNING));
                ui.radio_value(&mut d.policy, ConflictPolicy::Skip, tr("import-skip"));
                ui.radio_value(&mut d.policy, ConflictPolicy::Overwrite, tr("import-overwrite"));
                if !analysis.conflicting_newer.is_empty() {
                    ui.radio_value(
                        &mut d.policy,
                        ConflictPolicy::OverwriteOlder,
                        tr!("import-overwrite-older", count = analysis.conflicting_newer.len()),
                    );
                }
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(tr("import-into-group"));
                let current = groups.iter().find(|(u, _)| Some(*u) == d.group).map(|(_, n)| n.clone()).unwrap_or_default();
                egui::ComboBox::from_id_salt("import_group").selected_text(current).width(220.0).show_ui(ui, |ui| {
                    for (u, n) in &groups {
                        ui.selectable_value(&mut d.group, Some(*u), n);
                    }
                });
            });
            let count = analysis.importable_count(d.policy);
            buttons_row(ui, &tr!("import-button", count = count), count > 0)
        });
        if answer != Some(true) {
            return answer.is_none();
        }
        let analysis = d.analysis.take().unwrap_or_default();
        let n = library::apply_import(&mut self.shared.library.write(), analysis, d.policy, d.group);
        self.commit_library();
        self.info(tr!("msg-combos-imported", count = n));
        false
    }

    fn restore_backup_ui(&mut self, ctx: &egui::Context, d: &mut RestoreBackupDialog) -> bool {
        let mut remove_all = false;
        let answer = ok_cancel(ctx, "restore_backup", |ui| {
            ui.set_width(420.0);
            ui.heading(tr("restore-auto-title"));
            ui.add_space(4.0);
            ui.label(muted(ui, tr("restore-auto-info")));
            ui.add_space(6.0);
            if d.files.is_empty() {
                ui.label(tr("restore-no-backup"));
            } else {
                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    for (i, (_, label)) in d.files.iter().enumerate() {
                        if ui.selectable_label(d.selected == Some(i), label).clicked() {
                            d.selected = Some(i);
                        }
                    }
                });
                ui.add_space(4.0);
                if ui.button(RichText::new(tr("restore-remove-all")).color(style::DANGER)).clicked() {
                    remove_all = true;
                }
            }
            buttons_row(ui, tr("restore-action"), d.selected.is_some())
        });
        if remove_all {
            self.dialog = Some(Dialog::Confirm(Confirm::new(tr("confirm-remove-backups").to_string(), ConfirmAction::RemoveBackups)));
            return false;
        }
        if answer != Some(true) {
            return answer.is_none();
        }
        if let Some(i) = d.selected {
            let path = d.files[i].0.clone();
            self.dialog = Some(Dialog::Confirm(Confirm::new(tr("confirm-replace-library").to_string(), ConfirmAction::RestoreFrom(path))));
            return false;
        }
        true
    }

    fn app_list_ui(&mut self, ctx: &egui::Context, d: &mut AppListEditor) -> bool {
        let (title, help) = match d.kind {
            AppListKind::Excluded => (tr("applist-excluded-title"), tr("applist-excluded-help")),
            AppListKind::Sensitive => (tr("applist-sensitive-title"), tr("applist-sensitive-help")),
            AppListKind::EmojiExcluded => (tr("applist-emoji-title"), tr("applist-emoji-help")),
        };
        let answer = ok_cancel(ctx, "app_list", |ui| {
            ui.set_width(440.0);
            ui.heading(title);
            ui.add_space(4.0);
            ui.label(muted(ui, help));
            ui.label(muted(ui, tr("applist-format")));
            if let Some(app) = crate::platform::active_app_name() {
                ui.label(muted(ui, tr!("applist-example", name = app)).small());
            }
            ui.add_space(6.0);
            ui.add(egui::TextEdit::multiline(&mut d.text).font(egui::TextStyle::Monospace).desired_rows(10).desired_width(f32::INFINITY));
            buttons_row(ui, tr("app-ok"), true)
        });
        if answer != Some(true) {
            return answer.is_none();
        }
        let patterns: Vec<String> = d.text.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect();
        let list = crate::process_list::ProcessList { patterns };
        match d.kind {
            AppListKind::Excluded => *self.shared.excluded_apps.write() = list,
            AppListKind::Sensitive => *self.shared.sensitive_apps.write() = list,
            AppListKind::EmojiExcluded => *self.shared.emoji_excluded_apps.write() = list,
        }
        self.shared.save_app_lists();
        false
    }

    fn migration_ui(&mut self, ctx: &egui::Context, path: PathBuf, summary: &Result<(usize, usize), String>) -> bool {
        let answer = ok_cancel(ctx, "migration", |ui| {
            ui.set_max_width(460.0);
            ui.heading(tr("migration-title"));
            ui.add_space(6.0);
            ui.label(tr("migration-text"));
            match summary {
                Ok((c, g)) => ui.label(RichText::new(tr!("app-status-counts", combos = *c, groups = *g)).strong()),
                // why it cannot be read: for an old Beeftext format, to export it from an up-to-date Beeftext
                Err(e) => ui.label(RichText::new(e).color(style::WARNING)),
            };
            ui.label(muted(ui, path.display().to_string()).small());
            buttons_row(ui, tr("import-action"), summary.is_ok())
        });
        if answer == Some(true) {
            self.replace_library_from(&path);
        }
        answer.is_none()
    }
}

fn about_ui(ctx: &egui::Context, shared: &AppShared) -> bool {
    let tex = ctx.data_mut(|d| d.get_temp::<egui::TextureHandle>(Id::unique("about_icon")));
    let tex = tex.unwrap_or_else(|| {
        let img = egui::ColorImage::from_rgba_unmultiplied([96, 96], &crate::icon::render_rgba(96, true));
        let t = ctx.load_texture("about_icon", img, egui::TextureOptions::LINEAR);
        ctx.data_mut(|d| d.insert_temp(Id::unique("about_icon"), t.clone()));
        t
    });
    let mut close = false;
    let r = modal("about").show(ctx, |ui| {
        ui.set_width(440.0);
        ui.vertical_centered(|ui| {
            ui.image((tex.id(), egui::vec2(72.0, 72.0)));
            ui.add_space(4.0);
            ui.label(RichText::new("Mantice").size(24.0).strong());
            ui.label(muted(ui, tr!("about-version", version = env!("CARGO_PKG_VERSION"))));
            ui.add_space(8.0);
            ui.label(tr("about-tagline"));
            ui.add_space(6.0);
            ui.label(muted(ui, tr("about-credits-beeftext")).small());
            ui.label(muted(ui, tr("about-credits-emoji")).small());
            ui.add_space(6.0);
            ui.label(muted(ui, tr!("about-data-folder", path = shared.paths.data_dir.display().to_string())).small());
            ui.add_space(10.0);
            close = ok_button(ui);
        });
    });
    !(close || r.should_close())
}

fn help_ui(ctx: &egui::Context) -> bool {
    let help_combo = tr!("help-combo", upper = "#{upper:}", lower = "#{lower:}", trim = "#{trim:}");
    let entries: Vec<(&str, &str)> = vec![
        ("#{clipboard}", tr("help-clipboard")),
        ("#{cursor}", tr("help-cursor")),
        ("#{date} #{time} #{dateTime}", tr("help-date")),
        ("#{dateTime:yyyy-MM-dd HH:mm}", tr("help-custom-format")),
        ("#{dateTime:+1d-2h:format}", tr("help-shift")),
        ("#{combo:keyword}", help_combo.as_str()),
        ("#{input:Description}", tr("help-input")),
        ("#{envVar:NAME}", tr("help-env")),
        ("#{powershell:path[:timeoutMs]}", tr("help-powershell")),
        ("#{key:name[:count]}", tr("help-key")),
        ("#{shortcut:Ctrl+Shift+J}", tr("help-shortcut")),
        ("#{delay:ms}", tr("help-delay")),
    ];
    let mut close = false;
    let r = modal("help").show(ctx, |ui| {
        ui.set_width(640.0);
        ui.heading(tr("menu-variables-reference"));
        ui.add_space(4.0);
        ui.label(muted(ui, tr!("help-intro", escaped_brace = "\\}", escaped_backslash = "\\\\", brace = "}", backslash = "\\")));
        ui.add_space(6.0);
        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            egui::Grid::new("help_grid").num_columns(2).striped(true).spacing([14.0, 8.0]).show(ui, |ui| {
                for (v, d) in &entries {
                    ui.label(RichText::new(*v).monospace().color(style::accent(ui)));
                    ui.add(egui::Label::new(*d).wrap());
                    ui.end_row();
                }
            });
        });
        ui.add_space(8.0);
        close = right_aligned(ui, ok_button);
    });
    !(close || r.should_close())
}
