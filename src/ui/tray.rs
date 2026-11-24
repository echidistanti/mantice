//! System tray icon and menu.

use std::sync::Arc;

use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::app_state::{AppShared, UiMsg};
use crate::i18n::tr;

const TRAY_ICON_SIZE: u32 = 32;

pub struct Tray {
    icon: TrayIcon,
    open: MenuItem,
    enabled_item: CheckMenuItem,
    new_combo: MenuItem,
    prefs: MenuItem,
    quit: MenuItem,
    shown_enabled: bool,
    /// Language generation of the texts, see `i18n::generation`.
    shown_generation: u64,
}

fn make_icon(active: bool) -> Option<Icon> {
    Icon::from_rgba(crate::icon::render_rgba(TRAY_ICON_SIZE, active), TRAY_ICON_SIZE, TRAY_ICON_SIZE).ok()
}

fn tooltip(enabled: bool) -> String {
    if enabled { "Mantice".to_string() } else { tr("tray-tooltip-paused").to_string() }
}

impl Tray {
    /// Create the tray icon. Must be called on the thread running the platform event loop.
    pub fn create(shared: Arc<AppShared>) -> Option<Tray> {
        let enabled = shared.is_enabled();
        let generation = crate::i18n::generation();
        let menu = Menu::new();
        let open = MenuItem::with_id("open", tr("tray-open"), true, None);
        let enabled_item = CheckMenuItem::with_id("enabled", tr("app-enabled"), true, enabled, None);
        let new_combo = MenuItem::with_id("new_combo", tr("action-new-combo"), true, None);
        let prefs = MenuItem::with_id("prefs", tr("menu-preferences"), true, None);
        let quit = MenuItem::with_id("quit", tr("menu-quit"), true, None);
        let sep1 = PredefinedMenuItem::separator();
        let sep2 = PredefinedMenuItem::separator();
        if let Err(e) = menu.append_items(&[&open, &sep1, &enabled_item, &new_combo, &prefs, &sep2, &quit]) {
            log::warn!("Could not build the tray menu: {e}");
        }

        let s = shared.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| match event.id.as_ref() {
            "open" => s.ui.send(UiMsg::ShowMainWindow),
            "enabled" => s.set_enabled(!s.is_enabled()),
            "new_combo" => s.ui.send(UiMsg::NewCombo),
            "prefs" => s.ui.send(UiMsg::ShowPreferences),
            "quit" => s.ui.send(UiMsg::Quit),
            _ => {}
        }));
        let s = shared.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| match event {
            TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } => s.ui.send(UiMsg::ShowMainWindow),
            TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } => s.ui.send(UiMsg::ShowMainWindow),
            _ => {}
        }));

        let mut builder = TrayIconBuilder::new().with_menu(Box::new(menu)).with_tooltip(tooltip(enabled)).with_menu_on_left_click(false);
        if let Some(icon) = make_icon(enabled) {
            builder = builder.with_icon(icon);
        }
        match builder.build() {
            Ok(icon) => {
                Some(Tray { icon, open, enabled_item, new_combo, prefs, quit, shown_enabled: enabled, shown_generation: generation })
            }
            Err(e) => {
                log::warn!("Could not create the tray icon: {e}");
                None
            }
        }
    }

    pub fn update(&mut self, enabled: bool) {
        let generation = crate::i18n::generation();
        if generation != self.shown_generation {
            self.shown_generation = generation;
            self.open.set_text(tr("tray-open"));
            self.enabled_item.set_text(tr("app-enabled"));
            self.new_combo.set_text(tr("action-new-combo"));
            self.prefs.set_text(tr("menu-preferences"));
            self.quit.set_text(tr("menu-quit"));
            let _ = self.icon.set_tooltip(Some(tooltip(enabled)));
        }
        if self.shown_enabled == enabled && self.enabled_item.is_checked() == enabled {
            return;
        }
        self.shown_enabled = enabled;
        self.enabled_item.set_checked(enabled);
        let _ = self.icon.set_icon(make_icon(enabled));
        let _ = self.icon.set_tooltip(Some(tooltip(enabled)));
    }
}
