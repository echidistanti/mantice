//! State shared between the input monitor, the substitution engine and the user interface.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use chrono::NaiveDateTime;
use crossbeam_channel::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use uuid::Uuid;

use crate::engine::EngineMsg;
use crate::i18n::tr;
use crate::library::{self, Library};
use crate::paths::AppPaths;
use crate::platform::{InputEvent, InputSink};
use crate::prefs::Preferences;
use crate::process_list::{DEFAULT_SENSITIVE_APPS, ProcessList};
use crate::shortcut::Shortcut;

/// Messages sent to the user interface.
pub enum UiMsg {
    ShowMainWindow,
    ShowPicker,
    NewCombo,
    ShowPreferences,
    PromptInput { description: String, reply: Sender<Option<String>> },
    ShortcutRecorded(Shortcut),
    Quit,
}

/// Size and modification time of a file, to notice that someone else changed it.
type FileStamp = (SystemTime, u64);

fn file_stamp(path: &Path) -> Option<FileStamp> {
    let m = std::fs::metadata(path).ok()?;
    Some((m.modified().ok()?, m.len()))
}

/// Sends messages to the UI and wakes it up (the UI may be hidden in the system tray).
pub struct UiBridge {
    tx: Sender<UiMsg>,
    ctx: OnceLock<egui::Context>,
}

impl UiBridge {
    pub fn send(&self, msg: UiMsg) {
        let _ = self.tx.send(msg);
        self.wake();
    }

    pub fn wake(&self) {
        if let Some(ctx) = self.ctx.get() {
            ctx.request_repaint();
        }
    }

    pub fn set_context(&self, ctx: egui::Context) {
        let _ = self.ctx.set(ctx);
    }
}

pub struct AppShared {
    pub paths: AppPaths,
    pub library: RwLock<Library>,
    /// The combo list file as it was when last read or written. A file that differs (changed by another
    /// computer sharing the folder, or unreadable at startup) is never overwritten, see [`Self::save_library`].
    library_stamp: Mutex<Option<FileStamp>>,
    pub prefs: RwLock<Preferences>,
    pub excluded_apps: RwLock<ProcessList>,
    pub sensitive_apps: RwLock<ProcessList>,
    pub emoji_excluded_apps: RwLock<ProcessList>,
    pub emoji_last_use: Mutex<HashMap<String, NaiveDateTime>>,
    /// Errors that occurred while loading, displayed by the UI at startup.
    pub startup_errors: Mutex<Vec<String>>,

    // fast-path flags read by the input hook
    enabled: AtomicBool,
    space_trigger: AtomicBool,
    pub substituting: AtomicBool,
    pub recording_shortcut: AtomicBool,
    hook_shortcuts: RwLock<Vec<Shortcut>>,
    /// The text typed since the last breaker, kept by the input thread in the order of the keystrokes (the
    /// engine has its own copy, updated a little later), to decide at once whether a space completes a combo.
    typed: Mutex<String>,

    pub engine_tx: Sender<EngineMsg>,
    pub ui: UiBridge,
}

impl AppShared {
    pub fn new(paths: AppPaths, prefs: Preferences, engine_tx: Sender<EngineMsg>) -> (Self, Receiver<UiMsg>) {
        let (ui_tx, ui_rx) = crossbeam_channel::unbounded();
        let shared = Self {
            excluded_apps: RwLock::new(ProcessList::load(&paths.excluded_apps_file(), &[])),
            sensitive_apps: RwLock::new(ProcessList::load(&paths.sensitive_apps_file(), DEFAULT_SENSITIVE_APPS)),
            emoji_excluded_apps: RwLock::new(ProcessList::load(&paths.emoji_excluded_apps_file(), &[])),
            emoji_last_use: Mutex::new(load_emoji_last_use(&paths.emoji_last_use_file())),
            paths,
            library: RwLock::new(Library::default()),
            library_stamp: Mutex::new(None),
            prefs: RwLock::new(prefs),
            startup_errors: Mutex::new(Vec::new()),
            enabled: AtomicBool::new(true),
            space_trigger: AtomicBool::new(false),
            substituting: AtomicBool::new(false),
            recording_shortcut: AtomicBool::new(false),
            hook_shortcuts: RwLock::new(Vec::new()),
            typed: Mutex::new(String::new()),
            engine_tx,
            ui: UiBridge { tx: ui_tx, ctx: OnceLock::new() },
        };
        shared.refresh_hook_config();
        (shared, ui_rx)
    }

    /// Must be called after every change of the preferences.
    pub fn refresh_hook_config(&self) {
        let p = self.prefs.read();
        self.enabled.store(p.enabled, Ordering::Relaxed);
        self.space_trigger.store(p.triggers_on_space(), Ordering::Relaxed);
        let mut list = Vec::new();
        if p.enable_app_toggle_shortcut && p.app_toggle_shortcut.is_valid() {
            list.push(p.app_toggle_shortcut);
        }
        if p.enabled {
            if p.combo_picker_enabled && p.combo_picker_shortcut.is_valid() {
                list.push(p.combo_picker_shortcut);
            }
            if !p.use_automatic_substitution && p.combo_trigger_shortcut.is_valid() {
                list.push(p.combo_trigger_shortcut);
            }
        }
        *self.hook_shortcuts.write() = list;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.prefs.write().enabled = enabled;
        self.refresh_hook_config();
        self.save_prefs();
        let _ = self.engine_tx.send(EngineMsg::ResetBuffer);
        self.ui.wake();
    }

    pub fn begin_substitution(&self) {
        self.substituting.store(true, Ordering::SeqCst);
    }

    pub fn end_substitution(&self) {
        self.substituting.store(false, Ordering::SeqCst);
    }

    // ---- persistence ------------------------------------------------------------------------------------------------

    pub fn library_path(&self) -> PathBuf {
        self.prefs.read().combo_list_folder_path(&self.paths.data_dir).join(library::LIBRARY_FILE_NAME)
    }

    pub fn backup_dir(&self) -> PathBuf {
        let p = self.prefs.read();
        if p.use_custom_backup_location && !p.custom_backup_location.trim().is_empty() {
            PathBuf::from(&p.custom_backup_location)
        } else {
            self.paths.default_backup_dir()
        }
    }

    /// Load the library from disk (called at startup).
    pub fn load_library(&self) {
        let path = self.library_path();
        let mut lib = Library::default();
        let mut needs_save = false;
        let mut readable = true;
        if path.exists() {
            let stamp = file_stamp(&path);
            match Library::load(&path) {
                Ok(parsed) => {
                    if parsed.skipped > 0 {
                        self.startup_errors.lock().push(tr!("msg-invalid-combos-ignored", count = parsed.skipped));
                    }
                    lib = parsed.library;
                    *self.library_stamp.lock() = stamp;
                }
                Err(e) => {
                    log::error!("Could not load the combo list: {e:#}");
                    // the file is not overwritten (it was not read, see save_library); keep a copy of it anyway
                    readable = false;
                    let broken = path.with_extension(format!("broken-{}.json", chrono::Local::now().format("%Y%m%d%H%M%S")));
                    let mut text = format!("{}\n{e:#}", tr("msg-library-load-failed"));
                    if std::fs::copy(&path, &broken).is_ok() {
                        text.push('\n');
                        text.push_str(&tr!("msg-library-copy-saved", path = broken.display().to_string()));
                    }
                    self.startup_errors.lock().push(text);
                }
            }
        }
        if let Ok(text) = std::fs::read_to_string(self.paths.last_use_file()) {
            lib.apply_last_use_json(&text);
        }
        needs_save |= lib.ensure_correct_grouping(tr("app-default-group"));
        needs_save |= lib.localize_default_group_names(&crate::i18n::translations("app-default-group"), tr("app-default-group"));
        *self.library.write() = lib;
        if needs_save
            && readable
            && let Err(e) = self.save_library()
        {
            log::warn!("Could not save the combo list after loading it: {e:#}");
        }
    }

    /// After a change of language, rename the groups that still have the default name of the previous one.
    pub fn localize_default_group_names(&self) {
        let renamed =
            self.library.write().localize_default_group_names(&crate::i18n::translations("app-default-group"), tr("app-default-group"));
        if renamed && let Err(e) = self.save_library() {
            log::warn!("Could not save the combo list after renaming the default group: {e:#}");
        }
    }

    /// Save the library (with an automatic backup of the previous version if enabled).
    ///
    /// A file that is not the one read or written last is never overwritten: it was changed by another computer
    /// sharing the folder (or could not be read at startup). The library is reloaded from it instead, and an
    /// error tells the user that the last change was not saved.
    pub fn save_library(&self) -> anyhow::Result<()> {
        let on_disk = file_stamp(&self.library_path());
        if on_disk.is_some() && on_disk != *self.library_stamp.lock() {
            self.reload_library()?;
            anyhow::bail!(tr("msg-library-reloaded"));
        }
        self.write_library()
    }

    /// Replace the library and save it, whatever the file contains (restore, Beeftext import...).
    pub fn replace_library(&self, lib: Library) -> anyhow::Result<()> {
        *self.library.write() = lib;
        self.write_library()
    }

    fn write_library(&self) -> anyhow::Result<()> {
        let path = self.library_path();
        if self.prefs.read().auto_backup {
            crate::backup::archive(&path, &self.backup_dir());
        }
        let lib = self.library.read();
        lib.save(&path, true)?;
        *self.library_stamp.lock() = file_stamp(&path);
        let _ = library::write_atomic(&self.paths.last_use_file(), lib.last_use_json().as_bytes());
        Ok(())
    }

    /// Reload the library if its file was changed by someone else (another computer sharing the folder).
    /// Called every few seconds by a background thread.
    pub fn reload_library_if_changed(&self) {
        let on_disk = file_stamp(&self.library_path());
        if on_disk.is_none() || on_disk == *self.library_stamp.lock() {
            return;
        }
        match self.reload_library() {
            Ok(()) => self.ui.wake(),
            // not readable yet (being synchronized...): tried again next time
            Err(e) => log::debug!("Could not reload the combo list: {e:#}"),
        }
    }

    fn reload_library(&self) -> anyhow::Result<()> {
        let path = self.library_path();
        // the stamp is taken first: if the file changes while it is read, it is simply read again later
        let stamp = file_stamp(&path);
        let mut lib = Library::load(&path)?.library;
        if let Ok(text) = std::fs::read_to_string(self.paths.last_use_file()) {
            lib.apply_last_use_json(&text);
        }
        lib.ensure_correct_grouping(tr("app-default-group"));
        *self.library.write() = lib;
        *self.library_stamp.lock() = stamp;
        log::info!("The combo list was changed by another program or computer: reloaded.");
        Ok(())
    }

    pub fn save_last_use(&self) {
        let text = self.library.read().last_use_json();
        if let Err(e) = library::write_atomic(&self.paths.last_use_file(), text.as_bytes()) {
            log::warn!("Could not save the last use file: {e:#}");
        }
    }

    pub fn mark_combo_used(&self, uuid: &Uuid) {
        if let Some(c) = self.library.write().combo_mut(uuid) {
            c.last_use = Some(library::now());
        }
        self.save_last_use();
        self.ui.wake();
    }

    pub fn mark_emoji_used(&self, shortcode: &str) {
        let mut map = self.emoji_last_use.lock();
        map.insert(shortcode.to_string(), library::now());
        let items: HashMap<&String, String> = map.iter().map(|(k, v)| (k, library::format_json_date(v))).collect();
        if let Ok(text) = serde_json::to_string_pretty(&items) {
            let _ = library::write_atomic(&self.paths.emoji_last_use_file(), text.as_bytes());
        }
    }

    pub fn save_prefs(&self) {
        if let Err(e) = self.prefs.read().save(&self.paths.settings_file()) {
            log::error!("Could not save the preferences: {e:#}");
        }
    }

    pub fn save_app_lists(&self) {
        let r = [
            self.excluded_apps.read().save(&self.paths.excluded_apps_file()),
            self.sensitive_apps.read().save(&self.paths.sensitive_apps_file()),
            self.emoji_excluded_apps.read().save(&self.paths.emoji_excluded_apps_file()),
        ];
        for e in r.into_iter().filter_map(Result::err) {
            log::error!("Could not save an application list: {e:#}");
        }
    }
}

fn load_emoji_last_use(path: &std::path::Path) -> HashMap<String, NaiveDateTime> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<HashMap<String, String>>(&t).ok())
        .map(|m| m.into_iter().filter_map(|(k, v)| library::parse_json_date(&v).map(|d| (k, d))).collect())
        .unwrap_or_default()
}

impl InputSink for AppShared {
    fn on_event(&self, event: InputEvent) {
        {
            let mut typed = self.typed.lock();
            match &event {
                InputEvent::Char(c) if c.is_whitespace() => typed.clear(),
                InputEvent::Char(c) => {
                    typed.push(*c);
                    // keep the end of the text, cutting on a character boundary (accented letters are multi-byte)
                    if typed.len() > 1024 {
                        let cut = typed.char_indices().map(|(i, _)| i).find(|i| *i >= 512).unwrap_or(0);
                        typed.drain(..cut);
                    }
                }
                InputEvent::Backspace => {
                    typed.pop();
                }
                InputEvent::Breaker | InputEvent::SwallowedSpace => typed.clear(),
                InputEvent::Shortcut(_) => {}
            }
        }
        let _ = self.engine_tx.send(EngineMsg::Input(event));
    }

    fn space_completes_combo(&self) -> bool {
        if !self.space_is_trigger() || crate::platform::is_own_process_foreground() {
            return false;
        }
        // never wait in the input thread: when the data is being changed, the space is simply not swallowed
        let (Some(prefs), Some(lib), Some(typed)) = (self.prefs.try_read(), self.library.try_read(), self.typed.try_lock()) else {
            return false;
        };
        !typed.is_empty() && lib.combos.iter().any(|c| lib.is_usable(c) && c.matches_input(&typed, &prefs))
    }

    fn wants_shortcut(&self, shortcut: &Shortcut) -> bool {
        if self.recording_shortcut.load(Ordering::Relaxed) {
            return shortcut.is_valid();
        }
        self.hook_shortcuts.try_read().map(|l| l.contains(shortcut)).unwrap_or(false)
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn is_suspended(&self) -> bool {
        self.substituting.load(Ordering::Relaxed)
    }

    fn is_excluded_app(&self, app_name: &str) -> bool {
        self.excluded_apps.try_read().map(|l| l.matches(app_name)).unwrap_or(false)
    }

    fn space_is_trigger(&self) -> bool {
        self.space_trigger.load(Ordering::Relaxed)
    }
}

/// Small helper used by the engine and the UI to wait a little.
pub fn sleep_ms(ms: u64) {
    if ms > 0 {
        std::thread::sleep(Duration::from_millis(ms));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Combo;

    fn shared_in(name: &str) -> (AppShared, PathBuf) {
        let dir = std::env::temp_dir().join(format!("mantice-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (tx, _rx) = crossbeam_channel::unbounded();
        let paths = AppPaths { data_dir: dir.clone(), portable: false };
        let (shared, _ui) = AppShared::new(paths, Preferences::default(), tx);
        (shared, dir.join(library::LIBRARY_FILE_NAME))
    }

    fn keywords(shared: &AppShared) -> Vec<String> {
        shared.library.read().combos.iter().map(|c| c.keyword.clone()).collect()
    }

    #[test]
    fn unreadable_list_is_not_overwritten() {
        let (shared, file) = shared_in("unreadable");
        let broken = r#"{ "fileFormatVersion": 10, "combos": [ {"keyword": "a", "#;
        std::fs::write(&file, broken).unwrap();
        shared.load_library();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), broken);
        assert_eq!(shared.startup_errors.lock().len(), 1);
        // a change made afterwards is not saved over the file either
        shared.library.write().combos.push(Combo::new("", "kw", "text"));
        assert!(shared.save_library().is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), broken);
    }

    #[test]
    fn changes_of_another_computer_are_reloaded_not_overwritten() {
        let (shared, file) = shared_in("shared-folder");
        shared.load_library();
        shared.library.write().combos.push(Combo::new("", "mine", "text"));
        shared.save_library().unwrap();

        // another computer sharing the folder adds a combo
        let mut other = Library::load(&file).unwrap().library;
        other.combos.push(Combo::new("", "theirs", "other text"));
        other.save(&file, true).unwrap();
        shared.reload_library_if_changed();
        assert_eq!(keywords(&shared), ["mine", "theirs"]);

        // it changes the file again while a change is being made here: the file wins, and the user is told
        other.combos.retain(|c| c.keyword != "mine");
        other.save(&file, true).unwrap();
        shared.library.write().combos.push(Combo::new("", "late", "text"));
        assert!(shared.save_library().is_err());
        assert_eq!(keywords(&shared), ["theirs"]);
        assert_eq!(Library::load(&file).unwrap().library.combos.len(), 1);
        // from now on, saving works again
        shared.library.write().combos.push(Combo::new("", "late", "text"));
        shared.save_library().unwrap();
        assert_eq!(Library::load(&file).unwrap().library.combos.len(), 2);
    }
}
