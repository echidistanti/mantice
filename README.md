# Mantice

[![build](https://github.com/echidistanti/mantice/actions/workflows/build.yml/badge.svg)](https://github.com/echidistanti/mantice/actions/workflows/build.yml)
[![release](https://img.shields.io/github/v/release/echidistanti/mantice)](https://github.com/echidistanti/mantice/releases/latest)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**A fast, lightweight text expander for Windows, written in Rust.**
Type a short keyword in any application and Mantice replaces it with the text you stored for it: an email signature, a boilerplate paragraph, today's date, an emoji, even a sequence of key presses.

Mantice is a clone of [Beeftext](https://github.com/xmichelo/Beeftext) (C++/Qt, MIT license). It reads and writes the same combo file format, so a Beeftext library moves over with one click and keeps working in both programs.
It also imports the packages of the [espanso hub](https://hub.espanso.org), each one into a group of its own.

The name is the Italian word for the bellows of an accordion: a short keyword opens up into a long text. Up to version 0.6.0 the program was called ExpanderCraft.

```text
you type        →  you get
@@              →  jane.doe@example.com
sig             →  Kind regards,
                   Jane Doe
today           →  2026-11-20            (#{dateTime:yyyy-MM-dd})
|tada|          →  🎉
```

## Contents

- [Features](#features)
- [Installation](#installation)
- [Getting started](#getting-started)
- [Variables](#variables)
- [Emoji](#emoji)
- [Your data and settings](#your-data-and-settings)
- [Importing espanso packages](#importing-espanso-packages)
- [Compatibility with Beeftext](#compatibility-with-beeftext)
- [Platform support](#platform-support)
- [Troubleshooting](#troubleshooting)
- [Building from source](#building-from-source)
- [Architecture](#architecture)
- [Translations](#translations)
- [Publishing a release](#publishing-a-release)
- [License and credits](#license-and-credits)

## Features

**Expansion**

- **Combos**: a keyword mapped to a snippet, with a name, a description, a group and an enabled/disabled state.
- **Strict or loose matching**, per combo or as the default for new ones. Strict means the typed text must be exactly the keyword; loose lets the combo trigger in the middle of a word. When several loose keywords match, the longest wins.
- **Case sensitivity**, per combo or as the default.
- **Automatic or manual triggering**. In automatic mode a combo fires the moment its keyword is typed. In manual mode you type the keyword and press a trigger shortcut (`Ctrl+Alt+Shift+B` by default). You can also trigger with a space typed after the keyword, and choose whether the space is kept.
- **Combo picker**: a search window opened by a global shortcut (`Ctrl+Alt+Shift+Enter` by default), sorted by last use, that inserts the combo you choose into the application you came from.
- **Enable/disable shortcut** (`Ctrl+Alt+Shift+V`, off by default) to pause Mantice without quitting it.
- **Clipboard-based insertion** that restores what was on the clipboard afterwards (on Windows every format, images and files included), keeps the pasted text out of the Windows `Win+V` and cloud clipboard histories, and can paste with `Shift+Insert` instead of `Ctrl+V` for applications that need it.
- **Excluded applications** (no expansion at all) and **sensitive applications** (the text is typed key by key instead of pasted), both with `*` and `?` wildcards.

**Snippets**

- **[Variables](#variables)** for the clipboard, dates and times (with offsets and custom formats), environment variables, user input, PowerShell output, other combos, the cursor position, key presses, shortcuts and delays. Variables nest, and recursion is detected.
- **[Emoji shortcodes](#emoji)** between delimiters (`|smile|` → 😄), with a searchable list built into the program.

**Organization and data**

- **Groups** that can be enabled, disabled and reordered.
- **Import and export** as JSON (the Beeftext format) and CSV (the TextExpander format), with conflict handling: skip, overwrite, or overwrite only the older combos.
- **[espanso packages](#importing-espanso-packages)** from the espanso hub, by name or link, or from a file or a folder, each one into a group of its own.
- **Automatic backups** at every save (the last 50 are kept), restore from any backup, and manual backup and restore.
- **A combo list in a folder of your choice**, for instance a OneDrive or Dropbox folder, to share your combos between computers. Changes made on another computer are picked up within a few seconds, and the file is never overwritten with an older list or with a list that could not be read.
- **Portable edition**: with a `Portable.bin` file next to the executable, all the data goes into a `Data` sub-folder.

**Application**

- Notification-area (tray) icon, launch at login and single-instance behavior (starting a second copy brings up the window of the first).
- A confirmation sound, either built in or a WAV file of your choice.
- Interface in **English** (the default), **Italian**, **French** and **Spanish**, switchable on the fly, with light, dark or system theme.

## Installation

### Windows 10 and 11 (64-bit)

Each [release](https://github.com/echidistanti/mantice/releases/latest) offers two downloads:

- **`Mantice-x.y.z-windows-x64-setup.exe`**, the installer. It installs for the current user by default, with no administrator rights; a dialog at the start lets you install for all users instead. It adds a Start menu entry (and, if you tick the box, a desktop shortcut) and an uninstaller in *Settings → Apps*. To update, run the installer of the new version over the old one. The installer is available in English, Italian, French and Spanish, and follows the language of Windows. If Mantice is running, setup asks you to quit it first (tray icon → Quit). Uninstalling removes the program but keeps your combos and settings in `%LOCALAPPDATA%\Mantice`.
- **`Mantice-x.y.z-windows-x64.zip`**, the portable package: extract it anywhere and run `mantice.exe`. There is nothing to install.

The installer accepts the standard [Inno Setup command-line switches](https://jrsoftware.org/ishelp/topic_setupcmdline.htm), for instance `Mantice-x.y.z-windows-x64-setup.exe /VERYSILENT /CURRENTUSER` for an unattended per-user installation.

The installer and the executable are not code-signed, so Windows SmartScreen may warn you the first time you run them: choose *More info → Run anyway*.

To make the zip version fully portable, create an empty file named `Portable.bin` next to `mantice.exe`. All the settings, the combo list and the backups then live in a `Data` folder beside the executable, so you can carry the whole thing on a USB stick.

## Getting started

1. **Launch Mantice.** The main window opens and an icon appears in the notification area. Closing the window hides it to the tray; *File → Quit* (or the tray menu) really exits.
2. **Import from Beeftext, if you have it.** On first launch, if a Beeftext library is found on the computer, Mantice offers to import it with all its groups.
3. **Create a combo.** Choose *Combo → New combo* (or press *Create your first combo* when the list is empty), type a keyword and the snippet, and save.
4. **Try it.** Type the keyword in any application. By default the combo fires as soon as the keyword is complete and the keyword is replaced by the snippet.
5. **Tune it** in *File → Preferences*: Behavior (triggering, picker and toggle shortcuts, sound, launch at login), Appearance (language and theme), Combos (default matching, warnings, combo list folder), Emojis, and Advanced (text insertion, backups, PowerShell executable, diagnostics).

### Default shortcuts

| Action | Shortcut |
|---|---|
| Open the combo picker | `Ctrl+Alt+Shift+Enter` |
| Trigger a combo manually (manual mode) | `Ctrl+Alt+Shift+B` |
| Enable or disable Mantice (off by default) | `Ctrl+Alt+Shift+V` |

All three can be changed in the preferences. A configured shortcut is intercepted: it never reaches the active application.

## Variables

A snippet can contain variables, written `#{name}` or `#{name:parameter}`. They are evaluated each time the combo fires. The same list is available in the program under *Help → Variables reference*.

| Variable | Result |
|---|---|
| `#{clipboard}` | The text on the clipboard. |
| `#{cursor}` | Where the text cursor ends up after the substitution. |
| `#{date}` `#{time}` `#{dateTime}` | The current date and/or time in the system format (on Windows, the long format of the regional settings). |
| `#{dateTime:yyyy-MM-dd HH:mm}` | A custom, Qt-style format: `d dd ddd dddd M MM MMM MMMM yy yyyy h hh H HH m mm s ss z zzz AP ap`, quoted literal text between single quotes, and `w`/`ww` for the ISO week. |
| `#{dateTime:+1d-2h:format}` | A shifted date and time (units `y M w d h m s z`, the sign is mandatory). The format may be empty; the final colon is mandatory. |
| `#{combo:kw}` `#{upper:kw}` `#{lower:kw}` `#{trim:kw}` | The snippet of another combo, optionally transformed. |
| `#{input:Description}` | Asks you for a value. The same description is asked only once per substitution. |
| `#{envVar:NAME}` | The value of an environment variable. |
| `#{powershell:path.ps1[:timeoutMs]}` | The output of a PowerShell script (10 seconds by default, `0` for no limit). |
| `#{key:name[:repeat]}` | Presses a key (`tab`, `enter`, `up`, `f5`, `volumeUp`…), optionally several times. |
| `#{shortcut:Ctrl+Shift+J}` | Presses a keyboard shortcut. |
| `#{delay:ms}` | Pauses for the given number of milliseconds. |

Inside a variable, `\}` writes a `}` and `\\` writes a `\`. Unknown variables are left in the text as they are.

For example, a combo with the keyword `meet` and this snippet:

```text
Hi #{input:Name},

Thanks for your time on #{dateTime:dddd d MMMM}. #{cursor}
```

asks for a name, fills in the date, and leaves the cursor at the end, ready for you to continue typing.

**A note on safety.** Text that comes from outside the snippet (the clipboard, `#{input:}`, `#{envVar:}`, `#{powershell:}`) is inserted exactly as it is. If it contains `#{key:}`, `#{shortcut:}`, `#{delay:}` or `#{cursor}`, those are not executed, so text copied from a web page can never press keys on your behalf.

## Emoji

Turn on *Use emoji shortcodes* in *Preferences › Emojis* and type a shortcode between the delimiters (`|` on both sides by default): `|smile|` becomes 😄. The shortcodes are the ones used by GitHub ([gemoji](https://github.com/github/gemoji)). The full list is in the program (*Help → Emoji list*, or the button in *Preferences › Emojis*): search by shortcode, English name or category, and click a row to copy its shortcode with your delimiters. Emojis can also appear in the combo picker after three typed characters, and they have their own list of excluded applications.

## Your data and settings

| | Windows |
|---|---|
| Data folder | `%LOCALAPPDATA%\Mantice` |
| Portable edition | `Data` next to the executable (when `Portable.bin` is present) |

If you used ExpanderCraft, the previous name of the program, the first launch of Mantice moves `%LOCALAPPDATA%\ExpanderCraft` to the new folder (it copies it when the old program is still running, and leaves the original in place).

The data folder contains:

| File or folder | Content |
|---|---|
| `comboList.json` | Your combos and groups (Beeftext format, version 10). Can be moved to another folder in *Preferences › Combos*. |
| `settings.json` | Your preferences. If a value cannot be read, only that value is reset and the original file is kept as `settings.broken.json`. |
| `comboLastUse.json`, `emojiLastUse.json` | Last-use data used to sort the picker. |
| `excludedApps.json`, `sensitiveApps.json`, `emojiExcludedApps.json` | The application lists. |
| `Backup/` | Automatic backups of the combo list (the 50 most recent). |
| `log.txt` | The log file, when enabled in *Preferences › Advanced*. |

*Tools → Open data folder* opens it for you. If the combo list cannot be read at startup, Mantice does not replace it with an empty one and does not overwrite it until you restore a backup.

To share combos between computers, point the combo list folder of each one to the same OneDrive or Dropbox folder.

## Importing espanso packages

*File → Import from espanso…* reads a package of [espanso](https://espanso.org), the text expander, and turns it into combos. The dialog takes:

- **the name of a package of the [espanso hub](https://hub.espanso.org)** (`basic-emojis`), or the command the hub shows (`espanso install basic-emojis`);
- **a link**: the page of a package (`https://hub.espanso.org/basic-emojis`), or a `.zip` or `.yml` file (a link to a file on GitHub is read from its raw version);
- **a file or a folder of this computer**, with the *File…* and *Folder…* buttons: a `.zip` archive, a `package.yml` (the `_manifest.yml` next to it is read too), or a folder with the match files, which can also be your own espanso `match` folder.

Then it shows what the package contains before anything is imported: the new combos, the ones that have the same keyword as an existing combo (skip, overwrite, or overwrite only the older ones, as in any import), what was left out and why, and the group they go to. By default the combos go into **a new group named after the package** (with the description of the package), which you can rename. A group with that name that already exists is proposed instead, so that importing a package again updates it; you can also pick any existing group.

**What a match becomes**

| espanso | Mantice |
|---|---|
| `trigger`, or each item of `triggers` | the keyword of a combo (one combo per trigger) |
| `replace` | the snippet |
| `label` | the name of the combo |
| `word: true` | strict matching (the keyword has to be a whole word); otherwise the keyword can be typed anywhere |
| `propagate_case: true` | case-insensitive matching. espanso also adapts the case of the text to the one typed; Mantice types the text as written |
| other matches | case-sensitive matching |
| `$\|$` | `#{cursor}` |
| variable `echo` | its text, written into the snippet |
| variable `clipboard` | `#{clipboard}` |
| variable `date` | `#{dateTime:…}`, with the `strftime` format translated into a Qt one and `offset` into a shift. A `locale` is ignored: the date uses the language of the program |
| variable `match` | `#{combo:…}`, when the match it points to is imported as well |

**What is left out**, and listed with its reason in the dialog: triggers that are regular expressions, forms, images, formatted text (`markdown`, `html`), variables that run a command or a script (`shell`, `script`) or ask for a choice (`form`, `choice`, `list`, `random`), variables Mantice does not have, date formats it cannot write, matches without a trigger or without text, and triggers that end with Tab or Enter (Mantice cannot see those keys as text). A trigger that appears twice in a package is imported once. A package can have several `.yml` files: all of them are read, as espanso does.

Some packages are huge (the translation and spelling ones have tens of thousands of triggers): the dialog warns you before they make the list very long. Downloads use the HTTP stack of Windows (proxy and certificates of the system), only over `https`, and the package is read in the background.

## Compatibility with Beeftext

Mantice reads and writes the `comboList.json` format of the latest Beeftext (version 10). Files in the older formats (1–9) have to be exported first from an up-to-date Beeftext. On first launch, if a Beeftext library is found (`%LOCALAPPDATA%\beeftext.org\Beeftext\comboList.json`), Mantice offers to import it with all its groups. A Beeftext backup can be opened with *Tools → Restore combo list*.

**Differences from Beeftext**

- `#{key:insert}` presses Ins (in Beeftext it pressed Home), and `#{key:backspace}` is available.
- `#{shortcut:}` stops at the first `}`, so several shortcuts in the same text work.
- Configured shortcuts are intercepted and do not reach the active application.
- After a substitution, modifier keys are pressed again only if you are still holding them, so Shift or Ctrl never stay stuck.
- The pasted text does not enter the Windows clipboard history.
- A combo cannot call itself with `#{combo:}`.
- Text coming from outside the snippet is inserted as plain text (see [Variables](#variables)).
- With the space as trigger and the final space kept, `#{cursor}` accounts for the added space.
- There is no update check. There are four translations (English, Italian, French, Spanish) against the eleven of Beeftext.

## Platform support

| | Windows |
|---|---|
| Keyboard and mouse capture | `WH_KEYBOARD_LL` / `WH_MOUSE_LL` hooks |
| Text from a key press | `ToUnicodeEx` with the layout of the active window, dead keys preserved |
| Key synthesis | `SendInput` (events are marked and ignored by the hook) |
| Global shortcuts intercepted | yes |
| Clipboard with backup | every format in memory; excluded from the Win+V and cloud histories |
| Active application | executable name |
| Launch at login | `HKCU\…\Run` |
| Verification | build, clippy, unit tests, and end-to-end tests under Wine (Notepad) |

Mantice runs on Windows only; macOS and Linux are not supported. All the code that depends on the operating system lives in `src/platform/`.

**Known limits**

- Like every expander based on `SendInput`, Mantice cannot type into windows running as administrator unless it is elevated too (UIPI).

## Troubleshooting

- **Nothing expands in one particular program.** Check that the program is not in the excluded applications list. If it runs as administrator, Mantice has to run as administrator too (see the known limits above).
- **Characters come out wrong, or too early.** If the application is slow to react, increase the delay between keystrokes in *Preferences › Advanced*, or add the application to the sensitive applications list so its text is typed key by key. Some programs need `Shift+Insert` instead of `Ctrl+V`: there is a switch for that in the same page.
- **The combo list cannot be loaded.** Mantice tells you, saves a copy of the file next to the original and leaves the original untouched. Restore a backup from *Tools → Restore an automatic backup*.
- **Something else.** Turn on *Write a log file* in *Preferences › Advanced*, reproduce the problem, and open the log with *Tools → Open log file*. The file is `log.txt` in the data folder.

## Building from source

You need stable Rust 1.88 or later (edition 2024): <https://rustup.rs>.

### Windows

```powershell
cargo build --release
# executable: target\release\mantice.exe
```

You can also cross-compile from Linux:

```bash
sudo apt install gcc-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

### Windows installer

The installer is built with [Inno Setup](https://jrsoftware.org/isinfo.php) 6.3 or later, from `installer/mantice.iss`. After `cargo build --release`:

```powershell
.\installer\install-inno-setup.ps1   # once: installs a pinned Inno Setup 6 (needs administrator rights)
.\installer\build.ps1                # writes installer\Output\Mantice-<version>-windows-x64-setup.exe
```

`build.ps1` takes the version from `Cargo.toml` and reuses the icon that `build.rs` draws for the executable. The `build` workflow does the same on every push: the setup program is in the artifacts of the run (`mantice-Windows-setup`), ready to try.

### Tests

```bash
cargo test
```

On Linux, where the program itself does not build, the tests can be run against the Windows version with Wine:

```bash
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine cargo test --target x86_64-pc-windows-gnu
```

About 90 unit tests cover the translations, the list shared between computers, unreadable preferences, emoji search, the file format, variables, date formats, fragments, the cursor, matching, the substitution engine (with a simulated keyboard and clipboard), and the reading of espanso packages (the reader was also checked against all the packages of the hub).

Every push and pull request is also built and tested on GitHub Actions for Windows (`build` workflow), with `cargo clippy -- -D warnings`.

## Architecture

```text
src/
├── main.rs            start-up: paths, preferences, log, single instance, threads
├── library.rs         Combo, Group, Library: Beeftext v10 JSON format, CSV, import with conflicts
├── espanso.rs         reading of espanso packages: YAML match files translated into combos (pure, tested on the whole hub)
├── espanso_source.rs  where a package comes from: folder, zip, file, hub (by name or link)
├── variables.rs       variable evaluation (the VariableHost trait keeps it testable)
├── fragments.rs       splitting into text / #{key} / #{shortcut} / #{delay}, #{cursor} computation
├── qtdate.rs          Qt-style date and time formats, shifts, localized names
├── engine.rs          engine thread: typed buffer, matching, emoji, rendering
├── app_state.rs       state shared (RwLock) between the hooks, the engine and the interface
├── i18n.rs            Fluent translations: tr("id"), tr!("id", name = value), CLDR plurals
├── prefs.rs  shortcut.rs  process_list.rs  backup.rs  emoji.rs  sound.rs  paths.rs  logger.rs  icon.rs
├── platform/
│   ├── mod.rs         Keyboard, Clipboard and InputSink traits, shared types
│   └── windows.rs     Windows backend (also the https download, with WinINet)
└── ui/                egui/eframe interface: main window, dialogs, preferences, picker, emoji list, tray
locales/               en.ftl (reference), it.ftl, fr.ftl, es.ftl
installer/             Inno Setup script and the PowerShell scripts that install Inno Setup and build the setup program
assets/                Windows resources: manifest and version information (the icon is drawn by build.rs)
.github/workflows/     build (tests, clippy, build, installer) and release (packages attached to a release)
```

The flow: the platform hook turns key presses into `InputEvent`s (character, backspace, break, shortcut) and sends them to the engine thread. The engine updates the buffer, looks for matching combos, evaluates the variables (the `#{input:}` dialog goes through the interface over a channel), erases the keyword, and pastes or types the result. The interface runs on the main thread and stays alive when its window is hidden in the tray; the picker and the input dialog are independent windows.

To add a platform (Linux is next), write a module in `platform/` that exposes the same functions (`start_input_monitor`, `new_keyboard`, `new_clipboard`, `active_app_name`, …).

## Translations

The interface text lives in `locales/*.ftl`, in the [Fluent](https://projectfluent.org) format, and is embedded in the executable. English (`en.ftl`) is the reference: a message missing from another language is shown in English. By default Mantice uses the language of the operating system when it is one of the translated ones, and English otherwise. You can pick a specific language in the preferences under Appearance. The group created at first launch, while it still has its default name, takes its name in the chosen language.

In the code, a plain message is written `tr("menu-quit")` and one with parameters `tr!("msg-combos-imported", count = n)`. Plurals follow the CLDR rules of each language (in French 0 and 1 are singular):

```ftl
msg-combos-imported =
    { $count ->
        [one] { $count } combo a été importé.
       *[other] { $count } combos ont été importés.
    }
```

To add a language:

1. copy `locales/en.ftl` to `locales/xx.ftl` and translate the values, leaving the ids and the `{ $name }` variables unchanged;
2. in `src/i18n.rs`, add a variant to `Lang` (code, name in its own language, `include_str!`);
3. in `src/prefs.rs`, add the entry to `Language`, and to the menu list in `ui/prefs_window.rs`;
4. run `cargo test`: the tests check that the file has every message of `en.ftl` with the same variables, and that every id used in the code exists.

The key names in the shortcuts shown in the preferences are translated (Maiusc, Maj, Mayús…), but in files and in `#{shortcut:}` variables they stay in English, so combos work with any language.

## Publishing a release

1. Update `version` in `Cargo.toml` and turn the *Unreleased* section of `CHANGELOG.md` into the section of the new version (`## 0.3.0 — date`).
2. Commit and push to `main`.
3. On GitHub, open *Releases → Draft a new release*, type the tag `v0.3.0` (it is created on `main`) and publish.

The `release` workflow checks that the tag matches the version in `Cargo.toml`, runs the tests, builds, and attaches `Mantice-0.3.0-windows-x64-setup.exe` (the installer) and `Mantice-0.3.0-windows-x64.zip` (the portable package) to the release. If the release description is empty, it fills it with the notes of that version from `CHANGELOG.md`. To rebuild the packages of a release that is already published, use *Actions → release → Run workflow* and give the tag.

If you attach a package built elsewhere before publishing the release, the workflow builds nothing and leaves the release as it is: in that case the description has to be written by hand. To keep the `build` workflow from starting on a push, put `[skip ci]` in the commit message.

## License and credits

[MIT](LICENSE). Mantice is inspired by [Beeftext](https://github.com/xmichelo/Beeftext) by Xavier Michelon (MIT) and follows its file format. The emoji shortcodes come from [gemoji](https://github.com/github/gemoji) (MIT) through the [`emojis`](https://crates.io/crates/emojis) crate. The interface is built with [egui](https://github.com/emilk/egui).

The full list of changes of every version is in [CHANGELOG.md](CHANGELOG.md).
