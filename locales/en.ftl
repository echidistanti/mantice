# Mantice user interface: English.
# Message ids are shared by all the locales; en.ftl is the reference.
# Plural forms follow the CLDR rules of the language ([one], [other]...).
# To add a language, copy this file to xx.ftl, translate the values and register it in src/i18n.rs.

## General

app-default-group = Default Group
app-error = Error
app-warning = Warning
app-information = Information
app-confirmation = Confirmation
app-yes = Yes
app-ok = OK
app-cancel = Cancel
app-browse = Browse...
app-reset = Reset
app-clear = Clear
app-optional = Optional
app-enabled = Enabled
app-default-with-value = Default ({ $value })
app-portable-edition = Portable edition
app-status-counts =
    { $combos ->
        [one] { $combos } combo
       *[other] { $combos } combos
    } · { $groups ->
        [one] { $groups } group
       *[other] { $groups } groups
    }

## Main window: menus

menu-file = File
label-combo = Combo
label-group = Group
menu-tools = Tools
menu-help = Help
menu-import = Import combos...
menu-export-all = Export all combos...
menu-export-selected = Export selected combos...
menu-preferences = Preferences...
menu-hide-window = Hide window
menu-quit = Quit
menu-new-group = New group...
menu-edit-group = Edit group...
menu-toggle-group = Enable / disable group
menu-delete-group = Delete group...
menu-backup = Back up combo list...
menu-restore = Restore combo list...
menu-restore-auto = Restore an automatic backup...
menu-open-data-folder = Open data folder
menu-open-log = Open log file
menu-variables-reference = Variables reference
menu-about = About Mantice

## Main window: combo actions

action-new-combo = New combo...
action-edit = Edit...
action-duplicate = Duplicate
action-copy-snippet = Copy snippet to clipboard
action-enable = Enable
action-disable = Disable
action-matching-mode = Matching mode
action-case-sensitivity = Case sensitivity
action-move-to-group = Move to group
action-delete = Delete...

## Main window: groups, table, status bar

groups-title = Groups
groups-all = All combos
group-enable = Enable group
group-disable = Disable group
group-move-up = Move up
group-move-down = Move down
combos-search = Search combos...
label-new-combo = New combo
label-name = Name
label-keyword = Keyword
label-snippet = Snippet
column-last-used = Last used
combos-empty-title = No combo yet
combos-empty-text = A combo associates a short keyword to a snippet. Type the keyword in any application and it is replaced by the snippet.
combos-create-first = Create your first combo
combos-no-match = No combo matches the search.
status-active = Active
status-paused = Paused
status-toggle-hint = Click to enable or disable Mantice

## Matching and case sensitivity

matching-strict = Strict
matching-loose = Loose
case-sensitive = Case sensitive
case-insensitive = Case insensitive

## Main window: confirmations and messages

confirm-delete-combos =
    { $count ->
        [one] Delete the selected combo?
       *[other] Delete the { $count } selected combos?
    }
confirm-delete-group = Delete the group “{ $name }”?
confirm-delete-group-with-combos =
    { $count ->
        [one] Delete the group “{ $name }” and the combo it contains?
       *[other] Delete the group “{ $name }” and the { $count } combos it contains?
    }
confirm-replace-library = All the existing combos and groups will be replaced by the content of the backup file. Continue?
msg-last-group = The last group cannot be deleted.
filter-combo-files = Combo files
filter-all-files = All files
filter-csv-textexpander = CSV (TextExpander compatible)
file-name-backup = Mantice backup { $date }
msg-import-failed = The file could not be imported.
msg-exported = The combos were exported.
msg-export-failed = The combos could not be exported.
msg-backup-saved = The backup was saved.
msg-load-failed = The file could not be loaded.
msg-combos-restored =
    { $count ->
        [one] { $count } combo was restored.
       *[other] { $count } combos were restored.
    }
msg-combos-imported =
    { $count ->
        [one] { $count } combo was imported.
       *[other] { $count } combos were imported.
    }
msg-backups-removed =
    { $count ->
        [one] { $count } backup file was removed.
       *[other] { $count } backup files were removed.
    }
msg-library-load-failed = The combo list could not be loaded.
msg-library-copy-saved = A copy of the file was saved as: { $path }
msg-library-save-failed = The combo list could not be saved.
msg-library-reloaded = The combo list was changed by another program or computer, and it has been reloaded. Your last change was not saved: please make it again.
msg-invalid-combos-ignored =
    { $count ->
        [one] { $count } invalid combo was ignored while loading the combo list.
       *[other] { $count } invalid combos were ignored while loading the combo list.
    }

## Combo editor

editor-edit-title = Edit combo
editor-keyword-hint = e.g. btsig
editor-matching = Matching
editor-description = Description
editor-combo-enabled = Enabled
editor-snippet-hint = The text that replaces the keyword
editor-snippet-empty = The snippet cannot be empty.
editor-warn-empty-keyword = Without a keyword, the combo can only be inserted with the combo picker.
editor-warn-short-keyword = Short keywords may be triggered by mistake.
editor-warn-duplicate-keyword = This keyword is also used by “{ $name }”: one of them will be chosen at random.
editor-insert-variable = Insert variable
editor-recording-shortcut = Press the shortcut to insert (Esc to cancel)...
var-clipboard = Clipboard content
var-cursor = Cursor position
var-date = Date
var-time = Time
var-date-time = Date and time
var-custom-date-time = Custom date/time
var-shifted-date-time = Shifted date/time
var-combo-upper = Combo in uppercase
var-combo-lower = Combo in lowercase
var-combo-trim = Combo, trimmed
var-input = User input
var-env = Environment variable
var-powershell = PowerShell script
var-delay = Delay (ms)
var-key = Key
var-shortcut-record = Shortcut... (record)

## Group editor

group-new-title = New group
group-edit-title = Edit group

## Import

import-title = Import combos
import-new-combos = New combos: { $count }
import-conflicts =
    { $count ->
        [one] { $count } combo has the same keyword as an existing combo.
       *[other] { $count } combos have the same keyword as an existing combo.
    }
import-skip = Skip the conflicting combos
import-overwrite = Overwrite the existing combos
import-overwrite-older = Overwrite only the older existing combos ({ $count })
import-into-group = Import into group
import-button = Import ({ $count })
import-action = Import

## Backups

restore-auto-title = Restore an automatic backup
restore-auto-info = A backup of the combo list is made every time it is saved.
restore-no-backup = No backup available.
restore-remove-all = Remove all backups...
restore-action = Restore
confirm-remove-backups = Remove all the automatic backups?

## Application lists

applist-excluded-title = Excluded applications
applist-excluded-help = Mantice does not perform any substitution in these applications.
applist-sensitive-title = Sensitive applications
applist-sensitive-help = These applications do not support pasting with the keyboard: snippets are typed character by character instead.
applist-emoji-title = Applications excluded from emoji substitution
applist-emoji-help = Emoji shortcodes are not replaced in these applications.
applist-format = One process name per line (e.g. notepad.exe). Wildcards * and ? are accepted.
applist-example = Example of a process name: { $name }

## Beeftext migration

migration-title = Beeftext library found
migration-text = A Beeftext combo list was found on this computer. Mantice can import it with all its groups.

## About

about-version = Version { $version }
about-tagline = A text expander written in Rust. Type a keyword, get a snippet.
about-credits-beeftext = Inspired by Beeftext by Xavier Michelon (MIT License). Combo files are compatible with Beeftext.
about-credits-emoji = Emoji shortcodes from gemoji (MIT License).
about-data-folder = Data folder: { $path }

## Variables reference

help-intro = Variables are evaluated each time a combo is triggered. Use { $escaped_brace } and { $escaped_backslash } to write { $brace } and { $backslash } inside a variable.
help-clipboard = The text content of the clipboard.
help-cursor = Position of the text cursor after the substitution.
help-date = Current date and/or time, in the format of the system.
help-custom-format = Current date/time with a custom format (d dd ddd dddd M MM MMM MMMM yy yyyy h hh H HH m mm s ss zzz AP w ww).
help-shift = Shifted date/time: y M w d h m s z units, the sign is mandatory. The format can be empty; the last colon is mandatory.
help-combo = The snippet of another combo (also { $upper }, { $lower } and { $trim }).
help-input = Asks the user for a value. The same description is only asked once.
help-env = The value of an environment variable.
help-powershell = The output of a PowerShell script (10 s timeout by default, 0 means no timeout).
help-key = Presses a key (tab, enter, up, f5...), optionally several times.
help-shortcut = Presses a keyboard shortcut.
help-delay = Pauses the substitution.

## Preferences

prefs-title = Preferences
prefs-tab-behavior = Behavior
prefs-tab-appearance = Appearance
prefs-tab-combos = Combos
prefs-tab-emojis = Emojis
prefs-tab-advanced = Advanced
prefs-section-general = General
prefs-enabled = Mantice is enabled
prefs-autostart = Launch Mantice at login
prefs-portable-unavailable = Not available in the portable edition.
prefs-play-sound = Play a sound when a combo is triggered
prefs-custom-sound = Custom sound
prefs-section-substitution = Substitution
prefs-automatic = Automatic: combos are triggered as soon as the keyword is typed
prefs-manual = Manual: type the keyword, then press the trigger shortcut
prefs-trigger-shortcut = Trigger shortcut
prefs-trigger-on-space = Trigger combos with a space typed after the keyword
prefs-keep-space = Keep the space after the snippet
prefs-section-picker = Combo picker
prefs-enable-picker = Enable the combo picker
prefs-shortcut = Shortcut
prefs-section-toggle = Enable / disable shortcut
prefs-use-toggle = Use a shortcut to enable or disable Mantice
prefs-recording = Press a shortcut... (Esc to cancel)
prefs-record-hint = Click, then press the new shortcut
prefs-section-language = Language
prefs-system-language = System language ({ $language })
prefs-section-theme = Theme
prefs-theme-system = Follow the system
prefs-theme-light = Light
prefs-theme-dark = Dark
prefs-section-defaults = Defaults for new combos
prefs-matching-help = Strict: the typed text must be exactly the keyword. Loose: the combo can be triggered in the middle of a word.
prefs-section-warnings = Warnings
prefs-warn-short = Warn about keywords shorter than 3 characters
prefs-warn-empty = Warn about empty keywords
prefs-section-combo-file = Combo list file
prefs-change-folder = Change folder...
prefs-default-folder = Use the default folder
prefs-sync-folder-hint = A synchronized folder (OneDrive, Dropbox...) lets you share your combos between computers.
msg-combo-list-moved =
    The combo list is now saved in:
    { $path }
prefs-section-emoji = Emoji shortcodes
prefs-emoji-enable = Use emoji shortcodes
prefs-left-delimiter = Left delimiter
prefs-right-delimiter = Right delimiter
prefs-example = Example:
prefs-emoji-list = Emoji list...
prefs-excluded-apps = Excluded applications...
prefs-sensitive-apps = Sensitive applications...
prefs-show-emojis-in-picker = Show emojis in the combo picker (after 3 characters)
prefs-section-text-insertion = Text insertion
prefs-restore-clipboard = Restore the clipboard after a substitution
prefs-shift-insert = Paste with Shift+Insert instead of Ctrl+V
prefs-keystroke-delay = Delay between keystrokes when typing (ms)
prefs-section-backup = Backup
prefs-auto-backup = Back up the combo list automatically
prefs-custom-location = Custom location
prefs-custom-executable = Custom executable
prefs-powershell-hint = For instance pwsh.exe (PowerShell 7) instead of Windows PowerShell.
prefs-section-diagnostics = Diagnostics
prefs-write-log = Write a log file
prefs-reset = Reset preferences...
prefs-confirm-reset = Reset all preferences to their default values?
msg-autostart-failed = The automatic startup could not be changed.

## Combo picker and input dialog

picker-title = Mantice - Combo picker
picker-search = Search a combo...
picker-no-match = No matching combo.
picker-help = ↑↓ to select · Enter to insert · Esc to close

## System tray

tray-open = Open Mantice
tray-tooltip-paused = Mantice (paused)

## Errors

error-open-file = Could not open the file “{ $path }”.
error-write-file = Could not write the file “{ $path }”.
error-not-json = The file is not valid JSON.
error-invalid-list = The combo list file is invalid.
error-no-version = The combo list file does not specify its version number.
error-newer-version = The combo list file was created by a newer version of the application.
error-older-version = The file uses an old Beeftext format (version { $version }). Export the combos from the latest version of Beeftext and import that file.
error-invalid-groups = The list of groups is not valid.
error-invalid-group = The group list contains an invalid group.
error-invalid-combos = The list of combos is not valid.
error-nothing-to-import = The file does not contain importable data.
error-registry = The Windows registry could not be updated (error { $code }).
platform-hook-failed = The keyboard hook could not be installed: substitutions are disabled.

## Key names (shown in shortcuts)

key-shift = Shift
key-enter = Enter
key-space = Space
key-backspace = Backspace
key-escape = Esc
key-insert = Ins
key-delete = Del
key-home = Home
key-end = End
key-page-up = PgUp
key-page-down = PgDown
key-left = Left
key-right = Right
key-up = Up
key-down = Down
key-print = Print

## Emoji list

menu-emoji-list = Emoji list
emoji-list-title = Emoji shortcodes
emoji-list-intro = Type the code between the delimiters, for instance { $example }. Names are in English.
emoji-list-disabled = Emoji substitution is disabled: enable it in Preferences › Emojis.
emoji-list-search = Search by code or name...
emoji-list-count =
    { $count ->
        [one] { $count } emoji
       *[other] { $count } emojis
    }
emoji-list-no-match = No emoji matches the search.
emoji-list-click-hint = Click a row to copy its code.
emoji-list-copied = Copied: { $code }
emoji-group-all = All categories
emoji-group-smileys = Smileys & emotion
emoji-group-people = People & body
emoji-group-animals = Animals & nature
emoji-group-food = Food & drink
emoji-group-travel = Travel & places
emoji-group-activities = Activities
emoji-group-objects = Objects
emoji-group-symbols = Symbols
emoji-group-flags = Flags
