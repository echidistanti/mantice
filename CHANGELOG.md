# Changelog

The changes of every version of Mantice (called ExpanderCraft up to version 0.6.0). Each GitHub release carries the section of its own version.

## Unreleased

- **ExpanderCraft is now Mantice**, the Italian word for the bellows of an accordion: a short keyword opens up into a long text. The program has a new icon (a bellows that widens, with the text cursor at its narrow end), and the executable, the installer and the packages are named `mantice`. Nothing to do when you update: at the first launch the data folder is moved from `%LOCALAPPDATA%\ExpanderCraft` to `%LOCALAPPDATA%\Mantice` (copied, if the old program is still running), the startup entry is renamed, and the installer replaces an installed ExpanderCraft, removing its executable and shortcuts. The portable edition is not touched: extract the new package over the old folder, or into a new one, and delete `expandercraft.exe`.
- **macOS is no longer a build target.** The program is Windows-only: the macOS backend, its dependencies and the macOS build of the `build` workflow are gone.

## 0.6.0 — 20 November 2026

- **Windows installer** (Inno Setup), attached to every release next to the portable zip. It installs for the current user without administrator rights (or for all users, if you choose so), adds a Start menu entry and an optional desktop shortcut, registers an uninstaller, and is available in English, Italian, French and Spanish. Updating is a matter of running the new installer. Your combos and settings are never removed by the uninstaller.
- The `build` workflow also builds the installer on every run and keeps it among the artifacts.

## 0.5.1 — 17 November 2026

- On Windows, ExpanderCraft draws the title bar itself: it always has the colors of the app theme, light or dark, whatever the Windows theme is. It has Windows 11 style buttons; the window moves by dragging the bar, maximizes with a double click and resizes from its edges and corners. Aero Snap, Win+arrow keys and Alt+Space work as before.
- The combo editor opens wider.

## 0.5.0 — 13 November 2026

- The title bar of the main window, the combo editor and the `#{input:}` prompt follows the theme chosen in the preferences (light or dark) instead of the one of the operating system, also at startup. With the "System" theme nothing changes.
- The combo editor (new combo and edit) opens in its own window instead of on top of the main window. The main window stays dimmed while the editor is open; a click on it brings the editor back to the front.

## 0.4.0 — 11 November 2026

- Only files in the format of the latest Beeftext (version 10, the one ExpanderCraft has always written) are read. Formats 1–9 of older Beeftext versions are no longer converted, and the conversion of old HTML snippets is gone with them: export such a file from an up-to-date Beeftext first. The migration dialog on first launch now says so, instead of disabling the Import button without an explanation.

## 0.3.0 — 11 November 2026

- Removed the cheat sheet (*Tools → Generate cheat sheet*), both the PDF and the CSV. Exporting combos to JSON and CSV is unchanged.
- `#{key:}` also accepts the short key names (`#{key:pgup}`, `#{key:del}`, `#{key:print}`), the same ones shortcuts use.
- Fewer dependencies: without the cheat sheet and without link support (never used) the program builds from 30 fewer packages (220 → 190 on Windows) and is about 1 MB smaller.

## 0.2.2 — 10 November 2026

- **Security**: text inserted by `#{clipboard}`, `#{input:}`, `#{envVar:}` and `#{powershell:}` is no longer interpreted. Before, copied text containing `#{shortcut:Win+R}…#{key:enter}` could press keys and run commands.
- A combo list that cannot be read at startup is no longer replaced by an empty list, and it is not overwritten until you restore it. The message mentions the backup copy only if the copy was really made.
- List in a shared folder (OneDrive, Dropbox…): changes made on another computer are reloaded within a few seconds. If the file changed while you were editing a combo, ExpanderCraft reloads it and warns you, instead of overwriting the changes of the other computer.
- An error during a substitution no longer turns expansion off until the next restart. A huge date shift (`#{dateTime:+100000000d:}`) is ignored.
- Importing an invalid JSON file (for instance one from a newer version) shows the error, instead of importing fragments of JSON as if it were a CSV.
- When the picker shortcut is pressed twice, the chosen combo still reaches the application you started from.
- An invalid value in the preferences resets only that value, not all the preferences (the list folder included). The original file is kept as `settings.broken.json`.
- Lighter interface: the Beeftext import dialog no longer re-reads the file on every redraw, and the picker and the table sort combos with less work.
- Code cleanup: removed the `rand` dependency and the code left unused by the Linux backend.

## 0.2.1 — 10 November 2026

- By default the interface uses the system language, and English when the system language is not translated.
- The group created at first launch, if it still has the default name ("Default Group"), is renamed in the chosen language.
- With the space as trigger, the space that completes a keyword no longer reaches the application: ExpanderCraft holds it back and erases only the keyword. In the Windows 11 Notepad the space started spell checking and autocorrect, and the combo sometimes came out as a plain "v".
- The substitution waits until you have released Ctrl, Alt, AltGr, Shift and Win before sending keys: with keywords ending in a character typed with AltGr (`@@`), the Windows 11 Notepad could enter its shortcut-key mode and lose the paste.
- Windows: in the Windows 11 Notepad, and in other recent apps, some combos appeared as a plain "v" instead of the text. The paste shortcut (and those of `#{shortcut:}`) now holds Ctrl long enough even when the app is late, for instance right after the space that triggers the combo.
- Windows: in the Windows 11 Notepad, combos starting with a punctuation mark (`.cf`) sometimes left the first character behind. Notepad dropped a backspace when they all arrived together: ExpanderCraft now sends them 15 ms apart.
- With loose matching the longest keyword wins: typing `@@@` triggers the combo `@@@`, not `@@`.
- Italian translation: "shortcode" replaces "codici delle emoji", and the preferences checkbox now reads "Usa gli shortcode delle emoji".

## 0.2.0 — 10 November 2026

- **Translated interface** in English (the default), Italian, French and Spanish, with the correct plural forms for each language. The language is chosen in Preferences › Appearance, "System language" included, and changes immediately, tray menu too.
- **PDF cheat sheet**: name, keyword and text of every combo, group by group, on a print-ready A4 page. The CSV is still available.
- **Emoji list** inside the program (*Help → Emoji list* or Preferences › Emojis): search by shortcode, name or category; one click copies the shortcode.
- Italian translation reviewed: "combo" is feminine everywhere, plus a few clearer wordings.
- Error messages about the combo file and about launching at login are translated.
- macOS: text put on the clipboard during a substitution stays out of the clipboard history.
- Linux is temporarily out of the main branch; the work on X11 and Wayland continues in the `linux-wayland` branch.

## 0.1.0 — 9 November 2026

- First version: a Beeftext clone in Rust for Windows, with the macOS backend already compiling.
