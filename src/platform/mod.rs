//! Platform abstraction layer.
//!
//! The Windows backend provides a set of free functions and implements the [`Keyboard`] and [`Clipboard`]
//! traits. The rest of the application only talks to this module.
//!
//! | Feature                    | Windows                 |
//! |----------------------------|-------------------------|
//! | Keyboard/mouse monitoring  | `WH_KEYBOARD_LL` hooks  |
//! | Keystroke synthesis        | `SendInput`             |
//! | Clipboard backup           | all formats (Win32)     |
//! | Active application         | process image name      |
//! | Auto-start                 | `HKCU\...\Run`          |

use std::sync::Arc;

use crate::shortcut::{Key, Shortcut};

/// An event reported by the input monitor.
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// A printable character was typed.
    Char(char),
    Backspace,
    /// A "combo breaker": a non printable key, a mouse click, etc. The typed text buffer is reset.
    Breaker,
    /// A shortcut (Ctrl, Alt or Meta + key) that the sink asked to receive.
    Shortcut(Shortcut),
    /// A space that completes a combo when the space is the trigger, swallowed by the input monitor: the
    /// application never received it (see [`InputSink::space_completes_combo`]).
    SwallowedSpace,
}

/// The receiver of input events. Called from the input monitoring thread: implementations must be fast.
pub trait InputSink: Send + Sync {
    fn on_event(&self, event: InputEvent);
    /// Should this shortcut be reported (and swallowed, where the platform allows it)?
    fn wants_shortcut(&self, shortcut: &Shortcut) -> bool;
    /// Is text expansion enabled? Shortcuts are still reported when disabled.
    fn is_enabled(&self) -> bool;
    /// True while a substitution is in progress: events are ignored.
    fn is_suspended(&self) -> bool;
    fn is_excluded_app(&self, app_name: &str) -> bool;
    /// Is the space character a trigger (rather than a combo breaker)?
    fn space_is_trigger(&self) -> bool;
    /// When the space is the trigger: would a space typed now complete a combo? The input monitor then
    /// swallows it, so that the application does not react to it (spelling, autocorrection) while the keyword
    /// is replaced: the Windows 11 Notepad, busy with it, could receive the paste shortcut as a plain "v".
    fn space_completes_combo(&self) -> bool {
        false
    }
}

/// Keystroke synthesis.
pub trait Keyboard: Send {
    /// Erase `count` characters with backspace (pressed modifiers are temporarily released).
    fn erase(&mut self, count: usize);
    /// Type text, character by character.
    fn type_text(&mut self, text: &str, delay_ms: u64);
    /// Trigger the paste command of the active application (Ctrl+V or Shift+Insert).
    fn paste(&mut self, use_shift_insert: bool);
    fn press_key(&mut self, key: Key, repeat: u32, delay_ms: u64);
    fn press_shortcut(&mut self, shortcut: &Shortcut);
    fn move_left(&mut self, count: usize);
    /// Whether the user is holding a modifier key (Ctrl, Alt, AltGr, Shift, Win). The substitution waits for
    /// their release, so that the keys it sends are not mixed with the ones the user holds.
    fn modifiers_held(&self) -> bool {
        false
    }
}

/// Opaque clipboard content saved before pasting a snippet.
pub struct ClipboardBackup(pub Box<dyn std::any::Any + Send>);

pub trait Clipboard: Send {
    fn text(&mut self) -> Option<String>;
    /// Put text in the clipboard (when possible, flagged so that clipboard history managers ignore it).
    fn set_text(&mut self, text: &str) -> bool;
    fn backup(&mut self) -> Option<ClipboardBackup>;
    fn restore(&mut self, backup: ClipboardBackup);
    fn clear(&mut self);
}

/// Identifies a window (or application) that can be re-activated later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowToken(pub u64);

#[derive(Debug, Clone, Default)]
pub struct PlatformStatus {
    /// Is the global input monitoring working?
    pub input_ok: bool,
    /// A message for the user (missing permission, limited support...).
    pub message: Option<String>,
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;

#[cfg(not(windows))]
compile_error!("Mantice supports Windows only.");

/// Open a file or folder with the default application of the system.
pub fn open_path(path: &std::path::Path) {
    let mut cmd = std::process::Command::new("explorer");
    cmd.arg(path);
    if let Err(e) = cmd.spawn() {
        log::warn!("Could not open '{}': {e}", path.display());
    }
}

pub type SharedSink = Arc<dyn InputSink>;
