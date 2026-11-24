//! Windows backend.
//!
//! * Input monitoring with low-level keyboard and mouse hooks (`WH_KEYBOARD_LL`, `WH_MOUSE_LL`), running in a
//!   dedicated thread with its own message loop. Keystrokes are translated to text with `ToUnicodeEx`, using the
//!   keyboard layout of the foreground window and without altering the dead key state (Windows 10 1607+).
//! * Keystrokes are synthesized with `SendInput`. Synthetic events carry a marker in `dwExtraInfo` so that the
//!   hook ignores them.
//! * The clipboard backup copies every memory-based format, like Beeftext.

#![allow(clippy::missing_safety_doc)]

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use parking_lot::Mutex;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Globalization::{DATE_LONGDATE, GetDateFormatEx, GetTimeFormatEx};
use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_MEMORY, SND_NODEFAULT};
use windows_sys::Win32::System::DataExchange::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock};
use windows_sys::Win32::System::Ole::{
    CF_BITMAP, CF_DSPBITMAP, CF_DSPENHMETAFILE, CF_DSPMETAFILEPICT, CF_ENHMETAFILE, CF_METAFILEPICT, CF_OWNERDISPLAY, CF_PALETTE,
    CF_UNICODETEXT,
};
use windows_sys::Win32::System::Registry::*;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use super::{Clipboard, ClipboardBackup, InputEvent, Keyboard, PlatformStatus, SharedSink, WindowToken};
use crate::shortcut::{Key, Shortcut};
use crate::variables::DateKind;

/// Marker put in `dwExtraInfo` of the events we synthesize ("ECRF").
const INJECTED_MARKER: usize = 0x4543_5246;
/// An unassigned virtual key, used to prevent the Alt and Win keys from activating menus when released.
const VK_MASK: u16 = 0xE8;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// ---------------------------------------------------------------------------------------------------------------------
// Input monitoring
// ---------------------------------------------------------------------------------------------------------------------

static SINK: OnceLock<SharedSink> = OnceLock::new();
/// Set when a trigger space was swallowed, so that its key-up is swallowed too.
static SPACE_SWALLOWED: AtomicBool = AtomicBool::new(false);
static INPUT_OK: OnceLock<bool> = OnceLock::new();
/// Physical state of the modifier keys (bit field), tracked from non-synthetic events.
static PHYSICAL_MODIFIERS: AtomicU32 = AtomicU32::new(0);
/// Cache of the foreground window → executable name.
static APP_CACHE: Mutex<(usize, Option<String>)> = Mutex::new((0, None));

const MODIFIER_KEYS: [u16; 8] = [VK_LCONTROL, VK_RCONTROL, VK_LMENU, VK_RMENU, VK_LSHIFT, VK_RSHIFT, VK_LWIN, VK_RWIN];

fn modifier_bit(vk: u16) -> Option<u32> {
    MODIFIER_KEYS.iter().position(|k| *k == vk).map(|i| 1 << i)
}

fn normalize_modifier(vk: u16, flags: u32) -> u16 {
    // low-level hooks usually report the left/right variants, but be safe
    let extended = flags & LLKHF_EXTENDED != 0;
    match vk {
        VK_CONTROL => {
            if extended {
                VK_RCONTROL
            } else {
                VK_LCONTROL
            }
        }
        VK_MENU => {
            if extended {
                VK_RMENU
            } else {
                VK_LMENU
            }
        }
        VK_SHIFT => VK_LSHIFT,
        v => v,
    }
}

pub fn start_input_monitor(sink: SharedSink) -> anyhow::Result<()> {
    let _ = SINK.set(sink);
    let (tx, rx) = crossbeam_channel::bounded::<Result<(), String>>(1);
    std::thread::Builder::new().name("mantice-hook".into()).spawn(move || unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        let kb = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0);
        if kb.is_null() {
            let _ = tx.send(Err(format!("could not register the keyboard hook (error {})", GetLastError())));
            return;
        }
        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0);
        if mouse.is_null() {
            log::warn!("Could not register the mouse hook (error {}).", GetLastError());
        }
        let _ = tx.send(Ok(()));
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        UnhookWindowsHookEx(kb);
        if !mouse.is_null() {
            UnhookWindowsHookEx(mouse);
        }
    })?;
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => {
            let _ = INPUT_OK.set(true);
            Ok(())
        }
        Ok(Err(e)) => anyhow::bail!(e),
        Err(_) => anyhow::bail!("the keyboard hook thread did not start"),
    }
}

fn key_down(vk: u16) -> bool {
    unsafe { (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0 || (GetKeyState(vk as i32) as u16 & 0x8000) != 0 }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if code < 0 || lparam == 0 {
            return CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
        }
        let info = &*(lparam as *const KBDLLHOOKSTRUCT);
        if info.dwExtraInfo == INJECTED_MARKER {
            return CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
        }
        let msg = wparam as u32;
        let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let vk = normalize_modifier(info.vkCode as u16, info.flags);
        if let Some(bit) = modifier_bit(vk) {
            if is_down {
                PHYSICAL_MODIFIERS.fetch_or(bit, Ordering::Relaxed);
            } else {
                PHYSICAL_MODIFIERS.fetch_and(!bit, Ordering::Relaxed);
            }
        }
        // the key-up of a swallowed trigger space is swallowed as well
        if !is_down && info.vkCode as u16 == VK_SPACE && SPACE_SWALLOWED.swap(false, Ordering::Relaxed) {
            return 1;
        }
        if is_down
            && let Some(sink) = SINK.get()
            && process_key(sink.as_ref(), info)
        {
            return 1; // swallowed
        }
        CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
    }
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if code >= 0 {
            let msg = wparam as u32;
            if matches!(msg, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_MOUSEWHEEL | WM_XBUTTONDOWN)
                && let Some(sink) = SINK.get()
                && sink.is_enabled()
                && !sink.is_suspended()
            {
                sink.on_event(InputEvent::Breaker);
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
    }
}

fn cached_active_app() -> Option<String> {
    let hwnd = unsafe { GetForegroundWindow() } as usize;
    let mut cache = APP_CACHE.lock();
    if cache.0 != hwnd || hwnd == 0 {
        *cache = (hwnd, active_app_name());
    }
    cache.1.clone()
}

/// Process a key down event. Returns true if the event must be swallowed.
unsafe fn process_key(sink: &dyn super::InputSink, info: &KBDLLHOOKSTRUCT) -> bool {
    if sink.is_suspended() {
        return false;
    }
    if let Some(app) = cached_active_app()
        && sink.is_excluded_app(&app)
    {
        return false;
    }
    let vk = info.vkCode as u16;
    let ctrl = key_down(VK_CONTROL);
    let alt = key_down(VK_MENU);
    let shift = key_down(VK_SHIFT);
    let win = key_down(VK_LWIN) || key_down(VK_RWIN);

    if (ctrl || alt || win)
        && let Some(key) = vk_to_key(vk)
        && !key.is_modifier()
    {
        let sc = Shortcut { ctrl, alt, shift, meta: win, key };
        if sink.wants_shortcut(&sc) {
            sink.on_event(InputEvent::Shortcut(sc));
            if alt || win {
                send_mask_key();
            }
            return true;
        }
    }
    if !sink.is_enabled() {
        return false;
    }
    if matches!(vk, VK_LSHIFT | VK_RSHIFT | VK_SHIFT | VK_CAPITAL) {
        return false;
    }
    if matches!(vk, VK_UP | VK_RIGHT | VK_DOWN | VK_LEFT | VK_PRIOR | VK_NEXT | VK_HOME | VK_END | VK_INSERT | VK_DELETE) {
        sink.on_event(InputEvent::Breaker);
        return false;
    }
    let text = if vk == VK_PACKET {
        String::from_utf16_lossy(&[info.scanCode as u16])
    } else {
        unsafe { translate_key(info, ctrl, alt, shift, win) }
    };
    if text == " " && vk == VK_SPACE && sink.space_is_trigger() && sink.space_completes_combo() {
        sink.on_event(InputEvent::SwallowedSpace);
        SPACE_SWALLOWED.store(true, Ordering::Relaxed);
        return true;
    }
    for c in text.chars() {
        if c == '\u{8}' {
            sink.on_event(InputEvent::Backspace);
        } else if c.is_control() {
            sink.on_event(InputEvent::Breaker);
        } else if c.is_whitespace() {
            sink.on_event(if sink.space_is_trigger() { InputEvent::Char(c) } else { InputEvent::Breaker });
        } else {
            sink.on_event(InputEvent::Char(c));
        }
    }
    false
}

unsafe fn translate_key(info: &KBDLLHOOKSTRUCT, ctrl: bool, alt: bool, shift: bool, win: bool) -> String {
    unsafe {
        let mut state = [0u8; 256];
        let set = |state: &mut [u8; 256], vk: u16, on: bool| {
            if on {
                state[vk as usize] = 0x80;
            }
        };
        set(&mut state, VK_SHIFT, shift);
        set(&mut state, VK_LSHIFT, key_down(VK_LSHIFT));
        set(&mut state, VK_RSHIFT, key_down(VK_RSHIFT));
        set(&mut state, VK_CONTROL, ctrl);
        set(&mut state, VK_LCONTROL, key_down(VK_LCONTROL));
        set(&mut state, VK_RCONTROL, key_down(VK_RCONTROL));
        set(&mut state, VK_MENU, alt);
        set(&mut state, VK_LMENU, key_down(VK_LMENU));
        set(&mut state, VK_RMENU, key_down(VK_RMENU));
        set(&mut state, VK_LWIN, win);
        state[VK_CAPITAL as usize] = (GetKeyState(VK_CAPITAL as i32) & 1) as u8;
        let mut buf = [0u16; 16];
        let hwnd = GetForegroundWindow();
        let hkl = GetKeyboardLayout(if hwnd.is_null() { 0 } else { GetWindowThreadProcessId(hwnd, std::ptr::null_mut()) });
        // flag 0x4: do not change the keyboard state (keeps dead keys working in the target application)
        let n = ToUnicodeEx(info.vkCode, info.scanCode, state.as_ptr(), buf.as_mut_ptr(), buf.len() as i32, 0x4, hkl);
        if n > 0 { String::from_utf16_lossy(&buf[..n as usize]) } else { String::new() }
    }
}

/// The virtual keys of the named keys, in both directions: a key is sent with its first virtual key. The numpad
/// operators are only read (a `Key::Char` is sent with the key of the current layout, see [`key_to_vk`]).
const VK_KEYS: &[(u16, Key)] = &[
    (VK_RETURN, Key::Enter),
    (VK_SPACE, Key::Space),
    (VK_TAB, Key::Tab),
    (VK_BACK, Key::Backspace),
    (VK_ESCAPE, Key::Escape),
    (VK_INSERT, Key::Insert),
    (VK_DELETE, Key::Delete),
    (VK_HOME, Key::Home),
    (VK_END, Key::End),
    (VK_PRIOR, Key::PageUp),
    (VK_NEXT, Key::PageDown),
    (VK_LEFT, Key::Left),
    (VK_RIGHT, Key::Right),
    (VK_UP, Key::Up),
    (VK_DOWN, Key::Down),
    (VK_SNAPSHOT, Key::PrintScreen),
    (VK_PAUSE, Key::Pause),
    (VK_NUMLOCK, Key::NumLock),
    (VK_SCROLL, Key::ScrollLock),
    (VK_CAPITAL, Key::CapsLock),
    (VK_VOLUME_MUTE, Key::VolumeMute),
    (VK_VOLUME_UP, Key::VolumeUp),
    (VK_VOLUME_DOWN, Key::VolumeDown),
    (VK_MEDIA_NEXT_TRACK, Key::MediaNext),
    (VK_MEDIA_PREV_TRACK, Key::MediaPrevious),
    (VK_MEDIA_STOP, Key::MediaStop),
    (VK_MEDIA_PLAY_PAUSE, Key::MediaPlayPause),
    (VK_LAUNCH_MEDIA_SELECT, Key::MediaSelect),
    (VK_LWIN, Key::Meta),
    (VK_RWIN, Key::Meta),
    (VK_CONTROL, Key::Control),
    (VK_LCONTROL, Key::Control),
    (VK_RCONTROL, Key::Control),
    (VK_LMENU, Key::Alt),
    (VK_MENU, Key::Alt),
    (VK_RMENU, Key::Alt),
    (VK_SHIFT, Key::Shift),
    (VK_LSHIFT, Key::Shift),
    (VK_RSHIFT, Key::Shift),
    (VK_ADD, Key::Char('+')),
    (VK_SUBTRACT, Key::Char('-')),
    (VK_MULTIPLY, Key::Char('*')),
    (VK_DIVIDE, Key::Char('/')),
    (VK_DECIMAL, Key::Char('.')),
];

fn vk_to_key(vk: u16) -> Option<Key> {
    match vk {
        0x41..=0x5A | 0x30..=0x39 => Some(Key::Char(vk as u8 as char)),
        VK_NUMPAD0..=VK_NUMPAD9 => Some(Key::Char((b'0' + (vk - VK_NUMPAD0) as u8) as char)),
        VK_F1..=VK_F24 => Some(Key::F((vk - VK_F1 + 1) as u8)),
        _ => VK_KEYS.iter().find(|(v, _)| *v == vk).map(|(_, k)| *k).or_else(|| {
            // OEM keys: use the character of the current layout
            let c = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_CHAR) } & 0x7FFF;
            let c = char::from_u32(c).filter(|c| !c.is_control() && *c != '\0')?;
            Some(Key::Char(c.to_uppercase().next().unwrap_or(c)))
        }),
    }
}

fn key_to_vk(key: Key) -> Option<u16> {
    match key {
        Key::Char(c) => {
            let u = c.to_ascii_uppercase();
            if u.is_ascii_uppercase() || u.is_ascii_digit() {
                return Some(u as u16);
            }
            let hwnd = unsafe { GetForegroundWindow() };
            let hkl = unsafe { GetKeyboardLayout(if hwnd.is_null() { 0 } else { GetWindowThreadProcessId(hwnd, std::ptr::null_mut()) }) };
            let mut buf = [0u16; 2];
            let unit = c.encode_utf16(&mut buf)[0];
            let r = unsafe { VkKeyScanExW(unit, hkl) };
            (r != -1).then_some((r as u16) & 0xFF)
        }
        Key::F(n) => Some(VK_F1 + (n.clamp(1, 24) as u16) - 1),
        k => VK_KEYS.iter().find(|(_, x)| *x == k).map(|(v, _)| *v),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Keystroke synthesis
// ---------------------------------------------------------------------------------------------------------------------

fn is_extended(vk: u16) -> bool {
    matches!(
        vk,
        VK_INSERT
            | VK_DELETE
            | VK_HOME
            | VK_END
            | VK_PRIOR
            | VK_NEXT
            | VK_LEFT
            | VK_RIGHT
            | VK_UP
            | VK_DOWN
            | VK_RCONTROL
            | VK_RMENU
            | VK_LWIN
            | VK_RWIN
            | VK_APPS
            | VK_NUMLOCK
            | VK_SNAPSHOT
            | VK_DIVIDE
            | VK_VOLUME_MUTE
            | VK_VOLUME_UP
            | VK_VOLUME_DOWN
            | VK_MEDIA_NEXT_TRACK
            | VK_MEDIA_PREV_TRACK
            | VK_MEDIA_STOP
            | VK_MEDIA_PLAY_PAUSE
            | VK_LAUNCH_MEDIA_SELECT
    )
}

fn vk_input(vk: u16, up: bool) -> INPUT {
    let mut flags = if up { KEYEVENTF_KEYUP } else { 0 };
    if is_extended(vk) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: INJECTED_MARKER,
            },
        },
    }
}

fn unicode_input(unit: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: unit,
                dwFlags: KEYEVENTF_UNICODE | if up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: INJECTED_MARKER,
            },
        },
    }
}

fn send(inputs: &[INPUT]) {
    if inputs.is_empty() {
        return;
    }
    unsafe {
        SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

fn tap(vk: u16) {
    send(&[vk_input(vk, false), vk_input(vk, true)]);
}

fn send_mask_key() {
    tap(VK_MASK);
}

/// Pause between the modifiers and the key of a synthesized shortcut. Sent in a single batch, Ctrl+V reaches
/// some applications (the Windows 11 Notepad, other WinUI apps) as a plain "v": they read the current state of
/// the modifiers when they get to the key, and when they lag behind the input (Notepad after a space, with its
/// spelling and autocorrection) Ctrl has already been released. The modifiers are held long enough for that.
const CHORD_DELAY: Duration = Duration::from_millis(30);
const CHORD_RELEASE_DELAY: Duration = Duration::from_millis(60);
/// Pause before the paste shortcut, so that the application has handled the backspaces that erased the keyword.
const PASTE_SETTLE_DELAY: Duration = Duration::from_millis(30);
/// Pause between the backspaces that erase a keyword (and the arrows that place the cursor). Sent in a single
/// batch, the Windows 11 Notepad loses one of them when the keyword starts with a punctuation mark: ".cf"
/// left the "." in about half the attempts. Measured on Notepad: 5 ms still fails, 15 ms never did.
const ERASE_KEY_DELAY: Duration = Duration::from_millis(15);

/// Tap a key `count` times, pausing between the taps.
fn tap_paced(vk: u16, count: usize) {
    for i in 0..count {
        tap(vk);
        if i + 1 < count {
            std::thread::sleep(ERASE_KEY_DELAY);
        }
    }
}

/// Press a key with modifiers held, leaving the application time to see the modifiers down.
fn send_chord(modifiers: &[u16], vk: u16) {
    if !modifiers.is_empty() {
        send(&modifiers.iter().map(|m| vk_input(*m, false)).collect::<Vec<_>>());
        std::thread::sleep(CHORD_DELAY);
    }
    send(&[vk_input(vk, false)]);
    std::thread::sleep(Duration::from_millis(10));
    send(&[vk_input(vk, true)]);
    if !modifiers.is_empty() {
        std::thread::sleep(CHORD_RELEASE_DELAY);
        send(&modifiers.iter().rev().map(|m| vk_input(*m, true)).collect::<Vec<_>>());
    }
}

/// Release the modifier keys currently down. Returns the keys that were released.
fn release_modifiers() -> Vec<u16> {
    let pressed: Vec<u16> =
        MODIFIER_KEYS.iter().copied().filter(|vk| unsafe { (GetAsyncKeyState(*vk as i32) as u16 & 0x8000) != 0 }).collect();
    if pressed.is_empty() {
        return pressed;
    }
    if pressed.iter().any(|k| matches!(*k, VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN)) {
        send_mask_key();
    }
    let ups: Vec<INPUT> = pressed.iter().map(|vk| vk_input(*vk, true)).collect();
    send(&ups);
    pressed
}

/// Press again the modifiers released by [`release_modifiers`], if they are still physically held.
fn restore_modifiers(keys: &[u16]) {
    let physical = PHYSICAL_MODIFIERS.load(Ordering::Relaxed);
    let downs: Vec<INPUT> =
        keys.iter().filter(|vk| modifier_bit(**vk).map(|b| physical & b != 0).unwrap_or(false)).map(|vk| vk_input(*vk, false)).collect();
    send(&downs);
}

pub struct WinKeyboard;

impl Keyboard for WinKeyboard {
    fn erase(&mut self, count: usize) {
        let mods = release_modifiers();
        tap_paced(VK_BACK, count);
        restore_modifiers(&mods);
    }

    fn type_text(&mut self, text: &str, delay_ms: u64) {
        let mods = release_modifiers();
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    tap(VK_RETURN);
                }
                '\n' => tap(VK_RETURN),
                '\t' => tap(VK_TAB),
                c => {
                    let mut buf = [0u16; 2];
                    let inputs: Vec<INPUT> =
                        c.encode_utf16(&mut buf).iter().flat_map(|u| [unicode_input(*u, false), unicode_input(*u, true)]).collect();
                    send(&inputs);
                }
            }
            if delay_ms > 0 {
                std::thread::sleep(Duration::from_millis(delay_ms));
            }
        }
        restore_modifiers(&mods);
    }

    fn paste(&mut self, use_shift_insert: bool) {
        let mods = release_modifiers();
        std::thread::sleep(PASTE_SETTLE_DELAY);
        if use_shift_insert {
            send_chord(&[VK_LSHIFT], VK_INSERT);
        } else {
            send_chord(&[VK_LCONTROL], b'V' as u16);
        }
        restore_modifiers(&mods);
    }

    fn press_key(&mut self, key: Key, repeat: u32, delay_ms: u64) {
        let Some(vk) = key_to_vk(key) else { return };
        for i in 0..repeat {
            tap(vk);
            if i + 1 < repeat && delay_ms > 0 {
                std::thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }

    fn press_shortcut(&mut self, sc: &Shortcut) {
        let Some(vk) = key_to_vk(sc.key) else { return };
        let mods = release_modifiers();
        let held: Vec<u16> = [(sc.ctrl, VK_CONTROL), (sc.alt, VK_MENU), (sc.meta, VK_LWIN), (sc.shift, VK_SHIFT)]
            .into_iter()
            .filter_map(|(on, k)| on.then_some(k))
            .collect();
        send_chord(&held, vk);
        restore_modifiers(&mods);
    }

    fn modifiers_held(&self) -> bool {
        MODIFIER_KEYS.iter().any(|vk| unsafe { (GetAsyncKeyState(*vk as i32) as u16 & 0x8000) != 0 })
    }

    fn move_left(&mut self, count: usize) {
        let mods = release_modifiers();
        tap_paced(VK_LEFT, count);
        restore_modifiers(&mods);
    }
}

pub fn new_keyboard() -> Box<dyn Keyboard> {
    Box::new(WinKeyboard)
}

// ---------------------------------------------------------------------------------------------------------------------
// Clipboard
// ---------------------------------------------------------------------------------------------------------------------

struct SavedFormat {
    format: u32,
    data: Vec<u8>,
}

pub struct WinClipboard {
    owner: usize,
}

// SAFETY: the window handle is only used as a clipboard owner identifier.
unsafe impl Send for WinClipboard {}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open(owner: HWND) -> Option<Self> {
        for _ in 0..20 {
            if unsafe { OpenClipboard(owner) } != 0 {
                return Some(ClipboardGuard);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        log::warn!("Could not open the clipboard.");
        None
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            CloseClipboard();
        }
    }
}

fn registered_format(name: &str) -> u32 {
    let w = wide(name);
    unsafe { RegisterClipboardFormatW(w.as_ptr()) }
}

unsafe fn set_clipboard_bytes(format: u32, data: &[u8]) -> bool {
    unsafe {
        let mem = GlobalAlloc(GMEM_MOVEABLE, data.len().max(1));
        if mem.is_null() {
            return false;
        }
        let ptr = GlobalLock(mem) as *mut u8;
        if ptr.is_null() {
            GlobalFree(mem);
            return false;
        }
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        GlobalUnlock(mem);
        if SetClipboardData(format, mem).is_null() {
            GlobalFree(mem);
            return false;
        }
        true
    }
}

impl WinClipboard {
    fn new() -> Self {
        let class = wide("STATIC");
        let title = wide("Mantice clipboard");
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                std::ptr::null_mut(),
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };
        Self { owner: hwnd as usize }
    }

    fn owner(&self) -> HWND {
        self.owner as HWND
    }
}

fn ensure_crlf(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    let mut prev = '\0';
    for c in text.chars() {
        if c == '\n' && prev != '\r' {
            out.push('\r');
        }
        out.push(c);
        prev = c;
    }
    out
}

impl Clipboard for WinClipboard {
    fn text(&mut self) -> Option<String> {
        let _g = ClipboardGuard::open(self.owner())?;
        unsafe {
            let h = GetClipboardData(CF_UNICODETEXT as u32);
            if h.is_null() {
                return None;
            }
            let ptr = GlobalLock(h) as *const u16;
            if ptr.is_null() {
                return None;
            }
            let max = GlobalSize(h) / 2;
            let mut len = 0;
            while len < max && *ptr.add(len) != 0 {
                len += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
            GlobalUnlock(h);
            Some(s)
        }
    }

    fn set_text(&mut self, text: &str) -> bool {
        let Some(_g) = ClipboardGuard::open(self.owner()) else { return false };
        unsafe {
            EmptyClipboard();
            let w = wide(&ensure_crlf(text));
            let bytes = std::slice::from_raw_parts(w.as_ptr() as *const u8, w.len() * 2);
            let ok = set_clipboard_bytes(CF_UNICODETEXT as u32, bytes);
            // keep snippets out of the clipboard history (Win+V) and of the cloud clipboard
            let zero = 0u32.to_le_bytes();
            set_clipboard_bytes(registered_format("ExcludeClipboardContentFromMonitorProcessing"), &zero);
            set_clipboard_bytes(registered_format("CanIncludeInClipboardHistory"), &zero);
            set_clipboard_bytes(registered_format("CanUploadToCloudClipboard"), &zero);
            ok
        }
    }

    fn backup(&mut self) -> Option<ClipboardBackup> {
        let _g = ClipboardGuard::open(self.owner())?;
        let mut saved = Vec::new();
        unsafe {
            let mut format = 0;
            loop {
                format = EnumClipboardFormats(format);
                if format == 0 {
                    break;
                }
                // GDI handles cannot be copied as memory blocks (bitmaps are synthesized from CF_DIB anyway)
                let gdi = [
                    CF_BITMAP,
                    CF_ENHMETAFILE,
                    CF_METAFILEPICT,
                    CF_PALETTE,
                    CF_OWNERDISPLAY,
                    CF_DSPBITMAP,
                    CF_DSPENHMETAFILE,
                    CF_DSPMETAFILEPICT,
                ];
                if gdi.iter().any(|f| *f as u32 == format) {
                    continue;
                }
                let h = GetClipboardData(format);
                if h.is_null() {
                    continue;
                }
                let size = GlobalSize(h);
                if size == 0 {
                    continue;
                }
                let ptr = GlobalLock(h) as *const u8;
                if ptr.is_null() {
                    continue;
                }
                saved.push(SavedFormat { format, data: std::slice::from_raw_parts(ptr, size).to_vec() });
                GlobalUnlock(h);
            }
        }
        if saved.is_empty() { None } else { Some(ClipboardBackup(Box::new(saved))) }
    }

    fn restore(&mut self, backup: ClipboardBackup) {
        let Ok(saved) = backup.0.downcast::<Vec<SavedFormat>>() else { return };
        let Some(_g) = ClipboardGuard::open(self.owner()) else { return };
        unsafe {
            EmptyClipboard();
            for f in saved.iter() {
                set_clipboard_bytes(f.format, &f.data);
            }
        }
    }

    fn clear(&mut self) {
        if let Some(_g) = ClipboardGuard::open(self.owner()) {
            unsafe {
                EmptyClipboard();
            }
        }
    }
}

pub fn new_clipboard() -> Box<dyn Clipboard> {
    Box::new(WinClipboard::new())
}

// ---------------------------------------------------------------------------------------------------------------------
// Windows and processes
// ---------------------------------------------------------------------------------------------------------------------

fn process_image_name(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut size) != 0;
        CloseHandle(h);
        if !ok {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        path.rsplit(['\\', '/']).next().map(str::to_string)
    }
}

pub fn active_app_name() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }
        process_image_name(pid)
    }
}

pub fn foreground_window() -> Option<WindowToken> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() { None } else { Some(WindowToken(hwnd as usize as u64)) }
}

pub fn activate_window(token: WindowToken) {
    let hwnd = token.0 as usize as HWND;
    unsafe {
        if IsWindow(hwnd) == 0 {
            return;
        }
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }
        if SetForegroundWindow(hwnd) == 0 {
            // we may have lost the right to set the foreground window: an Alt key press grants it back
            send(&[vk_input(VK_LMENU, false), vk_input(VK_LMENU, true)]);
            SetForegroundWindow(hwnd);
        }
    }
}

pub fn is_own_process_foreground() -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        pid == GetCurrentProcessId()
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------------------------------------------------

pub fn powershell_exe() -> &'static str {
    "powershell.exe"
}

pub fn configure_background_command(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

fn to_systemtime(dt: &chrono::DateTime<chrono::Local>) -> SYSTEMTIME {
    use chrono::{Datelike, Timelike};
    let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut st) };
    st.wYear = dt.year() as u16;
    st.wMonth = dt.month() as u16;
    st.wDay = dt.day() as u16;
    st.wDayOfWeek = dt.weekday().num_days_from_sunday() as u16;
    st.wHour = dt.hour() as u16;
    st.wMinute = dt.minute() as u16;
    st.wSecond = dt.second() as u16;
    st.wMilliseconds = (dt.timestamp_subsec_millis().min(999)) as u16;
    st
}

/// `#{date}`, `#{time}` and `#{dateTime}` use the user's regional settings (long date format, like Beeftext).
pub fn system_date(dt: &chrono::DateTime<chrono::Local>, kind: DateKind, locale: chrono::Locale) -> String {
    let st = to_systemtime(dt);
    let date = || unsafe {
        let mut buf = [0u16; 256];
        let n =
            GetDateFormatEx(std::ptr::null(), DATE_LONGDATE, &st, std::ptr::null(), buf.as_mut_ptr(), buf.len() as i32, std::ptr::null());
        (n > 1).then(|| String::from_utf16_lossy(&buf[..(n - 1) as usize]))
    };
    let time = || unsafe {
        let mut buf = [0u16; 128];
        let n = GetTimeFormatEx(std::ptr::null(), 0, &st, std::ptr::null(), buf.as_mut_ptr(), buf.len() as i32);
        (n > 1).then(|| String::from_utf16_lossy(&buf[..(n - 1) as usize]))
    };
    match kind {
        DateKind::Date => date().unwrap_or_else(|| crate::qtdate::default_date(dt, locale)),
        DateKind::Time => time().unwrap_or_else(|| crate::qtdate::default_time(dt, locale)),
        DateKind::DateTime => match (date(), time()) {
            (Some(d), Some(t)) => format!("{d} {t}"),
            _ => crate::qtdate::default_date_time(dt, locale),
        },
    }
}

pub fn play_wav_bytes(bytes: &'static [u8]) {
    unsafe {
        PlaySoundW(bytes.as_ptr() as *const u16, std::ptr::null_mut(), SND_MEMORY | SND_ASYNC | SND_NODEFAULT);
    }
}

pub fn play_wav_file(path: &str) {
    let w = wide(path);
    unsafe {
        PlaySoundW(w.as_ptr(), std::ptr::null_mut(), SND_FILENAME | SND_ASYNC | SND_NODEFAULT);
    }
}

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "Mantice";
/// The startup entry written by ExpanderCraft, the former name of the program: it points to an executable that
/// the installer removes, so it is deleted whenever the entry is updated.
const LEGACY_RUN_VALUE: &str = "ExpanderCraft";

pub fn set_autostart(enabled: bool) -> anyhow::Result<()> {
    unsafe {
        let key_name = wide(RUN_KEY);
        let mut hkey: HKEY = std::ptr::null_mut();
        let r = RegOpenKeyExW(HKEY_CURRENT_USER, key_name.as_ptr(), 0, KEY_SET_VALUE, &mut hkey);
        if r != ERROR_SUCCESS {
            anyhow::bail!(tr!("error-registry", code = r));
        }
        RegDeleteValueW(hkey, wide(LEGACY_RUN_VALUE).as_ptr()); // absent in most cases: the result does not matter
        let value = wide(RUN_VALUE);
        let r = if enabled {
            let exe = std::env::current_exe()?;
            let data = wide(&format!("\"{}\" --background", exe.display()));
            RegSetValueExW(hkey, value.as_ptr(), 0, REG_SZ, data.as_ptr() as *const u8, (data.len() * 2) as u32)
        } else {
            let r = RegDeleteValueW(hkey, value.as_ptr());
            if r == ERROR_FILE_NOT_FOUND { ERROR_SUCCESS } else { r }
        };
        RegCloseKey(hkey);
        if r != ERROR_SUCCESS {
            anyhow::bail!(tr!("error-registry", code = r));
        }
    }
    Ok(())
}

pub fn platform_status() -> PlatformStatus {
    let ok = INPUT_OK.get().copied().unwrap_or(false);
    PlatformStatus { input_ok: ok, message: (!ok).then(|| crate::i18n::tr("platform-hook-failed").to_string()) }
}

// ---------------------------------------------------------------------------------------------------------------------
// Single instance
// ---------------------------------------------------------------------------------------------------------------------

const MUTEX_NAME: &str = "Local\\Mantice.SingleInstance.6f1c2a4e";
const EVENT_NAME: &str = "Local\\Mantice.ShowMainWindow.6f1c2a4e";

/// Returns false if another instance is already running (and asks it to show its main window).
pub fn acquire_single_instance(_socket: &std::path::Path, on_show_request: impl Fn() + Send + 'static) -> bool {
    unsafe {
        let name = wide(MUTEX_NAME);
        let mutex = CreateMutexW(std::ptr::null(), 1, name.as_ptr());
        if mutex.is_null() {
            return true;
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let ev_name = wide(EVENT_NAME);
            let ev = OpenEventW(EVENT_MODIFY_STATE, 0, ev_name.as_ptr());
            if !ev.is_null() {
                SetEvent(ev);
                CloseHandle(ev);
            }
            return false;
        }
        let ev_name = wide(EVENT_NAME);
        let ev = CreateEventW(std::ptr::null(), 0, 0, ev_name.as_ptr());
        if !ev.is_null() {
            let ev = ev as usize;
            std::thread::Builder::new()
                .name("mantice-instance".into())
                .spawn(move || {
                    loop {
                        if WaitForSingleObject(ev as HANDLE, INFINITE) == WAIT_OBJECT_0 {
                            on_show_request();
                        } else {
                            break;
                        }
                    }
                })
                .ok();
        }
        // the mutex handle is intentionally never closed: it lives as long as the process
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_keys_both_ways() {
        // the modifiers are sent with the same virtual keys as before the table
        assert_eq!(key_to_vk(Key::Meta), Some(VK_LWIN));
        assert_eq!(key_to_vk(Key::Control), Some(VK_CONTROL));
        assert_eq!(key_to_vk(Key::Alt), Some(VK_LMENU));
        assert_eq!(key_to_vk(Key::Shift), Some(VK_SHIFT));
        for (vk, key) in VK_KEYS.iter().filter(|(_, k)| !matches!(k, Key::Char(_))) {
            assert_eq!(vk_to_key(*vk), Some(*key));
            assert_eq!(key_to_vk(*key).and_then(vk_to_key), Some(*key));
        }
        assert_eq!(vk_to_key(VK_ADD), Some(Key::Char('+')));
        assert_eq!((vk_to_key(VK_F1 + 4), key_to_vk(Key::F(5))), (Some(Key::F(5)), Some(VK_F1 + 4)));
    }
}
