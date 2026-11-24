# Mantice user interface: Italian.
# Message ids are shared by all the locales; en.ftl is the reference.
# Plural forms follow the CLDR rules of the language ([one], [other]...).

## General

app-default-group = Gruppo predefinito
app-error = Errore
app-warning = Attenzione
app-information = Informazione
app-confirmation = Conferma
app-yes = Sì
app-ok = OK
app-cancel = Annulla
app-browse = Sfoglia...
app-reset = Ripristina
app-clear = Cancella
app-optional = Facoltativo
app-enabled = Attivo
app-default-with-value = Valore predefinito ({ $value })
app-portable-edition = Edizione portatile
app-status-counts =
    { $combos } combo · { $groups ->
        [one] { $groups } gruppo
       *[other] { $groups } gruppi
    }

## Main window: menus

menu-file = File
label-combo = Combo
label-group = Gruppo
menu-tools = Strumenti
menu-help = Aiuto
menu-import = Importa combo...
menu-export-all = Esporta tutte le combo...
menu-export-selected = Esporta le combo selezionate...
menu-preferences = Preferenze...
menu-hide-window = Nascondi finestra
menu-quit = Esci
menu-new-group = Nuovo gruppo...
menu-edit-group = Modifica gruppo...
menu-toggle-group = Attiva / disattiva gruppo
menu-delete-group = Elimina gruppo...
menu-backup = Salva una copia della lista...
menu-restore = Ripristina la lista delle combo...
menu-restore-auto = Ripristina un backup automatico...
menu-open-data-folder = Apri la cartella dei dati
menu-open-log = Apri il file di log
menu-variables-reference = Guida alle variabili
menu-about = Informazioni su Mantice

## Main window: combo actions

action-new-combo = Nuova combo...
action-edit = Modifica...
action-duplicate = Duplica
action-copy-snippet = Copia il testo negli appunti
action-enable = Attiva
action-disable = Disattiva
action-matching-mode = Corrispondenza
action-case-sensitivity = Maiuscole/minuscole
action-move-to-group = Sposta nel gruppo
action-delete = Elimina...

## Main window: groups, table, status bar

groups-title = Gruppi
groups-all = Tutte le combo
group-enable = Attiva gruppo
group-disable = Disattiva gruppo
group-move-up = Sposta su
group-move-down = Sposta giù
combos-search = Cerca nelle combo...
label-new-combo = Nuova combo
label-name = Nome
label-keyword = Parola chiave
label-snippet = Testo
column-last-used = Ultimo uso
combos-empty-title = Ancora nessuna combo
combos-empty-text = Una combo associa una parola chiave breve a un testo. Digita la parola chiave in qualsiasi applicazione e verrà sostituita dal testo.
combos-create-first = Crea la prima combo
combos-no-match = Nessuna combo corrisponde alla ricerca.
status-active = Attivo
status-paused = In pausa
status-toggle-hint = Clicca per attivare o disattivare Mantice

## Matching and case sensitivity

matching-strict = Esatta
matching-loose = Libera
case-sensitive = Distingue maiuscole/minuscole
case-insensitive = Ignora maiuscole/minuscole

## Main window: confirmations and messages

confirm-delete-combos =
    { $count ->
        [one] Eliminare la combo selezionata?
       *[other] Eliminare le { $count } combo selezionate?
    }
confirm-delete-group = Eliminare il gruppo “{ $name }”?
confirm-delete-group-with-combos =
    { $count ->
        [one] Eliminare il gruppo “{ $name }” e la combo che contiene?
       *[other] Eliminare il gruppo “{ $name }” e le { $count } combo che contiene?
    }
confirm-replace-library = Tutte le combo e tutti i gruppi attuali verranno sostituiti dal contenuto del file di backup. Continuare?
msg-last-group = L'ultimo gruppo non si può eliminare.
filter-combo-files = File di combo
filter-all-files = Tutti i file
filter-csv-textexpander = CSV (compatibile con TextExpander)
file-name-backup = Backup Mantice { $date }
msg-import-failed = Impossibile importare il file.
msg-exported = Combo esportate.
msg-export-failed = Impossibile esportare le combo.
msg-backup-saved = Backup salvato.
msg-load-failed = Impossibile caricare il file.
msg-combos-restored =
    { $count ->
        [one] { $count } combo ripristinata.
       *[other] { $count } combo ripristinate.
    }
msg-combos-imported =
    { $count ->
        [one] { $count } combo importata.
       *[other] { $count } combo importate.
    }
msg-backups-removed =
    { $count ->
        [one] { $count } file di backup eliminato.
       *[other] { $count } file di backup eliminati.
    }
msg-library-load-failed = Impossibile caricare la lista delle combo.
msg-library-copy-saved = Una copia del file è stata salvata come: { $path }
msg-library-save-failed = Impossibile salvare la lista delle combo.
msg-library-reloaded = La lista delle combo è stata modificata da un altro programma o computer ed è stata ricaricata. L'ultima modifica non è stata salvata: rifalla.
msg-invalid-combos-ignored =
    { $count ->
        [one] Durante il caricamento è stata ignorata { $count } combo non valida.
       *[other] Durante il caricamento sono state ignorate { $count } combo non valide.
    }

## Combo editor

editor-edit-title = Modifica combo
editor-keyword-hint = es. ffirma
editor-matching = Corrispondenza
editor-description = Descrizione
editor-combo-enabled = Attiva
editor-snippet-hint = Il testo che sostituisce la parola chiave
editor-snippet-empty = Il testo non può essere vuoto.
editor-warn-empty-keyword = Senza parola chiave, la combo si può inserire solo dal selettore.
editor-warn-short-keyword = Le parole chiave brevi possono scattare per errore.
editor-warn-duplicate-keyword = Anche “{ $name }” usa questa parola chiave: ne verrà scelta una a caso.
editor-insert-variable = Inserisci variabile
editor-recording-shortcut = Premi la scorciatoia da inserire (Esc per annullare)...
var-clipboard = Contenuto degli appunti
var-cursor = Posizione del cursore
var-date = Data
var-time = Ora
var-date-time = Data e ora
var-custom-date-time = Data/ora personalizzata
var-shifted-date-time = Data/ora spostata
var-combo-upper = Combo in maiuscolo
var-combo-lower = Combo in minuscolo
var-combo-trim = Combo senza spazi ai bordi
var-input = Valore chiesto all'utente
var-env = Variabile d'ambiente
var-powershell = Script PowerShell
var-delay = Pausa (ms)
var-key = Tasto
var-shortcut-record = Scorciatoia... (registra)

## Group editor

group-new-title = Nuovo gruppo
group-edit-title = Modifica gruppo

## Import

import-title = Importa combo
import-new-combos = Nuove combo: { $count }
import-conflicts =
    { $count ->
        [one] { $count } combo ha la stessa parola chiave di una combo esistente.
       *[other] { $count } combo hanno la stessa parola chiave di una combo esistente.
    }
import-skip = Salta le combo in conflitto
import-overwrite = Sovrascrivi le combo esistenti
import-overwrite-older = Sovrascrivi solo le combo esistenti più vecchie ({ $count })
import-into-group = Importa nel gruppo
import-button = Importa ({ $count })
import-action = Importa

## Backups

restore-auto-title = Ripristina un backup automatico
restore-auto-info = Ogni salvataggio della lista delle combo crea una copia di sicurezza.
restore-no-backup = Nessun backup disponibile.
restore-remove-all = Elimina tutti i backup...
restore-action = Ripristina
confirm-remove-backups = Eliminare tutti i backup automatici?

## Application lists

applist-excluded-title = Applicazioni escluse
applist-excluded-help = Mantice non sostituisce nulla in queste applicazioni.
applist-sensitive-title = Applicazioni sensibili
applist-sensitive-help = Queste applicazioni non permettono di incollare da tastiera: il testo viene digitato carattere per carattere.
applist-emoji-title = Applicazioni escluse dalla sostituzione delle emoji
applist-emoji-help = In queste applicazioni gli shortcode delle emoji non vengono sostituiti.
applist-format = Un nome di processo per riga (per esempio notepad.exe). Sono ammessi i caratteri jolly * e ?.
applist-example = Esempio di nome di processo: { $name }

## Beeftext migration

migration-title = Trovata una libreria di Beeftext
migration-text = Su questo computer c'è una lista di combo di Beeftext. Mantice può importarla con tutti i suoi gruppi.

## About

about-version = Versione { $version }
about-tagline = Un espansore di testo scritto in Rust. Digiti una parola chiave, ottieni un testo.
about-credits-beeftext = Ispirato a Beeftext di Xavier Michelon (licenza MIT). I file delle combo sono compatibili con Beeftext.
about-credits-emoji = Shortcode delle emoji da gemoji (licenza MIT).
about-data-folder = Cartella dei dati: { $path }

## Variables reference

help-intro = Le variabili vengono valutate ogni volta che scatta una combo. Usa { $escaped_brace } e { $escaped_backslash } per scrivere { $brace } e { $backslash } dentro una variabile.
help-clipboard = Il testo contenuto negli appunti.
help-cursor = Dove si trova il cursore dopo la sostituzione.
help-date = Data e/o ora correnti, nel formato del sistema.
help-custom-format = Data/ora correnti con un formato personalizzato (d dd ddd dddd M MM MMM MMMM yy yyyy h hh H HH m mm s ss zzz AP w ww).
help-shift = Data/ora spostata: unità y M w d h m s z, il segno è obbligatorio. Il formato può restare vuoto; i due punti finali sono obbligatori.
help-combo = Il testo di un'altra combo (anche { $upper }, { $lower } e { $trim }).
help-input = Chiede un valore all'utente. La stessa descrizione viene chiesta una volta sola.
help-env = Il valore di una variabile d'ambiente.
help-powershell = L'output di uno script PowerShell (timeout predefinito di 10 s, 0 lo disattiva).
help-key = Preme un tasto (tab, enter, up, f5...), anche più volte.
help-shortcut = Preme una scorciatoia da tastiera.
help-delay = Mette in pausa la sostituzione.

## Preferences

prefs-title = Preferenze
prefs-tab-behavior = Comportamento
prefs-tab-appearance = Aspetto
prefs-tab-combos = Combo
prefs-tab-emojis = Emoji
prefs-tab-advanced = Avanzate
prefs-section-general = Generale
prefs-enabled = Mantice è attivo
prefs-autostart = Avvia Mantice all'accesso
prefs-portable-unavailable = Non disponibile nell'edizione portatile.
prefs-play-sound = Riproduci un suono quando scatta una combo
prefs-custom-sound = Suono personalizzato
prefs-section-substitution = Sostituzione
prefs-automatic = Automatica: la combo scatta appena digiti la parola chiave
prefs-manual = Manuale: digita la parola chiave, poi premi la scorciatoia
prefs-trigger-shortcut = Scorciatoia di attivazione
prefs-trigger-on-space = Fai scattare le combo con uno spazio dopo la parola chiave
prefs-keep-space = Mantieni lo spazio dopo il testo
prefs-section-picker = Selettore delle combo
prefs-enable-picker = Attiva il selettore delle combo
prefs-shortcut = Scorciatoia
prefs-section-toggle = Scorciatoia per attivare / disattivare
prefs-use-toggle = Usa una scorciatoia per attivare o disattivare Mantice
prefs-recording = Premi una scorciatoia... (Esc per annullare)
prefs-record-hint = Clicca, poi premi la nuova scorciatoia
prefs-section-language = Lingua
prefs-system-language = Lingua del sistema ({ $language })
prefs-section-theme = Tema
prefs-theme-system = Come il sistema
prefs-theme-light = Chiaro
prefs-theme-dark = Scuro
prefs-section-defaults = Valori predefiniti delle nuove combo
prefs-matching-help = Esatta: il testo digitato deve coincidere con la parola chiave. Libera: la combo può scattare anche in mezzo a una parola.
prefs-section-warnings = Avvisi
prefs-warn-short = Avvisa se la parola chiave ha meno di 3 caratteri
prefs-warn-empty = Avvisa se la parola chiave è vuota
prefs-section-combo-file = File della lista delle combo
prefs-change-folder = Cambia cartella...
prefs-default-folder = Usa la cartella predefinita
prefs-sync-folder-hint = Con una cartella sincronizzata (OneDrive, Dropbox...) puoi condividere le combo tra più computer.
msg-combo-list-moved =
    Ora la lista delle combo viene salvata in:
    { $path }
prefs-section-emoji = Shortcode delle emoji
prefs-emoji-enable = Usa gli shortcode delle emoji
prefs-left-delimiter = Delimitatore sinistro
prefs-right-delimiter = Delimitatore destro
prefs-example = Esempio:
prefs-emoji-list = Elenco delle emoji...
prefs-excluded-apps = Applicazioni escluse...
prefs-sensitive-apps = Applicazioni sensibili...
prefs-show-emojis-in-picker = Mostra le emoji nel selettore (dopo 3 caratteri)
prefs-section-text-insertion = Inserimento del testo
prefs-restore-clipboard = Ripristina gli appunti dopo una sostituzione
prefs-shift-insert = Incolla con Maiusc+Ins invece di Ctrl+V
prefs-keystroke-delay = Pausa tra i tasti durante la digitazione (ms)
prefs-section-backup = Backup
prefs-auto-backup = Salva automaticamente una copia della lista delle combo
prefs-custom-location = Percorso personalizzato
prefs-custom-executable = Eseguibile personalizzato
prefs-powershell-hint = Per esempio pwsh.exe (PowerShell 7) al posto di Windows PowerShell.
prefs-section-diagnostics = Diagnostica
prefs-write-log = Scrivi un file di log
prefs-reset = Ripristina le preferenze...
prefs-confirm-reset = Riportare tutte le preferenze ai valori predefiniti?
msg-autostart-failed = Impossibile modificare l'avvio automatico.

## Combo picker and input dialog

picker-title = Mantice - Selettore delle combo
picker-search = Cerca una combo...
picker-no-match = Nessuna combo corrispondente.
picker-help = ↑↓ per scegliere · Invio per inserire · Esc per chiudere

## System tray

tray-open = Apri Mantice
tray-tooltip-paused = Mantice (in pausa)

## Errors

error-open-file = Impossibile aprire il file “{ $path }”.
error-write-file = Impossibile scrivere il file “{ $path }”.
error-not-json = Il file non è un JSON valido.
error-invalid-list = Il file della lista delle combo non è valido.
error-no-version = Il file della lista delle combo non indica il numero di versione.
error-newer-version = Il file della lista delle combo è stato creato da una versione più recente dell'applicazione.
error-older-version = Il file usa un vecchio formato di Beeftext (versione { $version }). Esportare le combo dall'ultima versione di Beeftext e importare quel file.
error-invalid-groups = L'elenco dei gruppi non è valido.
error-invalid-group = L'elenco dei gruppi contiene un gruppo non valido.
error-invalid-combos = L'elenco delle combo non è valido.
error-nothing-to-import = Il file non contiene dati importabili.
error-registry = Impossibile aggiornare il registro di Windows (errore { $code }).
platform-hook-failed = Impossibile intercettare la tastiera: le sostituzioni sono disattivate.

## Key names (shown in shortcuts)

key-shift = Maiusc
key-enter = Invio
key-space = Spazio
key-backspace = Backspace
key-escape = Esc
key-insert = Ins
key-delete = Canc
key-home = Home
key-end = Fine
key-page-up = PgSu
key-page-down = PgGiù
key-left = Sinistra
key-right = Destra
key-up = Su
key-down = Giù
key-print = Stamp

## Emoji list

menu-emoji-list = Elenco delle emoji
emoji-list-title = Shortcode delle emoji
emoji-list-intro = Scrivi lo shortcode tra i delimitatori, per esempio { $example }. I nomi sono in inglese.
emoji-list-disabled = La sostituzione delle emoji è disattivata: attivala in Preferenze › Emoji.
emoji-list-search = Cerca per shortcode o nome...
emoji-list-count = { $count } emoji
emoji-list-no-match = Nessuna emoji corrisponde alla ricerca.
emoji-list-click-hint = Clicca una riga per copiarne lo shortcode.
emoji-list-copied = Copiato: { $code }
emoji-group-all = Tutte le categorie
emoji-group-smileys = Faccine ed emozioni
emoji-group-people = Persone e corpo
emoji-group-animals = Animali e natura
emoji-group-food = Cibo e bevande
emoji-group-travel = Viaggi e luoghi
emoji-group-activities = Attività
emoji-group-objects = Oggetti
emoji-group-symbols = Simboli
emoji-group-flags = Bandiere
