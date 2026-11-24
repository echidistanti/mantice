//! The dialog that imports an espanso package: where it comes from, what is in it, and the group it goes to.

use std::sync::mpsc::{Receiver, TryRecvError, channel};

use egui::RichText;
use uuid::Uuid;

use super::dialogs::{buttons_row, ok_cancel};
use super::style::{self, muted};
use super::{ExpanderApp, GroupFilter};
use crate::app_state::AppShared;
use crate::espanso::{Package, Reason};
use crate::i18n::tr;
use crate::library::{self, ConflictPolicy, Group, ImportAnalysis};

/// A package with this many combos (or more) makes the list of combos very long: the dialog says so.
const LARGE_PACKAGE: usize = 5_000;
/// The triggers shown as examples of what was left out.
const EXAMPLES: usize = 4;

pub struct EspansoDialog {
    input: String,
    error: Option<String>,
    stage: Stage,
    policy: ConflictPolicy,
    /// A new group (named `group_name`) or an existing one (`group`).
    new_group: bool,
    group_name: String,
    group: Option<Uuid>,
}

enum Stage {
    /// Asking where the package is.
    Input,
    /// A thread is getting it.
    Loading(Receiver<anyhow::Result<Package>>),
    Preview(Box<Preview>),
}

struct Preview {
    package: Package,
    analysis: ImportAnalysis,
    /// What was left out: the reason, how many, and a few triggers.
    left_out: Vec<(Reason, usize, String)>,
}

impl EspansoDialog {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            error: None,
            stage: Stage::Input,
            policy: ConflictPolicy::Skip,
            new_group: true,
            group_name: String::new(),
            group: None,
        }
    }

    fn start(&mut self, ctx: &egui::Context) {
        let (sender, receiver) = channel();
        let input = self.input.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            sender.send(crate::espanso_source::load(&input)).ok();
            ctx.request_repaint();
        });
        self.error = None;
        self.stage = Stage::Loading(receiver);
    }

    /// Take the package from the thread, if it has arrived.
    fn poll(&mut self, shared: &AppShared) {
        let Stage::Loading(receiver) = &self.stage else { return };
        let package = match receiver.try_recv() {
            Ok(Ok(package)) => package,
            Ok(Err(e)) => {
                self.error = Some(format!("{e:#}"));
                self.stage = Stage::Input;
                return;
            }
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                self.error = Some(tr("error-espanso-interrupted").to_string());
                self.stage = Stage::Input;
                return;
            }
        };
        let mut package = package;
        let left_out = Reason::ALL
            .into_iter()
            .filter_map(|reason| {
                let items: Vec<&str> = package.skipped.iter().filter(|s| s.reason == reason).map(|s| s.what.as_str()).collect();
                let examples: Vec<String> = items.iter().take(EXAMPLES).map(|w| shorten(w)).collect();
                (!items.is_empty()).then(|| (reason, items.len(), examples.join(", ")))
            })
            .collect();
        let library = shared.library.read();
        // the group of a package imported before takes the new combos too
        let same_name = library.groups.iter().find(|g| g.name.trim().eq_ignore_ascii_case(package.title.trim())).map(|g| g.uuid);
        self.new_group = same_name.is_none();
        self.group = same_name.or_else(|| library.groups.first().map(|g| g.uuid));
        self.group_name = package.title.clone();
        let analysis = library::analyze_import(&library, library::combos_from_espanso(&mut package));
        self.policy = ConflictPolicy::Skip;
        self.stage = Stage::Preview(Box::new(Preview { package, analysis, left_out }));
    }
}

/// A trigger short enough to be listed.
fn shorten(text: &str) -> String {
    let text = text.replace(['\n', '\t'], " ");
    match text.char_indices().nth(24) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text,
    }
}

fn reason_label(reason: Reason) -> &'static str {
    match reason {
        Reason::Regex => tr("espanso-reason-regex"),
        Reason::NoTrigger => tr("espanso-reason-no-trigger"),
        Reason::Form => tr("espanso-reason-form"),
        Reason::Image => tr("espanso-reason-image"),
        Reason::RichText => tr("espanso-reason-rich-text"),
        Reason::Command => tr("espanso-reason-command"),
        Reason::Choice => tr("espanso-reason-choice"),
        Reason::Variable => tr("espanso-reason-variable"),
        Reason::DateFormat => tr("espanso-reason-date-format"),
        Reason::Nested => tr("espanso-reason-nested"),
        Reason::NoText => tr("espanso-reason-no-text"),
        Reason::Control => tr("espanso-reason-control"),
    }
}

#[derive(Clone, Copy)]
enum Browse {
    File,
    Folder,
}

impl ExpanderApp {
    /// Returns whether the dialog stays open.
    pub(super) fn espanso_ui(&mut self, ctx: &egui::Context, d: &mut EspansoDialog) -> bool {
        d.poll(&self.shared);
        let groups: Vec<(Uuid, String)> = self.shared.library.read().groups.iter().map(|g| (g.uuid, g.name.clone())).collect();
        let mut browse = None;
        let mut submit = false;
        let EspansoDialog { input, error, stage, policy, new_group, group_name, group } = d;

        let answer = ok_cancel(ctx, "espanso", |ui| {
            ui.set_width(520.0);
            ui.heading(tr("espanso-title"));
            ui.add_space(6.0);
            match stage {
                Stage::Input | Stage::Loading(_) => {
                    let loading = matches!(stage, Stage::Loading(_));
                    ui.label(muted(ui, tr("espanso-help")));
                    ui.add_space(6.0);
                    ui.add_enabled_ui(!loading, |ui| {
                        let field = egui::TextEdit::singleline(input).hint_text(tr("espanso-source-hint")).desired_width(f32::INFINITY);
                        let response = ui.add(field);
                        if !response.has_focus() && input.is_empty() {
                            response.request_focus();
                        }
                        submit = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        ui.horizontal(|ui| {
                            if ui.button(tr("espanso-choose-file")).clicked() {
                                browse = Some(Browse::File);
                            }
                            if ui.button(tr("espanso-choose-folder")).clicked() {
                                browse = Some(Browse::Folder);
                            }
                        });
                    });
                    if let Some(message) = error.as_deref() {
                        ui.add_space(6.0);
                        ui.label(RichText::new(message).color(style::DANGER));
                    }
                    if loading {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(tr("espanso-loading"));
                        });
                    }
                    buttons_row(ui, tr("espanso-load"), !loading && !input.trim().is_empty())
                }
                Stage::Preview(preview) => {
                    let Preview { package, analysis, left_out } = &**preview;
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&package.title).strong());
                        if !package.version.is_empty() {
                            ui.label(muted(ui, &package.version));
                        }
                        if !package.author.is_empty() {
                            ui.label(muted(ui, tr!("espanso-by-author", author = package.author.as_str())));
                        }
                    });
                    if !package.description.is_empty() {
                        ui.label(muted(ui, &package.description));
                    }
                    ui.add_space(8.0);
                    ui.label(tr!("import-new-combos", count = analysis.new_combos.len()));
                    if analysis.conflict_count() > 0 {
                        ui.add_space(6.0);
                        ui.label(RichText::new(tr!("import-conflicts", count = analysis.conflict_count())).color(style::WARNING));
                        ui.radio_value(policy, ConflictPolicy::Skip, tr("import-skip"));
                        ui.radio_value(policy, ConflictPolicy::Overwrite, tr("import-overwrite"));
                        if !analysis.conflicting_newer.is_empty() {
                            ui.radio_value(
                                policy,
                                ConflictPolicy::OverwriteOlder,
                                tr!("import-overwrite-older", count = analysis.conflicting_newer.len()),
                            );
                        }
                    }
                    let count = analysis.importable_count(*policy);
                    if analysis.new_combos.len() + analysis.conflict_count() >= LARGE_PACKAGE {
                        ui.add_space(6.0);
                        ui.label(RichText::new(tr("espanso-large")).color(style::WARNING));
                    }
                    if package.approximated > 0 {
                        ui.add_space(6.0);
                        ui.label(muted(ui, tr!("espanso-approximated", count = package.approximated)));
                    }
                    if package.duplicates > 0 {
                        ui.label(muted(ui, tr!("espanso-duplicates", count = package.duplicates)));
                    }
                    if !package.unreadable.is_empty() {
                        ui.label(RichText::new(tr!("espanso-unreadable", files = package.unreadable.join(", "))).color(style::WARNING));
                    }
                    if !left_out.is_empty() {
                        ui.add_space(6.0);
                        egui::CollapsingHeader::new(tr!("espanso-left-out", count = package.skipped.len())).show(ui, |ui| {
                            for (reason, count, examples) in left_out {
                                ui.label(format!("{} ({count})", reason_label(*reason)));
                                ui.label(muted(ui, examples).small());
                            }
                        });
                    }
                    ui.add_space(8.0);
                    ui.label(tr("import-into-group"));
                    ui.horizontal(|ui| {
                        ui.radio_value(new_group, true, tr("espanso-new-group"));
                        ui.add_enabled(*new_group, egui::TextEdit::singleline(group_name).desired_width(260.0));
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(new_group, false, tr("espanso-existing-group"));
                        ui.add_enabled_ui(!*new_group, |ui| {
                            let current = groups.iter().find(|(u, _)| Some(*u) == *group).map(|(_, n)| n.clone()).unwrap_or_default();
                            egui::ComboBox::from_id_salt("espanso_group").selected_text(current).width(260.0).show_ui(ui, |ui| {
                                for (u, n) in &groups {
                                    ui.selectable_value(group, Some(*u), n);
                                }
                            });
                        });
                    });
                    let group_ok = if *new_group { !group_name.trim().is_empty() } else { group.is_some() };
                    buttons_row(ui, &tr!("import-button", count = count), count > 0 && group_ok)
                }
            }
        });

        if let Some(kind) = browse {
            let picker = self.file_dialog();
            let picked = match kind {
                Browse::File => picker
                    .add_filter(tr("espanso-file-filter"), &["zip", "yml", "yaml"])
                    .add_filter(tr("filter-all-files"), &["*"])
                    .pick_file(),
                Browse::Folder => picker.pick_folder(),
            };
            if let Some(path) = picked {
                self.remember_dir(&path);
                d.input = path.display().to_string();
                d.start(ctx);
            }
            return true;
        }
        match (&d.stage, answer) {
            (_, Some(false)) => false,
            (Stage::Input, Some(true)) => {
                d.start(ctx);
                true
            }
            (Stage::Input, None) if submit && !d.input.trim().is_empty() => {
                d.start(ctx);
                true
            }
            (Stage::Preview(_), Some(true)) => {
                self.import_espanso(d);
                false
            }
            _ => true,
        }
    }

    fn import_espanso(&mut self, d: &mut EspansoDialog) {
        let Stage::Preview(preview) = std::mem::replace(&mut d.stage, Stage::Input) else { return };
        let Preview { package, analysis, .. } = *preview;
        let (count, group_id, group_name) = {
            let mut library = self.shared.library.write();
            let created = d.new_group;
            let id = match d.group.filter(|_| !created) {
                Some(existing) => existing,
                None => {
                    let group = Group::new(d.group_name.trim(), package.description.as_str());
                    let id = group.uuid;
                    library.groups.push(group);
                    id
                }
            };
            let count = library::apply_import(&mut library, analysis, d.policy, Some(id));
            if count == 0 && created {
                library.groups.retain(|g| g.uuid != id);
            }
            let name = library.groups.iter().find(|g| g.uuid == id).map(|g| g.name.clone()).unwrap_or_default();
            (count, id, name)
        };
        self.commit_library();
        if count > 0 {
            self.group_filter = GroupFilter::Group(group_id);
        }
        self.info(tr!("msg-espanso-imported", count = count, group = group_name));
    }
}
