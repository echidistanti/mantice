//! The main window: menu bar, group list, combo table and status bar.

use egui::{Align, Layout, RichText, Sense};
use egui_extras::{Column, TableBuilder};
use uuid::Uuid;

use super::dialogs::{ComboEditor, Confirm, ConfirmAction, Dialog, GroupEditor, ImportDialog, RestoreBackupDialog};
use super::espanso_dialog::EspansoDialog;
use super::style::{self, muted, preview};
use super::{ExpanderApp, PrefsWindow};
use crate::i18n::tr;
use crate::library::{CaseSensitivity, Library, MatchingMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupFilter {
    All,
    Group(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Enabled,
    Name,
    Keyword,
    Snippet,
    Group,
    LastUsed,
}

/// Actions collected while drawing, applied afterwards (so that no lock is held while drawing).
pub enum Action {
    NewCombo,
    EditCombo(Uuid),
    Duplicate,
    DeleteSelected,
    SetEnabled(Vec<Uuid>, bool),
    SetMatching(MatchingMode),
    SetCase(CaseSensitivity),
    MoveToGroup(Uuid),
    CopySnippet(Uuid),
    NewGroup,
    EditGroup(Uuid),
    DeleteGroup(Uuid),
    ToggleGroup(Uuid),
    MoveGroup(Uuid, isize),
    Import,
    ImportEspanso,
    Export(bool),
    Backup,
    Restore,
    RestoreAuto,
    OpenDataFolder,
    OpenLog,
    Preferences,
    About,
    Help,
    EmojiList,
    Hide,
    Quit,
    ToggleEnabled,
    SelectAll,
    Click(Uuid, egui::Modifiers),
}

struct Row {
    uuid: Uuid,
    enabled: bool,
    usable: bool,
    name: String,
    keyword: String,
    snippet: String,
    group: String,
    last_used: Option<chrono::NaiveDateTime>,
}

impl ExpanderApp {
    pub(super) fn main_window_ui(&mut self, ui: &mut egui::Ui) {
        let mut actions: Vec<Action> = Vec::new();
        self.shortcuts(ui, &mut actions);

        super::title_bar::show(ui, "Mantice", super::title_bar::Buttons::All, true);
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| self.menu_bar(ui, &mut actions));
        });
        egui::Panel::bottom("status_bar").show(ui, |ui| self.status_bar(ui, &mut actions));
        egui::Panel::left("groups").resizable(true).default_size(210.0).min_size(160.0).show(ui, |ui| {
            self.groups_panel(ui, &mut actions);
        });
        egui::CentralPanel::default().show(ui, |ui| self.combos_panel(ui, &mut actions));

        for a in actions {
            self.apply(ui.ctx(), a);
        }
    }

    fn shortcuts(&mut self, ui: &egui::Ui, actions: &mut Vec<Action>) {
        if self.dialog.is_some() || self.recording.is_some() {
            return;
        }
        let text_focused = ui.ctx().memory(|m| m.focused().is_some());
        ui.input(|i| {
            if i.modifiers.command && i.key_pressed(egui::Key::N) {
                actions.push(Action::NewCombo);
            }
            if i.modifiers.command && i.key_pressed(egui::Key::F) {
                self.focus_search = true;
            }
            if i.modifiers.command && i.key_pressed(egui::Key::D) && !self.selection.is_empty() {
                actions.push(Action::Duplicate);
            }
            if !text_focused {
                if i.key_pressed(egui::Key::Delete) && !self.selection.is_empty() {
                    actions.push(Action::DeleteSelected);
                }
                if i.key_pressed(egui::Key::Enter) && self.selection.len() == 1 {
                    actions.push(Action::EditCombo(self.selection[0]));
                }
                if i.modifiers.command && i.key_pressed(egui::Key::A) {
                    actions.push(Action::SelectAll);
                }
            }
        });
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let has_sel = !self.selection.is_empty();
        ui.menu_button(tr("menu-file"), |ui| {
            if ui.button(tr("menu-import")).clicked() {
                actions.push(Action::Import);
            }
            if ui.button(tr("menu-import-espanso")).clicked() {
                actions.push(Action::ImportEspanso);
            }
            if ui.button(tr("menu-export-all")).clicked() {
                actions.push(Action::Export(false));
            }
            if ui.add_enabled(has_sel, egui::Button::new(tr("menu-export-selected"))).clicked() {
                actions.push(Action::Export(true));
            }
            ui.separator();
            if ui.button(tr("menu-preferences")).clicked() {
                actions.push(Action::Preferences);
            }
            ui.separator();
            if ui.button(tr("menu-hide-window")).clicked() {
                actions.push(Action::Hide);
            }
            if ui.button(tr("menu-quit")).clicked() {
                actions.push(Action::Quit);
            }
        });
        ui.menu_button(tr("label-combo"), |ui| self.combo_menu(ui, actions, true));
        ui.menu_button(tr("label-group"), |ui| {
            if ui.button(tr("menu-new-group")).clicked() {
                actions.push(Action::NewGroup);
            }
            if let super::GroupFilter::Group(g) = self.group_filter {
                if ui.button(tr("menu-edit-group")).clicked() {
                    actions.push(Action::EditGroup(g));
                }
                if ui.button(tr("menu-toggle-group")).clicked() {
                    actions.push(Action::ToggleGroup(g));
                }
                if ui.button(tr("menu-delete-group")).clicked() {
                    actions.push(Action::DeleteGroup(g));
                }
            }
        });
        ui.menu_button(tr("menu-tools"), |ui| {
            if ui.button(tr("menu-backup")).clicked() {
                actions.push(Action::Backup);
            }
            if ui.button(tr("menu-restore")).clicked() {
                actions.push(Action::Restore);
            }
            if ui.button(tr("menu-restore-auto")).clicked() {
                actions.push(Action::RestoreAuto);
            }
            ui.separator();
            if ui.button(tr("menu-open-data-folder")).clicked() {
                actions.push(Action::OpenDataFolder);
            }
            if ui.button(tr("menu-open-log")).clicked() {
                actions.push(Action::OpenLog);
            }
        });
        ui.menu_button(tr("menu-help"), |ui| {
            if ui.button(tr("menu-variables-reference")).clicked() {
                actions.push(Action::Help);
            }
            if ui.button(tr("menu-emoji-list")).clicked() {
                actions.push(Action::EmojiList);
            }
            if ui.button(tr("menu-about")).clicked() {
                actions.push(Action::About);
            }
        });
    }

    /// Combo actions, shared by the menu bar and the context menu of the table.
    fn combo_menu(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>, include_new: bool) {
        let n = self.selection.len();
        if include_new && ui.button(tr("action-new-combo")).clicked() {
            actions.push(Action::NewCombo);
            ui.close();
        }
        if ui.add_enabled(n == 1, egui::Button::new(tr("action-edit"))).clicked() {
            actions.push(Action::EditCombo(self.selection[0]));
            ui.close();
        }
        if ui.add_enabled(n > 0, egui::Button::new(tr("action-duplicate"))).clicked() {
            actions.push(Action::Duplicate);
            ui.close();
        }
        if ui.add_enabled(n == 1, egui::Button::new(tr("action-copy-snippet"))).clicked() {
            actions.push(Action::CopySnippet(self.selection[0]));
            ui.close();
        }
        ui.separator();
        if ui.add_enabled(n > 0, egui::Button::new(tr("action-enable"))).clicked() {
            actions.push(Action::SetEnabled(self.selection.clone(), true));
            ui.close();
        }
        if ui.add_enabled(n > 0, egui::Button::new(tr("action-disable"))).clicked() {
            actions.push(Action::SetEnabled(self.selection.clone(), false));
            ui.close();
        }
        ui.add_enabled_ui(n > 0, |ui| {
            ui.menu_button(tr("action-matching-mode"), |ui| {
                for (m, label) in matching_labels(&self.shared.prefs.read()) {
                    if ui.button(label).clicked() {
                        actions.push(Action::SetMatching(m));
                        ui.close();
                    }
                }
            });
            ui.menu_button(tr("action-case-sensitivity"), |ui| {
                for (c, label) in case_labels(&self.shared.prefs.read()) {
                    if ui.button(label).clicked() {
                        actions.push(Action::SetCase(c));
                        ui.close();
                    }
                }
            });
            ui.menu_button(tr("action-move-to-group"), |ui| {
                let groups: Vec<(Uuid, String)> = self.shared.library.read().groups.iter().map(|g| (g.uuid, g.name.clone())).collect();
                for (g, name) in groups {
                    if ui.button(name).clicked() {
                        actions.push(Action::MoveToGroup(g));
                        ui.close();
                    }
                }
            });
        });
        ui.separator();
        if ui.add_enabled(n > 0, egui::Button::new(RichText::new(tr("action-delete")).color(style::DANGER))).clicked() {
            actions.push(Action::DeleteSelected);
            ui.close();
        }
    }

    fn groups_panel(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr("groups-title")).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.small_button("+").on_hover_text(tr("menu-new-group")).clicked() {
                    actions.push(Action::NewGroup);
                }
            });
        });
        ui.separator();
        let (groups, counts, total): (Vec<(Uuid, String, bool)>, Vec<usize>, usize) = {
            let lib = self.shared.library.read();
            let groups: Vec<_> = lib.groups.iter().map(|g| (g.uuid, g.name.clone(), g.enabled)).collect();
            let counts = groups.iter().map(|(u, _, _)| lib.combos.iter().filter(|c| c.group == Some(*u)).count()).collect();
            (groups, counts, lib.combos.len())
        };
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                let all_selected = self.group_filter == GroupFilter::All;
                if ui.selectable_label(all_selected, format!("{}  ({total})", tr("groups-all"))).clicked() {
                    self.group_filter = GroupFilter::All;
                }
                ui.add_space(2.0);
                let len = groups.len();
                for (i, ((uuid, name, enabled), count)) in groups.into_iter().zip(counts).enumerate() {
                    let selected = self.group_filter == GroupFilter::Group(uuid);
                    let mut text = RichText::new(format!("{name}  ({count})"));
                    if !enabled {
                        text = text.color(ui.visuals().weak_text_color()).italics();
                    }
                    let r = ui.selectable_label(selected, text);
                    if r.clicked() {
                        self.group_filter = GroupFilter::Group(uuid);
                        self.selection.clear();
                    }
                    if r.double_clicked() {
                        actions.push(Action::EditGroup(uuid));
                    }
                    r.context_menu(|ui| {
                        if ui.button(tr("menu-edit-group")).clicked() {
                            actions.push(Action::EditGroup(uuid));
                            ui.close();
                        }
                        if ui.button(if enabled { tr("group-disable") } else { tr("group-enable") }).clicked() {
                            actions.push(Action::ToggleGroup(uuid));
                            ui.close();
                        }
                        if ui.add_enabled(i > 0, egui::Button::new(tr("group-move-up"))).clicked() {
                            actions.push(Action::MoveGroup(uuid, -1));
                            ui.close();
                        }
                        if ui.add_enabled(i + 1 < len, egui::Button::new(tr("group-move-down"))).clicked() {
                            actions.push(Action::MoveGroup(uuid, 1));
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(RichText::new(tr("menu-delete-group")).color(style::DANGER)).clicked() {
                            actions.push(Action::DeleteGroup(uuid));
                            ui.close();
                        }
                    });
                }
            });
        });
    }

    fn rows(&self) -> Vec<Row> {
        let lib = self.shared.library.read();
        let words: Vec<String> = self.search.to_lowercase().split_whitespace().map(str::to_string).collect();
        let mut rows: Vec<Row> = lib
            .combos
            .iter()
            .filter(|c| match self.group_filter {
                GroupFilter::All => true,
                GroupFilter::Group(g) => c.group == Some(g),
            })
            .filter(|c| {
                if words.is_empty() {
                    return true;
                }
                let hay = format!("{}\n{}\n{}\n{}", c.name, c.keyword, c.snippet, c.description).to_lowercase();
                words.iter().all(|w| hay.contains(w.as_str()))
            })
            .map(|c| Row {
                uuid: c.uuid,
                enabled: c.enabled,
                usable: lib.is_usable(c),
                name: c.display_name(),
                keyword: c.keyword.clone(),
                snippet: preview(&c.snippet, 160),
                group: lib.group_name(&c.group),
                last_used: c.last_use,
            })
            .collect();
        // sorted on the column, then on the name; each text is put in lowercase once, not at each comparison
        let (col, asc) = self.sort;
        rows.sort_by_cached_key(|r| {
            let text = match col {
                SortColumn::Keyword => r.keyword.to_lowercase(),
                SortColumn::Snippet => r.snippet.to_lowercase(),
                SortColumn::Group => r.group.to_lowercase(),
                SortColumn::Enabled | SortColumn::Name | SortColumn::LastUsed => String::new(),
            };
            let enabled = col == SortColumn::Enabled && r.enabled;
            let last_used = if col == SortColumn::LastUsed { r.last_used } else { None };
            (enabled, last_used, text, r.name.to_lowercase())
        });
        if !asc {
            rows.reverse();
        }
        rows
    }

    fn combos_panel(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let search = egui::TextEdit::singleline(&mut self.search).hint_text(tr("combos-search")).desired_width(280.0);
            let r = ui.add(search);
            if self.focus_search {
                r.request_focus();
                self.focus_search = false;
            }
            if !self.search.is_empty() && ui.small_button("✖").on_hover_text(tr("app-clear")).clicked() {
                self.search.clear();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let accent = style::accent(ui);
                let new_btn =
                    egui::Button::new(RichText::new(format!("+ {}", tr("label-new-combo"))).color(egui::Color32::WHITE)).fill(accent);
                if ui.add(new_btn).on_hover_text("Ctrl+N").clicked() {
                    actions.push(Action::NewCombo);
                }
            });
        });
        ui.add_space(4.0);

        let rows = self.rows();
        if rows.is_empty() {
            self.empty_state(ui, actions);
            return;
        }
        let selected = self.selected_set();
        let show_group = self.group_filter == GroupFilter::All;
        let mono = egui::TextStyle::Monospace.resolve(ui.style());
        let accent = style::accent(ui);
        let weak = ui.visuals().weak_text_color();

        let mut table = TableBuilder::new(ui)
            .id_salt("combo_table")
            .striped(true)
            .resizable(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::exact(28.0))
            .column(Column::initial(180.0).at_least(80.0).clip(true))
            .column(Column::initial(110.0).at_least(60.0).clip(true));
        table = table.column(Column::remainder().at_least(120.0).clip(true));
        if show_group {
            table = table.column(Column::initial(110.0).at_least(60.0).clip(true));
        }
        table = table.column(Column::initial(115.0).at_least(80.0));

        let mut header_click: Option<SortColumn> = None;
        let sort = self.sort;
        let header_label = |ui: &mut egui::Ui, text: &str, col: SortColumn, header_click: &mut Option<SortColumn>| {
            let arrow = if sort.0 == col { if sort.1 { " ⏶" } else { " ⏷" } } else { "" };
            if ui.add(egui::Label::new(RichText::new(format!("{text}{arrow}")).strong()).sense(Sense::click())).clicked() {
                *header_click = Some(col);
            }
        };
        table
            .header(26.0, |mut header| {
                header.col(|ui| header_label(ui, "✔", SortColumn::Enabled, &mut header_click));
                header.col(|ui| header_label(ui, tr("label-name"), SortColumn::Name, &mut header_click));
                header.col(|ui| header_label(ui, tr("label-keyword"), SortColumn::Keyword, &mut header_click));
                header.col(|ui| header_label(ui, tr("label-snippet"), SortColumn::Snippet, &mut header_click));
                if show_group {
                    header.col(|ui| header_label(ui, tr("label-group"), SortColumn::Group, &mut header_click));
                }
                header.col(|ui| header_label(ui, tr("column-last-used"), SortColumn::LastUsed, &mut header_click));
            })
            .body(|body| {
                body.rows(26.0, rows.len(), |mut row| {
                    let r = &rows[row.index()];
                    row.set_selected(selected.contains(&r.uuid));
                    let dim = |t: RichText| if r.usable { t } else { t.color(weak).italics() };
                    row.col(|ui| {
                        let mut enabled = r.enabled;
                        if ui.checkbox(&mut enabled, "").changed() {
                            actions.push(Action::SetEnabled(vec![r.uuid], enabled));
                        }
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new(dim(RichText::new(&r.name))).truncate().selectable(false));
                    });
                    row.col(|ui| {
                        let t = if r.keyword.is_empty() {
                            RichText::new("—").color(weak)
                        } else {
                            RichText::new(&r.keyword).font(mono.clone()).color(accent)
                        };
                        ui.add(egui::Label::new(dim(t)).truncate().selectable(false));
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new(dim(RichText::new(&r.snippet))).truncate().selectable(false));
                    });
                    if show_group {
                        row.col(|ui| {
                            ui.add(egui::Label::new(dim(RichText::new(&r.group))).truncate().selectable(false));
                        });
                    }
                    row.col(|ui| {
                        let t = r.last_used.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "—".into());
                        ui.label(RichText::new(t).color(weak).small());
                    });
                    let resp = row.response();
                    if resp.clicked() {
                        actions.push(Action::Click(r.uuid, resp.ctx.input(|i| i.modifiers)));
                    }
                    if resp.double_clicked() {
                        actions.push(Action::EditCombo(r.uuid));
                    }
                    if resp.secondary_clicked() && !selected.contains(&r.uuid) {
                        actions.push(Action::Click(r.uuid, egui::Modifiers::NONE));
                    }
                    resp.context_menu(|ui| self.combo_menu(ui, actions, false));
                });
            });
        if let Some(col) = header_click {
            self.sort = if self.sort.0 == col { (col, !self.sort.1) } else { (col, true) };
        }
        // keep the order of the visible rows for range selection
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::unique("visible_rows"), rows.iter().map(|r| r.uuid).collect::<Vec<_>>()));
    }

    fn empty_state(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.25);
            if self.search.is_empty() {
                ui.label(RichText::new(tr("combos-empty-title")).heading());
                ui.add_space(6.0);
                ui.label(muted(ui, tr("combos-empty-text")));
                ui.add_space(12.0);
                if ui.button(format!("+ {}", tr("combos-create-first"))).clicked() {
                    actions.push(Action::NewCombo);
                }
                if ui.button(tr("menu-import")).clicked() {
                    actions.push(Action::Import);
                }
                if ui.button(tr("menu-import-espanso")).clicked() {
                    actions.push(Action::ImportEspanso);
                }
            } else {
                ui.label(muted(ui, tr("combos-no-match")));
            }
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.horizontal(|ui| {
            let enabled = self.shared.is_enabled();
            let (dot, text) = if enabled {
                (RichText::new("●").color(style::accent(ui)), tr("status-active"))
            } else {
                (RichText::new("●").color(style::WARNING), tr("status-paused"))
            };
            ui.label(dot);
            if ui.add(egui::Label::new(text).sense(Sense::click())).on_hover_text(tr("status-toggle-hint")).clicked() {
                actions.push(Action::ToggleEnabled);
            }
            ui.separator();
            let (n, groups) = {
                let lib = self.shared.library.read();
                (lib.combos.len(), lib.groups.len())
            };
            ui.label(muted(ui, tr!("app-status-counts", combos = n, groups = groups)));
            if let Some(m) = &self.platform_status.message {
                ui.separator();
                ui.label(RichText::new(format!("⚠ {m}")).color(style::WARNING).small());
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if self.shared.paths.portable {
                    ui.label(muted(ui, tr("app-portable-edition")));
                }
            });
        });
    }

    // ---- actions ----------------------------------------------------------------------------------------------------

    fn apply(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::Click(uuid, mods) => self.click_row(ctx, uuid, mods),
            Action::SelectAll => {
                self.selection = ctx.data(|d| d.get_temp::<Vec<Uuid>>(egui::Id::unique("visible_rows"))).unwrap_or_default();
            }
            Action::NewCombo => {
                let group = self.default_group_for_new_combo();
                self.dialog = Some(Dialog::Combo(ComboEditor::new_combo(group)));
            }
            Action::EditCombo(uuid) => {
                if let Some(c) = self.shared.library.read().combo(&uuid).cloned() {
                    self.dialog = Some(Dialog::Combo(ComboEditor::edit(c)));
                }
            }
            Action::Duplicate => {
                let mut new_ids = Vec::new();
                {
                    let mut lib = self.shared.library.write();
                    let dups: Vec<_> = self.selection.iter().filter_map(|u| lib.combo(u)).map(|c| c.duplicate()).collect();
                    for d in dups {
                        new_ids.push(d.uuid);
                        lib.combos.push(d);
                    }
                }
                self.selection = new_ids;
                self.commit_library();
                if self.selection.len() == 1 {
                    let uuid = self.selection[0];
                    self.apply(ctx, Action::EditCombo(uuid));
                }
            }
            Action::DeleteSelected => {
                let n = self.selection.len();
                let text = tr!("confirm-delete-combos", count = n);
                self.dialog = Some(Dialog::Confirm(Confirm::new(text, ConfirmAction::DeleteCombos(self.selection.clone()))));
            }
            Action::SetEnabled(ids, enabled) => {
                {
                    let mut lib = self.shared.library.write();
                    for c in lib.combos.iter_mut().filter(|c| ids.contains(&c.uuid)) {
                        c.enabled = enabled;
                    }
                }
                self.commit_library();
            }
            Action::SetMatching(m) => {
                self.modify_selected(|c| {
                    c.matching_mode = m;
                    c.touch();
                });
            }
            Action::SetCase(cs) => {
                self.modify_selected(|c| {
                    c.case_sensitivity = cs;
                    c.touch();
                });
            }
            Action::MoveToGroup(g) => {
                self.modify_selected(|c| {
                    c.group = Some(g);
                    c.touch();
                });
            }
            Action::CopySnippet(uuid) => {
                if let Some(c) = self.shared.library.read().combo(&uuid) {
                    ctx.copy_text(c.snippet.clone());
                }
            }
            Action::NewGroup => self.dialog = Some(Dialog::Group(GroupEditor::new_group())),
            Action::EditGroup(uuid) => {
                if let Some(g) = self.shared.library.read().group(&uuid).cloned() {
                    self.dialog = Some(Dialog::Group(GroupEditor::edit(g)));
                }
            }
            Action::DeleteGroup(uuid) => {
                let (groups, count, name) = {
                    let lib = self.shared.library.read();
                    (
                        lib.groups.len(),
                        lib.combos.iter().filter(|c| c.group == Some(uuid)).count(),
                        lib.group(&uuid).map(|g| g.name.clone()).unwrap_or_default(),
                    )
                };
                if groups <= 1 {
                    self.info(tr("msg-last-group"));
                } else {
                    let text = if count == 0 {
                        tr!("confirm-delete-group", name = name)
                    } else {
                        tr!("confirm-delete-group-with-combos", name = name, count = count)
                    };
                    self.dialog = Some(Dialog::Confirm(Confirm::new(text, ConfirmAction::DeleteGroup(uuid))));
                }
            }
            Action::ToggleGroup(uuid) => {
                if let Some(g) = self.shared.library.write().group_mut(&uuid) {
                    g.enabled = !g.enabled;
                }
                self.commit_library();
            }
            Action::MoveGroup(uuid, delta) => {
                {
                    let mut lib = self.shared.library.write();
                    if let Some(i) = lib.groups.iter().position(|g| g.uuid == uuid) {
                        let j = i as isize + delta;
                        if j >= 0 && (j as usize) < lib.groups.len() {
                            lib.groups.swap(i, j as usize);
                        }
                    }
                }
                self.commit_library();
            }
            Action::Import => self.import_file(),
            Action::ImportEspanso => self.dialog = Some(Dialog::Espanso(EspansoDialog::new())),
            Action::Export(selected_only) => self.export(selected_only),
            Action::Backup => self.manual_backup(),
            Action::Restore => self.restore_from_file(),
            Action::RestoreAuto => self.dialog = Some(Dialog::RestoreBackup(RestoreBackupDialog::new(&self.shared.backup_dir()))),
            Action::OpenDataFolder => crate::platform::open_path(&self.shared.paths.data_dir),
            Action::OpenLog => {
                if let Some(p) = crate::logger::log_path() {
                    crate::platform::open_path(&p);
                }
            }
            Action::Preferences => self.prefs = Some(PrefsWindow::default()),
            Action::About => self.dialog = Some(Dialog::About),
            Action::Help => self.dialog = Some(Dialog::Help),
            Action::EmojiList => self.dialog = Some(Dialog::EmojiList(Default::default())),
            Action::Hide => self.hide_main(ctx),
            Action::Quit => self.quit(ctx),
            Action::ToggleEnabled => self.shared.set_enabled(!self.shared.is_enabled()),
        }
    }

    fn click_row(&mut self, ctx: &egui::Context, uuid: Uuid, mods: egui::Modifiers) {
        if mods.shift
            && let Some(anchor) = self.anchor
        {
            let visible = ctx.data(|d| d.get_temp::<Vec<Uuid>>(egui::Id::unique("visible_rows"))).unwrap_or_default();
            if let (Some(a), Some(b)) = (visible.iter().position(|u| *u == anchor), visible.iter().position(|u| *u == uuid)) {
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                self.selection = visible[lo..=hi].to_vec();
                return;
            }
        }
        if mods.command {
            if let Some(i) = self.selection.iter().position(|u| *u == uuid) {
                self.selection.remove(i);
            } else {
                self.selection.push(uuid);
            }
        } else {
            self.selection = vec![uuid];
        }
        self.anchor = Some(uuid);
    }

    fn modify_selected(&mut self, f: impl Fn(&mut crate::library::Combo)) {
        {
            let mut lib = self.shared.library.write();
            for c in lib.combos.iter_mut().filter(|c| self.selection.contains(&c.uuid)) {
                f(c);
            }
        }
        self.commit_library();
    }

    // ---- files ------------------------------------------------------------------------------------------------------

    pub(super) fn file_dialog(&self) -> rfd::FileDialog {
        let d = rfd::FileDialog::new();
        match self.last_dir() {
            Some(dir) => d.set_directory(dir),
            None => d,
        }
    }

    fn import_file(&mut self) {
        let Some(path) = self
            .file_dialog()
            .add_filter(tr("filter-combo-files"), &["json", "csv"])
            .add_filter(tr("filter-all-files"), &["*"])
            .pick_file()
        else {
            return;
        };
        self.remember_dir(&path);
        let target = self.default_group_for_new_combo();
        match ImportDialog::open(&self.shared, &path, target) {
            Ok(d) => self.dialog = Some(Dialog::Import(d)),
            Err(e) => self.error(format!("{}\n{e:#}", tr("msg-import-failed"))),
        }
    }

    fn export(&mut self, selected_only: bool) {
        let Some(path) = self
            .file_dialog()
            .set_file_name("combos.json")
            .add_filter("JSON", &["json"])
            .add_filter(tr("filter-csv-textexpander"), &["csv"])
            .save_file()
        else {
            return;
        };
        self.remember_dir(&path);
        let lib = self.shared.library.read().clone();
        let combos: Vec<_> = lib.combos.iter().filter(|c| !selected_only || self.selection.contains(&c.uuid)).cloned().collect();
        let is_csv = path.extension().map(|e| e.eq_ignore_ascii_case("csv")).unwrap_or(false);
        let result =
            if is_csv { Library::export_csv(combos.iter(), &path) } else { Library { combos, groups: Vec::new() }.save(&path, false) };
        match result {
            Ok(()) => self.info(tr("msg-exported")),
            Err(e) => self.error(format!("{}\n{e:#}", tr("msg-export-failed"))),
        }
    }

    fn manual_backup(&mut self) {
        let name = format!("{}.json", tr!("file-name-backup", date = chrono::Local::now().format("%Y-%m-%d").to_string()));
        let Some(path) = self.file_dialog().set_file_name(name).add_filter("JSON", &["json"]).save_file() else { return };
        self.remember_dir(&path);
        let r = self.shared.library.read().save(&path, true);
        match r {
            Ok(()) => self.info(tr("msg-backup-saved")),
            Err(e) => self.error(format!("{e:#}")),
        }
    }

    fn restore_from_file(&mut self) {
        let Some(path) = self.file_dialog().add_filter("JSON", &["json"]).pick_file() else { return };
        self.remember_dir(&path);
        self.dialog = Some(Dialog::Confirm(Confirm::new(tr("confirm-replace-library").to_string(), ConfirmAction::RestoreFrom(path))));
    }
}

pub fn matching_labels(prefs: &crate::prefs::Preferences) -> Vec<(MatchingMode, String)> {
    let default = match MatchingMode::Default.resolve(prefs) {
        MatchingMode::Loose => tr("matching-loose"),
        _ => tr("matching-strict"),
    };
    vec![
        (MatchingMode::Default, tr!("app-default-with-value", value = default)),
        (MatchingMode::Strict, tr("matching-strict").to_string()),
        (MatchingMode::Loose, tr("matching-loose").to_string()),
    ]
}

pub fn case_labels(prefs: &crate::prefs::Preferences) -> Vec<(CaseSensitivity, String)> {
    let default = match CaseSensitivity::Default.resolve(prefs) {
        CaseSensitivity::CaseInsensitive => tr("case-insensitive"),
        _ => tr("case-sensitive"),
    };
    vec![
        (CaseSensitivity::Default, tr!("app-default-with-value", value = default)),
        (CaseSensitivity::CaseSensitive, tr("case-sensitive").to_string()),
        (CaseSensitivity::CaseInsensitive, tr("case-insensitive").to_string()),
    ]
}
