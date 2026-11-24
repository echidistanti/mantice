//! The substitution engine: tracks what the user types, detects keywords, evaluates snippets and renders them
//! in the active application. Runs in its own thread, fed by the input monitor and the UI.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::app_state::{AppShared, UiMsg, sleep_ms};
use crate::fragments::{Fragment, extract_cursor, split_fragments};
use crate::library::Combo;
use crate::platform::{self, Clipboard, ClipboardBackup, InputEvent, Keyboard, WindowToken};
use crate::prefs::Preferences;
use crate::shortcut::Shortcut;
use crate::variables::{DateKind, EvalState, VariableHost, evaluate_snippet};

/// Delay between two fragments of a snippet, as in Beeftext.
const DELAY_BETWEEN_FRAGMENTS_MS: u64 = 100;
/// How long a substitution waits for the user to release the modifier keys.
const MODIFIER_RELEASE_TIMEOUT: Duration = Duration::from_millis(1000);

/// The clipboard is restored this long after pasting a snippet.
const CLIPBOARD_RESTORE_DELAY: Duration = Duration::from_millis(1000);
/// Maximum length of the typed text buffer.
const MAX_BUFFER_CHARS: usize = 256;

pub enum EngineMsg {
    Input(InputEvent),
    /// The user picked a combo in the picker window.
    PickerCombo(Uuid),
    /// The user picked an emoji in the picker window.
    PickerEmoji {
        shortcode: String,
        emoji: String,
    },
    PickerCancelled,
    ResetBuffer,
    Quit,
}

// ---------------------------------------------------------------------------------------------------------------------
// Clipboard keeper: pastes text through the clipboard and restores the previous content one second later.
// ---------------------------------------------------------------------------------------------------------------------

struct PendingRestore {
    deadline: Instant,
    backup: Option<ClipboardBackup>,
    restore: bool,
}

pub struct ClipboardKeeper {
    backend: Box<dyn Clipboard>,
    pending: Option<PendingRestore>,
}

impl ClipboardKeeper {
    pub fn new(backend: Box<dyn Clipboard>) -> Self {
        Self { backend, pending: None }
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.pending.as_ref().map(|p| p.deadline)
    }

    pub fn paste(&mut self, text: &str, restore: bool, shift_insert: bool, keyboard: &mut dyn Keyboard) {
        // if a restoration is pending, the clipboard currently holds our previous snippet: keep the original backup
        let backup = match self.pending.take() {
            Some(p) => p.backup,
            None if restore => self.backend.backup(),
            None => None,
        };
        if !self.backend.set_text(text) {
            log::warn!("Could not put the snippet in the clipboard, typing it instead.");
            keyboard.type_text(text, 5);
            if let Some(b) = backup {
                self.backend.restore(b);
            }
            return;
        }
        keyboard.paste(shift_insert);
        self.pending = Some(PendingRestore { deadline: Instant::now() + CLIPBOARD_RESTORE_DELAY, backup, restore });
    }

    /// Restore the clipboard now if a restoration is pending.
    pub fn finish(&mut self) {
        if let Some(p) = self.pending.take()
            && p.restore
        {
            match p.backup {
                Some(b) => self.backend.restore(b),
                None => self.backend.clear(),
            }
        }
    }

    pub fn text(&mut self) -> String {
        self.finish();
        self.backend.text().unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Variable host
// ---------------------------------------------------------------------------------------------------------------------

struct Host<'a> {
    shared: &'a AppShared,
    clipboard: &'a mut ClipboardKeeper,
    combos: Vec<(String, String)>,
    prefs: &'a Preferences,
    locale: chrono::Locale,
    return_window: Option<WindowToken>,
}

impl VariableHost for Host<'_> {
    fn clipboard_text(&mut self) -> String {
        self.clipboard.text()
    }

    fn prompt_input(&mut self, description: &str) -> Option<String> {
        if self.return_window.is_none() {
            self.return_window = platform::foreground_window();
        }
        let (tx, rx) = crossbeam_channel::bounded(1);
        // ignore the keystrokes typed in the input dialog
        self.shared.begin_substitution();
        self.shared.ui.send(UiMsg::PromptInput { description: description.to_string(), reply: tx });
        let value = rx.recv().ok().flatten();
        self.shared.end_substitution();
        value
    }

    fn snippets_for_keyword(&self, keyword: &str) -> Vec<String> {
        self.combos.iter().filter(|(k, _)| k == keyword).map(|(_, s)| s.clone()).collect()
    }

    fn run_powershell(&mut self, script_path: &str, timeout_ms: u64) -> String {
        run_powershell(self.prefs, script_path, timeout_ms)
    }

    fn locale(&self) -> chrono::Locale {
        self.locale
    }

    fn system_date(&self, dt: &chrono::DateTime<chrono::Local>, kind: DateKind) -> String {
        platform::system_date(dt, kind, self.locale)
    }

    fn random_index(&mut self, len: usize) -> usize {
        random_index(len)
    }
}

fn random_index(len: usize) -> usize {
    // a version 4 UUID comes from the random generator of the system
    if len <= 1 { 0 } else { (Uuid::new_v4().as_u128() % len as u128) as usize }
}

/// Run a PowerShell script and return its standard output (empty on error or timeout).
pub fn run_powershell(prefs: &Preferences, script_path: &str, timeout_ms: u64) -> String {
    use std::io::Read;
    if !std::path::Path::new(script_path).is_file() {
        log::warn!("Evaluation of #{{powershell:}} failed: the file '{script_path}' does not exist.");
        return String::new();
    }
    let mut exe = platform::powershell_exe().to_string();
    if prefs.use_custom_powershell && !prefs.custom_powershell_path.trim().is_empty() {
        if std::path::Path::new(&prefs.custom_powershell_path).is_file() {
            exe = prefs.custom_powershell_path.clone();
        } else {
            log::warn!("The custom PowerShell executable '{}' is invalid.", prefs.custom_powershell_path);
        }
    }
    let mut cmd = std::process::Command::new(exe);
    cmd.args(["-NonInteractive", "-ExecutionPolicy", "Unrestricted", "-File", script_path])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null());
    platform::configure_background_command(&mut cmd);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Evaluation of #{{powershell:}} failed: {e}");
            return String::new();
        }
    };
    // read the output in a separate thread, so that a script producing a lot of output cannot block
    let mut stdout = child.stdout.take();
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        if let Some(s) = stdout.as_mut() {
            let _ = s.read_to_end(&mut out);
        }
        out
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let out = reader.join().unwrap_or_default();
                if !status.success() {
                    log::warn!("Execution of '{script_path}' returned an error ({status}).");
                    return String::new();
                }
                return String::from_utf8_lossy(&out).into_owned();
            }
            Ok(None) => {
                if timeout_ms > 0 && start.elapsed() > Duration::from_millis(timeout_ms) {
                    let _ = child.kill();
                    let _ = child.wait();
                    log::warn!("The script '{script_path}' timed out.");
                    return String::new();
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                log::warn!("Error while waiting for '{script_path}': {e}");
                return String::new();
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------------------------------------------------

pub struct Engine {
    shared: Arc<AppShared>,
    rx: Receiver<EngineMsg>,
    buffer: String,
    keyboard: Box<dyn Keyboard>,
    clipboard: ClipboardKeeper,
    picker_return: Option<WindowToken>,
    locale: chrono::Locale,
    /// True while handling a space that the input monitor swallowed.
    space_swallowed: bool,
}

impl Engine {
    pub fn new(shared: Arc<AppShared>, rx: Receiver<EngineMsg>, keyboard: Box<dyn Keyboard>, clipboard: Box<dyn Clipboard>) -> Self {
        Self {
            shared,
            rx,
            buffer: String::new(),
            keyboard,
            clipboard: ClipboardKeeper::new(clipboard),
            picker_return: None,
            locale: crate::qtdate::system_locale(),
            space_swallowed: false,
        }
    }

    pub fn spawn(self) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new().name("mantice-engine".into()).spawn(move || self.run()).expect("could not start the engine thread")
    }

    fn run(mut self) {
        loop {
            let msg = match self.clipboard.deadline() {
                Some(deadline) => match self.rx.recv_deadline(deadline) {
                    Ok(m) => m,
                    Err(RecvTimeoutError::Timeout) => {
                        self.clipboard.finish();
                        continue;
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                },
                None => match self.rx.recv() {
                    Ok(m) => m,
                    Err(_) => break,
                },
            };
            if matches!(msg, EngineMsg::Quit) {
                break;
            }
            // an error (a panic) while handling one event must not stop the expansions until the next restart
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.handle(msg))).is_err() {
                log::error!("An error occurred while handling an event: it was ignored.");
                self.shared.end_substitution();
                self.buffer.clear();
                self.space_swallowed = false;
            }
        }
        self.clipboard.finish();
        log::info!("Engine stopped.");
    }

    fn handle(&mut self, msg: EngineMsg) {
        match msg {
            EngineMsg::Quit => {}
            EngineMsg::ResetBuffer => self.buffer.clear(),
            EngineMsg::Input(ev) => self.on_input(ev),
            EngineMsg::PickerCombo(uuid) => self.on_picker_combo(uuid),
            EngineMsg::PickerEmoji { shortcode, emoji } => self.on_picker_emoji(&shortcode, &emoji),
            EngineMsg::PickerCancelled => {
                self.buffer.clear();
                if let Some(w) = self.picker_return.take() {
                    platform::activate_window(w);
                }
            }
        }
    }

    fn on_input(&mut self, ev: InputEvent) {
        match ev {
            InputEvent::Char(c) => {
                self.buffer.push(c);
                let count = self.buffer.chars().count();
                if count > MAX_BUFFER_CHARS {
                    self.buffer = self.buffer.chars().skip(count - MAX_BUFFER_CHARS).collect();
                }
                let (auto, on_space) = {
                    let p = self.shared.prefs.read();
                    (p.use_automatic_substitution, p.triggers_on_space())
                };
                if !auto || (on_space && !c.is_whitespace()) {
                    return;
                }
                self.check_and_substitute();
            }
            InputEvent::Backspace => {
                self.buffer.pop();
            }
            InputEvent::Breaker => self.buffer.clear(),
            InputEvent::Shortcut(sc) => self.on_shortcut(sc),
            InputEvent::SwallowedSpace => {
                // the application did not get this space: there is one character less to erase, and if the
                // combo cannot be performed after all, the space is typed so that it is not lost
                self.buffer.push(' ');
                self.space_swallowed = true;
                if !self.check_and_substitute() {
                    self.keyboard.type_text(" ", 0);
                }
                self.space_swallowed = false;
            }
        }
    }

    fn on_shortcut(&mut self, sc: Shortcut) {
        log::debug!("Shortcut: {sc}");
        if self.shared.recording_shortcut.load(std::sync::atomic::Ordering::Relaxed) {
            self.shared.ui.send(UiMsg::ShortcutRecorded(sc));
            return;
        }
        let (toggle, enabled, picker, manual) = {
            let p = self.shared.prefs.read();
            (
                p.enable_app_toggle_shortcut && sc == p.app_toggle_shortcut,
                p.enabled,
                p.combo_picker_enabled && sc == p.combo_picker_shortcut,
                !p.use_automatic_substitution && sc == p.combo_trigger_shortcut,
            )
        };
        if toggle {
            let now_enabled = !self.shared.is_enabled();
            self.shared.set_enabled(now_enabled);
            log::info!("Mantice {} with the shortcut.", if now_enabled { "enabled" } else { "disabled" });
            return;
        }
        if !enabled {
            return;
        }
        if picker {
            // pressed again while the picker is open: keep the window to go back to
            if !platform::is_own_process_foreground() {
                self.picker_return = platform::foreground_window();
            }
            self.buffer.clear();
            self.shared.ui.send(UiMsg::ShowPicker);
            return;
        }
        if manual {
            self.check_and_substitute();
        }
    }

    /// Returns true if a combo or an emoji was substituted.
    fn check_and_substitute(&mut self) -> bool {
        let prefs = self.shared.prefs.read().clone();
        let on_space = prefs.triggers_on_space();
        let mut text = self.buffer.clone();
        if on_space {
            if !text.ends_with(char::is_whitespace) {
                return false;
            }
            text.pop();
        }
        let candidates: Vec<Combo> = {
            let lib = self.shared.library.read();
            let matching: Vec<&Combo> = lib.combos.iter().filter(|c| lib.is_usable(c) && c.matches_input(&text, &prefs)).collect();
            // with loose matching "@@@" also matches the keyword "@@": the longest keyword wins, and combos sharing
            // that keyword are picked at random
            let longest = matching.iter().map(|c| c.keyword_len()).max().unwrap_or(0);
            matching.into_iter().filter(|c| c.keyword_len() == longest).cloned().collect()
        };
        if candidates.is_empty() {
            let emoji = self.try_emoji(&text, &prefs);
            if !emoji && on_space {
                self.buffer.clear();
            }
            return emoji;
        }
        self.buffer.clear();
        if platform::is_own_process_foreground() {
            return false; // no substitution in Mantice's own windows
        }
        let combo = candidates[random_index(candidates.len())].clone();
        if self.perform_combo(&combo, false, &prefs) {
            self.after_substitution(&prefs);
            self.shared.mark_combo_used(&combo.uuid);
            return true;
        }
        false
    }

    /// Characters typed after the keyword that must be erased with it: the space that triggered the combo,
    /// unless the input monitor swallowed it.
    fn trigger_chars(&self, prefs: &Preferences) -> usize {
        usize::from(prefs.triggers_on_space() && !self.space_swallowed)
    }

    fn try_emoji(&mut self, text: &str, prefs: &Preferences) -> bool {
        let (l, r) = (prefs.emoji_left_delimiter.as_str(), prefs.emoji_right_delimiter.as_str());
        if !prefs.emoji_shortcodes_enabled || l.is_empty() || r.is_empty() {
            return false;
        }
        let Some(stripped) = text.strip_suffix(r) else { return false };
        let Some(idx) = stripped.rfind(l) else { return false };
        let shortcode = &stripped[idx + l.len()..];
        let Some(emoji) = crate::emoji::find(shortcode) else { return false };
        self.buffer.clear();
        if platform::is_own_process_foreground() {
            return true;
        }
        if let Some(app) = platform::active_app_name()
            && self.shared.emoji_excluded_apps.read().matches(&app)
        {
            return true;
        }
        let erase = l.graphemes(true).count() + shortcode.graphemes(true).count() + r.graphemes(true).count() + self.trigger_chars(prefs);
        self.render(emoji, erase, false, prefs);
        self.after_substitution(prefs);
        self.shared.mark_emoji_used(shortcode);
        true
    }

    fn on_picker_combo(&mut self, uuid: Uuid) {
        self.buffer.clear();
        let return_to = self.picker_return.take();
        let Some(combo) = self.shared.library.read().combo(&uuid).cloned() else { return };
        if let Some(w) = return_to {
            platform::activate_window(w);
        }
        sleep_ms(150);
        let prefs = self.shared.prefs.read().clone();
        if self.perform_combo(&combo, true, &prefs) {
            self.after_substitution(&prefs);
            self.shared.mark_combo_used(&uuid);
        }
    }

    fn on_picker_emoji(&mut self, shortcode: &str, emoji: &str) {
        self.buffer.clear();
        if let Some(w) = self.picker_return.take() {
            platform::activate_window(w);
        }
        sleep_ms(150);
        let prefs = self.shared.prefs.read().clone();
        self.render(emoji, 0, true, &prefs);
        self.after_substitution(&prefs);
        self.shared.mark_emoji_used(shortcode);
    }

    fn after_substitution(&self, prefs: &Preferences) {
        if prefs.play_sound_on_combo {
            crate::sound::play(prefs.use_custom_sound.then_some(prefs.custom_sound_path.as_str()));
        }
    }

    /// Evaluate the snippet of a combo and render it. Returns false if the user cancelled an input dialog.
    fn perform_combo(&mut self, combo: &Combo, from_picker: bool, prefs: &Preferences) -> bool {
        let combos: Vec<(String, String)> =
            self.shared.library.read().combos.iter().map(|c| (c.keyword.clone(), c.snippet.clone())).collect();
        let mut forbidden = HashSet::new();
        if !combo.keyword.is_empty() {
            forbidden.insert(combo.keyword.clone());
        }
        let mut state = EvalState::default();
        let (evaluated, return_window) = {
            let mut host =
                Host { shared: &self.shared, clipboard: &mut self.clipboard, combos, prefs, locale: self.locale, return_window: None };
            let r = evaluate_snippet(&combo.snippet, &mut host, &forbidden, &mut state);
            (r, host.return_window)
        };
        if let Some(w) = return_window {
            platform::activate_window(w);
        }
        let Some(text) = evaluated else {
            log::info!("Substitution cancelled by the user.");
            return false;
        };
        if state.prompted_user() {
            // slow applications (Electron & co) need some time to properly regain focus
            sleep_ms(300);
        }
        let erase = if from_picker { 0 } else { combo.keyword_len() + self.trigger_chars(prefs) };
        self.render(&text, erase, from_picker, prefs);
        true
    }

    /// Erase the keyword and render the evaluated snippet.
    fn render(&mut self, text: &str, erase: usize, from_picker: bool, prefs: &Preferences) {
        let (text, cursor) = extract_cursor(text);
        let mut fragments = split_fragments(&text);
        let final_space = !from_picker && prefs.triggers_on_space() && prefs.keep_final_space_character;
        if final_space {
            match fragments.last_mut() {
                Some(Fragment::Text(t)) => t.push(' '),
                _ => fragments.push(Fragment::Text(" ".into())),
            }
        }
        let cursor = cursor.map(|c| c + usize::from(final_space));

        // a panic in here is caught by `run`, which also ends the substitution
        self.shared.begin_substitution();
        // A keyword can end with a character typed with a modifier (@ is AltGr+ò on an Italian keyboard). Sending
        // keys while the user holds it mixes the two: the application can see a lone Alt press, which activates
        // the menu or the keytips of Windows 11 apps such as Notepad, and the paste shortcut gets lost.
        let wait_start = Instant::now();
        while self.keyboard.modifiers_held() && wait_start.elapsed() < MODIFIER_RELEASE_TIMEOUT {
            sleep_ms(10);
        }
        if erase > 0 {
            self.keyboard.erase(erase);
        }
        let count = fragments.len();
        for (i, f) in fragments.iter().enumerate() {
            match f {
                Fragment::Text(t) => self.insert_text(t, prefs),
                Fragment::Key { key, repeat } => self.keyboard.press_key(*key, *repeat, prefs.delay_between_keystrokes_ms),
                Fragment::Shortcut(s) => self.keyboard.press_shortcut(s),
                Fragment::Delay(ms) => sleep_ms(*ms),
            }
            if i + 1 != count {
                sleep_ms(DELAY_BETWEEN_FRAGMENTS_MS);
            }
        }
        if let Some(n) = cursor
            && n > 0
        {
            self.keyboard.move_left(n);
        }
        self.shared.end_substitution();
        self.buffer.clear();
    }

    fn insert_text(&mut self, text: &str, prefs: &Preferences) {
        let sensitive = platform::active_app_name().map(|app| self.shared.sensitive_apps.read().matches(&app)).unwrap_or(false);
        if sensitive {
            self.keyboard.type_text(text, prefs.delay_between_keystrokes_ms);
        } else {
            self.clipboard.paste(
                text,
                prefs.restore_clipboard_after_substitution,
                prefs.use_shift_insert_for_pasting,
                self.keyboard.as_mut(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shortcut::Key;
    use parking_lot::Mutex;

    #[derive(Default)]
    struct Log(Arc<Mutex<Vec<String>>>);

    struct MockKeyboard(Arc<Mutex<Vec<String>>>);
    impl Keyboard for MockKeyboard {
        fn erase(&mut self, count: usize) {
            self.0.lock().push(format!("erase({count})"));
        }
        fn type_text(&mut self, text: &str, _delay_ms: u64) {
            self.0.lock().push(format!("type({text})"));
        }
        fn paste(&mut self, _s: bool) {
            self.0.lock().push("paste".into());
        }
        fn press_key(&mut self, key: Key, repeat: u32, _d: u64) {
            self.0.lock().push(format!("key({key:?},{repeat})"));
        }
        fn press_shortcut(&mut self, s: &Shortcut) {
            self.0.lock().push(format!("shortcut({s})"));
        }
        fn move_left(&mut self, count: usize) {
            self.0.lock().push(format!("left({count})"));
        }
    }

    struct MockClipboard(Arc<Mutex<Vec<String>>>, Option<String>);
    impl Clipboard for MockClipboard {
        fn text(&mut self) -> Option<String> {
            self.1.clone()
        }
        fn set_text(&mut self, text: &str) -> bool {
            self.0.lock().push(format!("clip({text})"));
            self.1 = Some(text.to_string());
            true
        }
        fn backup(&mut self) -> Option<ClipboardBackup> {
            self.1.clone().map(|t| ClipboardBackup(Box::new(t)))
        }
        fn restore(&mut self, backup: ClipboardBackup) {
            let t = *backup.0.downcast::<String>().unwrap();
            self.0.lock().push(format!("restore({t})"));
            self.1 = Some(t);
        }
        fn clear(&mut self) {
            self.1 = None;
        }
    }

    fn engine_with(snippets: &[(&str, &str)], mut prefs: Preferences) -> (Engine, Arc<Mutex<Vec<String>>>) {
        prefs.play_sound_on_combo = false;
        let (tx, rx) = crossbeam_channel::unbounded();
        let paths = crate::paths::AppPaths { data_dir: std::env::temp_dir().join("mantice-engine-test"), portable: false };
        let (shared, _ui_rx) = AppShared::new(paths, prefs, tx);
        {
            let mut lib = shared.library.write();
            for (k, s) in snippets {
                lib.combos.push(Combo::new("", k, s));
            }
        }
        let log = Log::default().0;
        let engine = Engine::new(
            Arc::new(shared),
            rx,
            Box::new(MockKeyboard(log.clone())),
            Box::new(MockClipboard(log.clone(), Some("user clipboard".into()))),
        );
        (engine, log)
    }

    fn type_str(e: &mut Engine, s: &str) {
        for c in s.chars() {
            e.on_input(InputEvent::Char(c));
        }
    }

    #[test]
    fn simple_substitution() {
        let (mut e, log) = engine_with(&[("btsig", "Regards,\nJohn")], Preferences::default());
        type_str(&mut e, "btsi");
        assert!(log.lock().is_empty());
        type_str(&mut e, "g");
        assert_eq!(*log.lock(), vec!["erase(5)", "clip(Regards,\nJohn)", "paste"]);
        assert!(e.buffer.is_empty());
        e.clipboard.finish();
        assert_eq!(log.lock().last().unwrap(), "restore(user clipboard)");
    }

    #[test]
    fn strict_matching_needs_breaker() {
        let (mut e, log) = engine_with(&[("==>", "➜")], Preferences::default());
        type_str(&mut e, "First==>");
        assert!(log.lock().is_empty());
        e.on_input(InputEvent::Breaker);
        type_str(&mut e, "==>");
        assert_eq!(log.lock()[0], "erase(3)");
    }

    #[test]
    fn backspace_edits_buffer() {
        let (mut e, log) = engine_with(&[("abc", "x")], Preferences::default());
        type_str(&mut e, "abd");
        e.on_input(InputEvent::Backspace);
        type_str(&mut e, "c");
        assert_eq!(log.lock()[0], "erase(3)");
    }

    #[test]
    fn fragments_and_cursor() {
        let (mut e, log) = engine_with(&[("kw", "a#{key:tab}b#{cursor}cd#{delay:1}")], Preferences::default());
        type_str(&mut e, "kw");
        assert_eq!(*log.lock(), vec!["erase(2)", "clip(a)", "paste", "key(Tab,1)", "clip(bcd)", "paste", "left(2)"]);
    }

    #[test]
    fn trigger_on_space_keeps_space() {
        let prefs = Preferences { combo_triggers_on_space: true, keep_final_space_character: true, ..Default::default() };
        let (mut e, log) = engine_with(&[("kw", "hello")], prefs);
        type_str(&mut e, "kw");
        assert!(log.lock().is_empty());
        type_str(&mut e, " ");
        assert_eq!(*log.lock(), vec!["erase(3)", "clip(hello )", "paste"]);
    }

    #[test]
    fn swallowed_trigger_space() {
        let prefs = Preferences { combo_triggers_on_space: true, ..Default::default() };
        let (mut e, log) = engine_with(&[("kw", "hello")], prefs);
        // the input thread decides to swallow the space: it saw "kw" typed
        let shared = e.shared.clone();
        use crate::platform::InputSink as _;
        for c in "kw".chars() {
            shared.on_event(InputEvent::Char(c));
            e.on_input(InputEvent::Char(c));
        }
        assert!(shared.space_completes_combo());
        e.on_input(InputEvent::SwallowedSpace);
        // the space never reached the application: only the keyword is erased
        assert_eq!(*log.lock(), vec!["erase(2)", "clip(hello)", "paste"]);
        // after the substitution, a new space does not complete anything
        shared.on_event(InputEvent::SwallowedSpace);
        assert!(!shared.space_completes_combo());
    }

    #[test]
    fn longest_keyword_wins_with_loose_matching() {
        let prefs =
            Preferences { combo_triggers_on_space: true, default_matching_mode: crate::library::MatchingMode::Loose, ..Default::default() };
        let (mut e, log) = engine_with(&[("@@", "short"), ("@@@", "long")], prefs);
        type_str(&mut e, "@@@ ");
        assert_eq!(*log.lock(), vec!["erase(4)", "clip(long)", "paste"]);
    }

    #[test]
    fn swallowed_space_is_typed_back_when_nothing_matches() {
        let prefs = Preferences { combo_triggers_on_space: true, ..Default::default() };
        let (mut e, log) = engine_with(&[("kw", "hello")], prefs);
        // the engine's buffer differs from the input thread's (it was reset in between)
        type_str(&mut e, "k");
        e.on_input(InputEvent::SwallowedSpace);
        assert_eq!(*log.lock(), vec!["type( )"]);
    }

    #[test]
    fn emoji_shortcodes() {
        let prefs = Preferences { emoji_shortcodes_enabled: true, ..Default::default() };
        let (mut e, log) = engine_with(&[], prefs);
        type_str(&mut e, "hi |smirk_cat|");
        assert_eq!(*log.lock(), vec!["erase(11)", "clip(😼)", "paste"]);
    }

    #[test]
    fn clipboard_variable_reads_user_clipboard() {
        let (mut e, log) = engine_with(&[("a1", "[#{clipboard}]"), ("a2", "x")], Preferences::default());
        type_str(&mut e, "a2");
        e.on_input(InputEvent::Breaker);
        // the restore of the first paste is still pending: #{clipboard} must see the original content
        type_str(&mut e, "a1");
        assert!(log.lock().contains(&"clip([user clipboard])".to_string()));
    }

    #[test]
    fn copied_text_is_pasted_not_run() {
        let (mut e, log) = engine_with(&[("kw", "> #{clipboard}"), ("ht", "##{clipboard}")], Preferences::default());
        let copied = "#{shortcut:Win+R}#{delay:500}cmd#{key:enter}";
        e.clipboard = ClipboardKeeper::new(Box::new(MockClipboard(log.clone(), Some(copied.into()))));
        type_str(&mut e, "kw");
        assert_eq!(*log.lock(), vec!["erase(2)".to_string(), format!("clip(> {copied})"), "paste".into()]);
        // a '#' written by the snippet just before the copied text does not make it run either
        e.clipboard = ClipboardKeeper::new(Box::new(MockClipboard(log.clone(), Some("{key:enter}".into()))));
        log.lock().clear();
        e.on_input(InputEvent::Breaker);
        type_str(&mut e, "ht");
        assert_eq!(*log.lock(), vec!["erase(2)", "clip(#{key:enter})", "paste"]);
    }

    #[test]
    fn huge_date_shift_does_not_crash() {
        let (mut e, log) = engine_with(&[("kw", "#{dateTime:+100000000d:yyyy}")], Preferences::default());
        type_str(&mut e, "kw");
        assert_eq!(log.lock()[0], "erase(2)");
    }

    /// Panics the first time it erases a keyword.
    struct PanickingKeyboard(MockKeyboard, bool);
    impl Keyboard for PanickingKeyboard {
        fn erase(&mut self, count: usize) {
            if !std::mem::replace(&mut self.1, true) {
                panic!("test panic");
            }
            self.0.erase(count);
        }
        fn type_text(&mut self, text: &str, delay_ms: u64) {
            self.0.type_text(text, delay_ms);
        }
        fn paste(&mut self, s: bool) {
            self.0.paste(s);
        }
        fn press_key(&mut self, key: Key, repeat: u32, d: u64) {
            self.0.press_key(key, repeat, d);
        }
        fn press_shortcut(&mut self, s: &Shortcut) {
            self.0.press_shortcut(s);
        }
        fn move_left(&mut self, count: usize) {
            self.0.move_left(count);
        }
    }

    #[test]
    fn engine_survives_a_panic() {
        let (mut e, log) = engine_with(&[("kw", "x")], Preferences::default());
        e.keyboard = Box::new(PanickingKeyboard(MockKeyboard(log.clone()), false));
        let shared = e.shared.clone();
        for _ in 0..2 {
            for c in "kw".chars() {
                shared.engine_tx.send(EngineMsg::Input(InputEvent::Char(c))).unwrap();
            }
        }
        shared.engine_tx.send(EngineMsg::Quit).unwrap();
        e.run();
        // the first substitution failed, the second one worked (and the clipboard was restored on quitting)
        assert!(!shared.substituting.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(*log.lock(), vec!["erase(2)", "clip(x)", "paste", "restore(user clipboard)"]);
    }

    #[test]
    fn manual_trigger() {
        let prefs = Preferences { use_automatic_substitution: false, ..Default::default() };
        let (mut e, log) = engine_with(&[("kw", "x")], prefs);
        type_str(&mut e, "kw");
        assert!(log.lock().is_empty());
        e.on_input(InputEvent::Shortcut(Shortcut::default_trigger()));
        assert_eq!(log.lock()[0], "erase(2)");
    }
}
