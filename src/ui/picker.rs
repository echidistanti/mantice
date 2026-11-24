//! The combo picker (a search popup opened with a global shortcut) and the `#{input:}` dialog.
//! Both are separate OS windows, shown even when the main window is hidden in the tray.

use std::sync::Arc;
use std::time::Instant;

use chrono::NaiveDateTime;
use crossbeam_channel::Sender;
use egui::{Key, Modifiers, RichText, ViewportBuilder, ViewportCommand, ViewportId};
use parking_lot::Mutex;
use uuid::Uuid;

use super::style::{self, muted, preview};
use super::{ExpanderApp, centered_position};
use crate::app_state::AppShared;
use crate::engine::EngineMsg;
use crate::i18n::tr;

const MAX_RESULTS: usize = 200;

#[derive(Clone)]
enum PickerKind {
    Combo(Uuid),
    Emoji { shortcode: String, emoji: String },
}

struct PickerEntry {
    kind: PickerKind,
    title: String,
    /// The title in lowercase, to sort without converting it at each comparison.
    sort_title: String,
    keyword: String,
    preview: String,
    haystack: String,
    last_use: Option<NaiveDateTime>,
}

pub struct PickerState {
    query: String,
    selected: usize,
    combos: Vec<PickerEntry>,
    emojis: Vec<PickerEntry>,
    frames: u32,
    was_focused: bool,
    opened: Instant,
}

impl PickerState {
    pub fn new(shared: &AppShared) -> Self {
        let lib = shared.library.read();
        let combos = lib
            .combos
            .iter()
            .filter(|c| lib.is_usable(c))
            .map(|c| PickerEntry {
                kind: PickerKind::Combo(c.uuid),
                title: c.display_name(),
                sort_title: c.display_name().to_lowercase(),
                keyword: c.keyword.clone(),
                preview: preview(&c.snippet, 110),
                haystack: format!("{}\n{}\n{}\n{}\n{}", c.name, c.keyword, c.snippet, c.description, lib.group_name(&c.group))
                    .to_lowercase(),
                last_use: c.last_use,
            })
            .collect();
        let emojis = if shared.prefs.read().show_emojis_in_picker {
            let last = shared.emoji_last_use.lock().clone();
            crate::emoji::all()
                .iter()
                .map(|e| {
                    let title = format!("{}  {}", e.emoji, e.name);
                    PickerEntry {
                        kind: PickerKind::Emoji { shortcode: e.shortcode().to_string(), emoji: e.emoji.to_string() },
                        sort_title: title.to_lowercase(),
                        title,
                        keyword: format!(":{}:", e.shortcode()),
                        preview: String::new(),
                        haystack: format!("{} {}", e.shortcode(), e.name.to_lowercase()),
                        last_use: last.get(e.shortcode()).copied(),
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        Self { query: String::new(), selected: 0, combos, emojis, frames: 0, was_focused: false, opened: Instant::now() }
    }
}

/// The entries matching the query, most recently used first. A function of the fields (not a method), so that
/// `selected` can still be changed while the results are borrowed.
fn matching<'a>(query: &str, combos: &'a [PickerEntry], emojis: &'a [PickerEntry]) -> Vec<&'a PickerEntry> {
    let words: Vec<String> = query.to_lowercase().split_whitespace().map(str::to_string).collect();
    let matches = |e: &&PickerEntry| words.iter().all(|w| e.haystack.contains(w.as_str()));
    let by_use = |a: &&PickerEntry, b: &&PickerEntry| b.last_use.cmp(&a.last_use).then_with(|| a.sort_title.cmp(&b.sort_title));
    let mut found: Vec<&PickerEntry> = combos.iter().filter(matches).collect();
    found.sort_by(by_use);
    if query.trim().chars().count() >= 3 {
        let mut emojis: Vec<&PickerEntry> = emojis.iter().filter(matches).collect();
        emojis.sort_by(by_use);
        found.extend(emojis);
    }
    found.truncate(MAX_RESULTS);
    found
}

pub struct InputPrompt {
    description: String,
    value: String,
    pub reply: Sender<Option<String>>,
    frames: u32,
}

impl InputPrompt {
    pub fn new(description: String, reply: Sender<Option<String>>) -> Self {
        Self { description, value: String::new(), reply, frames: 0 }
    }
}

/// The popups are "deferred" viewports: they are drawn by their own callbacks, so their state is shared.
#[derive(Default)]
pub struct Popups {
    pub picker: Option<PickerState>,
    pub input: Option<InputPrompt>,
    /// After an answer, the input window stays open a moment: if the snippet asks for another value,
    /// the same window is reused and keeps the keyboard focus.
    pub input_linger_until: Option<Instant>,
}

impl Popups {
    pub fn input_window_needed(&mut self) -> bool {
        if self.input.is_some() {
            return true;
        }
        match self.input_linger_until {
            Some(t) if Instant::now() < t => true,
            Some(_) => {
                self.input_linger_until = None;
                false
            }
            None => false,
        }
    }
}

pub type SharedPopups = Arc<Mutex<Popups>>;

const PICKER_ID: &str = "mantice_picker";
const INPUT_ID: &str = "mantice_input";

impl ExpanderApp {
    pub(super) fn picker_viewport(&mut self, ctx: &egui::Context) {
        let size = egui::vec2(580.0, 420.0);
        let mut builder = ViewportBuilder::default()
            .with_title(tr("picker-title"))
            .with_inner_size(size)
            .with_decorations(false)
            .with_resizable(false)
            .with_always_on_top()
            .with_taskbar(false)
            .with_active(true);
        if let Some(pos) = centered_position(ctx, size) {
            builder = builder.with_position(pos);
        }
        let popups = self.popups.clone();
        let shared = self.shared.clone();
        ctx.show_viewport_deferred(ViewportId::from_hash_of(PICKER_ID), builder, move |ui, _class| {
            let mut guard = popups.lock();
            let Some(state) = guard.picker.as_mut() else {
                ui.ctx().send_viewport_cmd(ViewportCommand::Visible(false));
                ui.ctx().request_repaint_of(ViewportId::ROOT);
                return;
            };
            if let Some(choice) = picker_ui(ui, state) {
                guard.picker = None;
                drop(guard);
                ui.ctx().send_viewport_cmd(ViewportCommand::Visible(false));
                let msg = match choice {
                    Some(PickerKind::Combo(uuid)) => EngineMsg::PickerCombo(uuid),
                    Some(PickerKind::Emoji { shortcode, emoji }) => EngineMsg::PickerEmoji { shortcode, emoji },
                    None => EngineMsg::PickerCancelled,
                };
                let _ = shared.engine_tx.send(msg);
                ui.ctx().request_repaint_of(ViewportId::ROOT);
            }
        });
    }

    pub(super) fn input_viewport(&mut self, ctx: &egui::Context) {
        let size = super::title_bar::outer_size(egui::vec2(440.0, 170.0));
        let mut builder = ViewportBuilder::default()
            .with_title("Mantice")
            .with_decorations(!super::title_bar::CUSTOM)
            .with_inner_size(size)
            .with_resizable(false)
            .with_always_on_top()
            .with_minimize_button(false)
            .with_maximize_button(false)
            .with_active(true);
        if let Some(pos) = centered_position(ctx, size) {
            builder = builder.with_position(pos);
        }
        let popups = self.popups.clone();
        ctx.show_viewport_deferred(ViewportId::from_hash_of(INPUT_ID), builder, move |ui, _class| {
            super::title_bar::show(ui, "Mantice", super::title_bar::Buttons::Close, false);
            let mut guard = popups.lock();
            let Some(prompt) = guard.input.as_mut() else {
                if guard.input_window_needed() {
                    // waiting for a possible next prompt
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.centered_and_justified(|ui| ui.spinner());
                    });
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));
                } else {
                    // hide the window right away, without waiting for the main viewport to close it
                    ui.ctx().send_viewport_cmd(ViewportCommand::Visible(false));
                }
                ui.ctx().request_repaint_of(ViewportId::ROOT);
                return;
            };
            if let Some(value) = input_ui(ui, prompt)
                && let Some(prompt) = guard.input.take()
            {
                let _ = prompt.reply.send(value);
                guard.input_linger_until = Some(Instant::now() + std::time::Duration::from_millis(450));
                ui.ctx().request_repaint_of(ViewportId::ROOT);
            }
        });
    }
}

/// Draw the picker. Returns Some(choice) when the picker must be closed (None = cancelled).
fn picker_ui(ui: &mut egui::Ui, state: &mut PickerState) -> Option<Option<PickerKind>> {
    let mut outcome: Option<Option<PickerKind>> = None;
    state.frames += 1;
    if state.frames <= 3 {
        ui.ctx().send_viewport_cmd(ViewportCommand::Visible(true));
        ui.ctx().send_viewport_cmd(ViewportCommand::Focus);
        ui.ctx().request_repaint();
    }
    let focused = ui.input(|i| i.viewport().focused).unwrap_or(false);
    if focused {
        state.was_focused = true;
    } else if state.was_focused && state.opened.elapsed().as_millis() > 300 {
        log::debug!("Combo picker closed: focus lost");
        outcome = Some(None); // clicked elsewhere
    }
    if ui.input(|i| i.viewport().close_requested()) {
        outcome = Some(None);
    }

    let (down, up, enter, escape) = ui.input_mut(|i| {
        (
            i.consume_key(Modifiers::NONE, Key::ArrowDown),
            i.consume_key(Modifiers::NONE, Key::ArrowUp),
            i.consume_key(Modifiers::NONE, Key::Enter),
            i.consume_key(Modifiers::NONE, Key::Escape),
        )
    });
    if escape {
        outcome = Some(None);
    }

    let frame =
        egui::Frame::central_panel(ui.style()).inner_margin(12.0).stroke(egui::Stroke::new(1.0, style::accent(ui).linear_multiply(0.6)));
    egui::CentralPanel::default().frame(frame).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(18.0));
            let r = ui.add(
                egui::TextEdit::singleline(&mut state.query)
                    .hint_text(tr("picker-search"))
                    .font(egui::TextStyle::Heading)
                    .frame(egui::Frame::NONE)
                    .desired_width(f32::INFINITY),
            );
            if state.frames <= 3 || !r.has_focus() {
                r.request_focus();
            }
            if r.changed() {
                state.selected = 0;
            }
        });
        ui.separator();
        let results = matching(&state.query, &state.combos, &state.emojis);
        if down && !results.is_empty() {
            state.selected = (state.selected + 1).min(results.len() - 1);
        }
        if up {
            state.selected = state.selected.saturating_sub(1);
        }
        let current = state.selected;
        let mut hovered: Option<usize> = None;
        if results.is_empty() {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| ui.label(muted(ui, tr("picker-no-match"))));
        }
        let accent = style::accent(ui);
        egui::ScrollArea::vertical().auto_shrink([false, false]).max_height(ui.available_height() - 22.0).show(ui, |ui| {
            for (i, e) in results.iter().enumerate() {
                let selected = i == current;
                let resp = egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(8, 5))
                    .corner_radius(6)
                    .fill(if selected { ui.visuals().selection.bg_fill } else { egui::Color32::TRANSPARENT })
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&e.title).strong());
                            if !e.keyword.is_empty() {
                                ui.label(RichText::new(&e.keyword).monospace().color(accent));
                            }
                        });
                        if !e.preview.is_empty() {
                            ui.add(egui::Label::new(muted(ui, &e.preview).small()).truncate());
                        }
                    })
                    .response
                    .interact(egui::Sense::click());
                if selected && (down || up) {
                    resp.scroll_to_me(None);
                }
                if resp.hovered() && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO) {
                    hovered = Some(i);
                }
                if resp.clicked() {
                    outcome = Some(Some(e.kind.clone()));
                }
            }
        });
        if enter && let Some(e) = results.get(current) {
            outcome = Some(Some(e.kind.clone()));
        }
        if let Some(i) = hovered {
            state.selected = i;
        }
        ui.label(muted(ui, tr("picker-help")).small());
    });
    outcome
}

/// Draw the input dialog. Returns Some(value) when the dialog must be closed (None = cancelled).
fn input_ui(ui: &mut egui::Ui, prompt: &mut InputPrompt) -> Option<Option<String>> {
    let mut result: Option<Option<String>> = None;
    prompt.frames += 1;
    if prompt.frames <= 3 {
        ui.ctx().send_viewport_cmd(ViewportCommand::Visible(true));
        ui.ctx().send_viewport_cmd(ViewportCommand::Focus);
        ui.ctx().request_repaint();
    }
    if ui.input(|i| i.viewport().close_requested()) {
        result = Some(None);
    }
    let (enter, escape) = ui.input_mut(|i| (i.consume_key(Modifiers::NONE, Key::Enter), i.consume_key(Modifiers::NONE, Key::Escape)));
    egui::CentralPanel::default().frame(egui::Frame::central_panel(ui.style()).inner_margin(14.0)).show(ui, |ui| {
        ui.label(RichText::new(&prompt.description).strong());
        ui.add_space(6.0);
        let r = ui.add(egui::TextEdit::singleline(&mut prompt.value).desired_width(f32::INFINITY));
        if prompt.frames <= 3 || !r.has_focus() {
            r.request_focus();
        }
        ui.add_space(10.0);
        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 30.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let b = egui::Button::new(RichText::new(tr("app-ok")).color(egui::Color32::WHITE))
                .fill(style::accent(ui))
                .min_size(egui::vec2(90.0, 0.0));
            if ui.add(b).clicked() || enter {
                result = Some(Some(prompt.value.clone()));
            }
            if ui.add(egui::Button::new(tr("app-cancel")).min_size(egui::vec2(90.0, 0.0))).clicked() || escape {
                result = Some(None);
            }
        });
    });
    result
}
