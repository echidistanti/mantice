# Mantice user interface: French.
# Message ids are shared by all the locales; en.ftl is the reference.
# Plural forms follow the CLDR rules of the language ([one], [other]...).

## General

app-default-group = Groupe par défaut
app-error = Erreur
app-warning = Avertissement
app-information = Information
app-confirmation = Confirmation
app-yes = Oui
app-ok = OK
app-cancel = Annuler
app-browse = Parcourir...
app-reset = Réinitialiser
app-clear = Effacer
app-optional = Facultatif
app-enabled = Activé
app-default-with-value = Par défaut ({ $value })
app-portable-edition = Édition portable
app-status-counts =
    { $combos ->
        [one] { $combos } combo
       *[other] { $combos } combos
    } · { $groups ->
        [one] { $groups } groupe
       *[other] { $groups } groupes
    }

## Main window: menus

menu-file = Fichier
label-combo = Combo
label-group = Groupe
menu-tools = Outils
menu-help = Aide
menu-import = Importer des combos...
menu-import-espanso = Importer depuis espanso...
menu-export-all = Exporter tous les combos...
menu-export-selected = Exporter les combos sélectionnés...
menu-preferences = Préférences...
menu-hide-window = Masquer la fenêtre
menu-quit = Quitter
menu-new-group = Nouveau groupe...
menu-edit-group = Modifier le groupe...
menu-toggle-group = Activer / désactiver le groupe
menu-delete-group = Supprimer le groupe...
menu-backup = Sauvegarder la liste des combos...
menu-restore = Restaurer la liste des combos...
menu-restore-auto = Restaurer une sauvegarde automatique...
menu-open-data-folder = Ouvrir le dossier des données
menu-open-log = Ouvrir le fichier journal
menu-variables-reference = Référence des variables
menu-about = À propos de Mantice

## Main window: combo actions

action-new-combo = Nouveau combo...
action-edit = Modifier...
action-duplicate = Dupliquer
action-copy-snippet = Copier le snippet dans le presse-papiers
action-enable = Activer
action-disable = Désactiver
action-matching-mode = Mode de déclenchement
action-case-sensitivity = Sensibilité à la casse
action-move-to-group = Déplacer vers le groupe
action-delete = Supprimer...

## Main window: groups, table, status bar

groups-title = Groupes
groups-all = Tous les combos
group-enable = Activer le groupe
group-disable = Désactiver le groupe
group-move-up = Monter
group-move-down = Descendre
combos-search = Rechercher des combos...
label-new-combo = Nouveau combo
label-name = Nom
label-keyword = Mot-clé
label-snippet = Snippet
column-last-used = Dernière utilisation
combos-empty-title = Aucun combo pour l'instant
combos-empty-text = Un combo associe un mot-clé court à un snippet. Tapez le mot-clé dans n'importe quelle application : il est remplacé par le snippet.
combos-create-first = Créer votre premier combo
combos-no-match = Aucun combo ne correspond à la recherche.
status-active = Actif
status-paused = En pause
status-toggle-hint = Cliquer pour activer ou désactiver Mantice

## Matching and case sensitivity

matching-strict = Strict
matching-loose = Permissif
case-sensitive = Sensible à la casse
case-insensitive = Insensible à la casse

## Main window: confirmations and messages

confirm-delete-combos =
    { $count ->
        [one] Supprimer le combo sélectionné ?
       *[other] Supprimer les { $count } combos sélectionnés ?
    }
confirm-delete-group = Supprimer le groupe « { $name } » ?
confirm-delete-group-with-combos =
    { $count ->
        [one] Supprimer le groupe « { $name } » et le combo qu'il contient ?
       *[other] Supprimer le groupe « { $name } » et les { $count } combos qu'il contient ?
    }
confirm-replace-library = Tous les combos et groupes existants seront remplacés par le contenu du fichier de sauvegarde. Continuer ?
msg-last-group = Le dernier groupe ne peut pas être supprimé.
filter-combo-files = Fichiers de combos
filter-all-files = Tous les fichiers
filter-csv-textexpander = CSV (compatible TextExpander)
file-name-backup = Sauvegarde Mantice { $date }
msg-import-failed = Le fichier n'a pas pu être importé.
msg-exported = Les combos ont été exportés.
msg-export-failed = Les combos n'ont pas pu être exportés.
msg-backup-saved = La sauvegarde a été enregistrée.
msg-load-failed = Le fichier n'a pas pu être chargé.
msg-combos-restored =
    { $count ->
        [one] { $count } combo a été restauré.
       *[other] { $count } combos ont été restaurés.
    }
msg-combos-imported =
    { $count ->
        [one] { $count } combo a été importé.
       *[other] { $count } combos ont été importés.
    }
msg-backups-removed =
    { $count ->
        [one] { $count } fichier de sauvegarde a été supprimé.
       *[other] { $count } fichiers de sauvegarde ont été supprimés.
    }
msg-library-load-failed = La liste des combos n'a pas pu être chargée.
msg-library-copy-saved = Une copie du fichier a été enregistrée sous : { $path }
msg-library-save-failed = La liste des combos n'a pas pu être enregistrée.
msg-library-reloaded = La liste des combos a été modifiée par un autre programme ou ordinateur, et elle a été rechargée. Votre dernière modification n'a pas été enregistrée : refaites-la.
msg-invalid-combos-ignored =
    { $count ->
        [one] { $count } combo invalide a été ignoré lors du chargement de la liste.
       *[other] { $count } combos invalides ont été ignorés lors du chargement de la liste.
    }

## Combo editor

editor-edit-title = Modifier le combo
editor-keyword-hint = ex. : btsig
editor-matching = Déclenchement
editor-description = Description
editor-combo-enabled = Activé
editor-snippet-hint = Le texte qui remplace le mot-clé
editor-snippet-empty = Le snippet ne peut pas être vide.
editor-warn-empty-keyword = Sans mot-clé, le combo ne peut être inséré qu'avec le sélecteur de combos.
editor-warn-short-keyword = Les mots-clés courts peuvent se déclencher par erreur.
editor-warn-duplicate-keyword = Ce mot-clé est aussi utilisé par « { $name } » : l'un d'eux sera choisi au hasard.
editor-insert-variable = Insérer une variable
editor-recording-shortcut = Appuyez sur le raccourci à insérer (Échap pour annuler)...
var-clipboard = Contenu du presse-papiers
var-cursor = Position du curseur
var-date = Date
var-time = Heure
var-date-time = Date et heure
var-custom-date-time = Date/heure personnalisée
var-shifted-date-time = Date/heure décalée
var-combo-upper = Combo en majuscules
var-combo-lower = Combo en minuscules
var-combo-trim = Combo sans espaces aux extrémités
var-input = Saisie de l'utilisateur
var-env = Variable d'environnement
var-powershell = Script PowerShell
var-delay = Pause (ms)
var-key = Touche
var-shortcut-record = Raccourci... (enregistrer)

## Group editor

group-new-title = Nouveau groupe
group-edit-title = Modifier le groupe

## Import

import-title = Importer des combos
import-new-combos = Nouveaux combos : { $count }
import-conflicts =
    { $count ->
        [one] { $count } combo a le même mot-clé qu'un combo existant.
       *[other] { $count } combos ont le même mot-clé qu'un combo existant.
    }
import-skip = Ignorer les combos en conflit
import-overwrite = Remplacer les combos existants
import-overwrite-older = Remplacer uniquement les combos existants plus anciens ({ $count })
import-into-group = Importer dans le groupe
import-button = Importer ({ $count })
import-action = Importer

## Import of espanso packages

espanso-title = Importer un paquet espanso
espanso-help = Saisissez le nom d'un paquet du hub espanso (hub.espanso.org) ou collez son lien. Vous pouvez aussi choisir un fichier .zip ou .yml, ou un dossier, de cet ordinateur.
espanso-source-hint = Nom ou lien du paquet, par exemple basic-emojis
espanso-choose-file = Fichier...
espanso-choose-folder = Dossier...
espanso-file-filter = Paquets espanso
espanso-load = Charger
espanso-loading = Chargement du paquet...
espanso-by-author = par { $author }
espanso-new-group = Nouveau groupe
espanso-existing-group = Groupe existant
espanso-left-out = Ignorés : { $count }
espanso-approximated = Combos qui se comportent un peu différemment d'espanso : { $count }. Mantice tape toujours le texte tel qu'il est écrit, sans suivre la casse du mot-clé, et les dates utilisent sa propre langue.
espanso-duplicates = Déclencheurs répétés, importés une seule fois : { $count }
espanso-unreadable = Fichiers qui ne sont pas du YAML valide, ignorés : { $files }
espanso-large = Ce paquet est très volumineux : la liste des combos deviendra très longue.
espanso-reason-regex = Déclenché par une expression régulière
espanso-reason-no-trigger = Sans déclencheur
espanso-reason-form = Formulaire
espanso-reason-image = Image
espanso-reason-rich-text = Texte mis en forme (Markdown, HTML)
espanso-reason-command = Exécute une commande ou un script
espanso-reason-choice = Demande de choisir une valeur (choice, list, random)
espanso-reason-variable = Variable que Mantice n'a pas
espanso-reason-date-format = Format de date que Mantice ne sait pas écrire
espanso-reason-nested = Utilise un combo qui a été ignoré
espanso-reason-no-text = Rien à taper
espanso-reason-control = Déclencheur avec Tab ou Entrée
msg-espanso-imported =
    { $count ->
        [one] { $count } combo a été importé dans le groupe « { $group } ».
       *[other] { $count } combos ont été importés dans le groupe « { $group } ».
    }
error-espanso-search-page = Ceci est la page de recherche du hub. Ouvrez un paquet et collez son lien, ou saisissez son nom.
error-espanso-unknown-source = « { $input } » n'est ni le nom d'un paquet, ni un lien, ni un fichier ou un dossier de cet ordinateur.
error-espanso-unsupported-link = Ce lien n'est ni un paquet du hub, ni un fichier .zip ou .yml.
error-espanso-not-https = Seuls les liens https peuvent être téléchargés.
error-espanso-not-in-hub = Le hub n'a aucun paquet nommé « { $name } ».
error-espanso-not-in-hub-similar = Le hub n'a aucun paquet nommé « { $name } ». Noms proches : { $similar }.
error-espanso-index = La liste des paquets du hub n'a pas pu être lue.
error-espanso-zip = Le fichier .zip n'a pas pu être lu : { $detail }
error-espanso-yaml = Le paquet n'est pas du YAML valide : { $detail }
error-espanso-no-matches = Les fichiers ne contiennent aucun match espanso.
error-espanso-no-files = Il n'y a aucun fichier .yml.
error-espanso-too-big = Le paquet est trop volumineux pour être importé.
error-espanso-interrupted = Le chargement s'est interrompu de façon inattendue.
error-download-failed = Le téléchargement a échoué (erreur Windows { $code }). Vérifiez la connexion internet.
error-download-status = Le serveur a répondu avec l'erreur { $status }.

## Backups

restore-auto-title = Restaurer une sauvegarde automatique
restore-auto-info = Une sauvegarde de la liste des combos est créée à chaque enregistrement.
restore-no-backup = Aucune sauvegarde disponible.
restore-remove-all = Supprimer toutes les sauvegardes...
restore-action = Restaurer
confirm-remove-backups = Supprimer toutes les sauvegardes automatiques ?

## Application lists

applist-excluded-title = Applications exclues
applist-excluded-help = Mantice n'effectue aucune substitution dans ces applications.
applist-sensitive-title = Applications sensibles
applist-sensitive-help = Ces applications ne permettent pas de coller au clavier : les snippets y sont tapés caractère par caractère.
applist-emoji-title = Applications exclues de la substitution des emojis
applist-emoji-help = Les shortcodes d'emojis ne sont pas remplacés dans ces applications.
applist-format = Un nom de processus par ligne (ex. notepad.exe). Les caractères génériques * et ? sont acceptés.
applist-example = Exemple de nom de processus : { $name }

## Beeftext migration

migration-title = Bibliothèque Beeftext trouvée
migration-text = Une liste de combos Beeftext a été trouvée sur cet ordinateur. Mantice peut l'importer avec tous ses groupes.

## About

about-version = Version { $version }
about-tagline = Un expanseur de texte écrit en Rust. Tapez un mot-clé, obtenez un snippet.
about-credits-beeftext = Inspiré de Beeftext de Xavier Michelon (licence MIT). Les fichiers de combos sont compatibles avec Beeftext.
about-credits-emoji = Shortcodes d'emojis issus de gemoji (licence MIT).
about-data-folder = Dossier des données : { $path }

## Variables reference

help-intro = Les variables sont évaluées à chaque déclenchement d'un combo. Utilisez { $escaped_brace } et { $escaped_backslash } pour écrire { $brace } et { $backslash } dans une variable.
help-clipboard = Le texte contenu dans le presse-papiers.
help-cursor = Position du curseur de texte après la substitution.
help-date = Date et/ou heure actuelles, au format du système.
help-custom-format = Date/heure actuelles avec un format personnalisé (d dd ddd dddd M MM MMM MMMM yy yyyy h hh H HH m mm s ss zzz AP w ww).
help-shift = Date/heure décalée : unités y M w d h m s z, le signe est obligatoire. Le format peut rester vide ; les deux-points finaux sont obligatoires.
help-combo = Le snippet d'un autre combo (également { $upper }, { $lower } et { $trim }).
help-input = Demande une valeur à l'utilisateur. Une même description n'est demandée qu'une fois.
help-env = La valeur d'une variable d'environnement.
help-powershell = La sortie d'un script PowerShell (délai de 10 s par défaut, 0 = aucun délai).
help-key = Appuie sur une touche (tab, enter, up, f5...), éventuellement plusieurs fois.
help-shortcut = Appuie sur un raccourci clavier.
help-delay = Met la substitution en pause.

## Preferences

prefs-title = Préférences
prefs-tab-behavior = Comportement
prefs-tab-appearance = Apparence
prefs-tab-combos = Combos
prefs-tab-emojis = Emojis
prefs-tab-advanced = Avancé
prefs-section-general = Général
prefs-enabled = Mantice est activé
prefs-autostart = Lancer Mantice à l'ouverture de session
prefs-portable-unavailable = Non disponible dans l'édition portable.
prefs-play-sound = Émettre un son au déclenchement d'un combo
prefs-custom-sound = Son personnalisé
prefs-section-substitution = Substitution
prefs-automatic = Automatique : le combo se déclenche dès que le mot-clé est tapé
prefs-manual = Manuel : tapez le mot-clé, puis appuyez sur le raccourci de déclenchement
prefs-trigger-shortcut = Raccourci de déclenchement
prefs-trigger-on-space = Déclencher les combos par un espace tapé après le mot-clé
prefs-keep-space = Conserver l'espace après le snippet
prefs-section-picker = Sélecteur de combos
prefs-enable-picker = Activer le sélecteur de combos
prefs-shortcut = Raccourci
prefs-section-toggle = Raccourci d'activation / désactivation
prefs-use-toggle = Utiliser un raccourci pour activer ou désactiver Mantice
prefs-recording = Appuyez sur un raccourci... (Échap pour annuler)
prefs-record-hint = Cliquez, puis appuyez sur le nouveau raccourci
prefs-section-language = Langue
prefs-system-language = Langue du système ({ $language })
prefs-section-theme = Thème
prefs-theme-system = Comme le système
prefs-theme-light = Clair
prefs-theme-dark = Sombre
prefs-section-defaults = Valeurs par défaut des nouveaux combos
prefs-matching-help = Strict : le texte tapé doit être exactement le mot-clé. Permissif : le combo peut se déclencher au milieu d'un mot.
prefs-section-warnings = Avertissements
prefs-warn-short = Avertir pour les mots-clés de moins de 3 caractères
prefs-warn-empty = Avertir pour les mots-clés vides
prefs-section-combo-file = Fichier de la liste des combos
prefs-change-folder = Changer de dossier...
prefs-default-folder = Utiliser le dossier par défaut
prefs-sync-folder-hint = Un dossier synchronisé (OneDrive, Dropbox...) permet de partager vos combos entre plusieurs ordinateurs.
msg-combo-list-moved =
    La liste des combos est maintenant enregistrée dans :
    { $path }
prefs-section-emoji = Shortcodes d'emojis
prefs-emoji-enable = Utiliser les shortcodes d'emojis
prefs-left-delimiter = Délimiteur gauche
prefs-right-delimiter = Délimiteur droit
prefs-example = Exemple :
prefs-emoji-list = Liste des emojis...
prefs-excluded-apps = Applications exclues...
prefs-sensitive-apps = Applications sensibles...
prefs-show-emojis-in-picker = Afficher les emojis dans le sélecteur de combos (à partir de 3 caractères)
prefs-section-text-insertion = Insertion du texte
prefs-restore-clipboard = Restaurer le presse-papiers après une substitution
prefs-shift-insert = Coller avec Maj+Inser au lieu de Ctrl+V
prefs-keystroke-delay = Délai entre les frappes lors de la saisie (ms)
prefs-section-backup = Sauvegarde
prefs-auto-backup = Sauvegarder automatiquement la liste des combos
prefs-custom-location = Emplacement personnalisé
prefs-custom-executable = Exécutable personnalisé
prefs-powershell-hint = Par exemple pwsh.exe (PowerShell 7) au lieu de Windows PowerShell.
prefs-section-diagnostics = Diagnostic
prefs-write-log = Écrire un fichier journal
prefs-reset = Réinitialiser les préférences...
prefs-confirm-reset = Rétablir toutes les préférences à leurs valeurs par défaut ?
msg-autostart-failed = Le démarrage automatique n'a pas pu être modifié.

## Combo picker and input dialog

picker-title = Mantice - Sélecteur de combos
picker-search = Rechercher un combo...
picker-no-match = Aucun combo correspondant.
picker-help = ↑↓ pour choisir · Entrée pour insérer · Échap pour fermer

## System tray

tray-open = Ouvrir Mantice
tray-tooltip-paused = Mantice (en pause)

## Errors

error-open-file = Impossible d'ouvrir le fichier « { $path } ».
error-write-file = Impossible d'écrire le fichier « { $path } ».
error-not-json = Le fichier n'est pas un JSON valide.
error-invalid-list = Le fichier de la liste des combos est invalide.
error-no-version = Le fichier de la liste des combos n'indique pas son numéro de version.
error-newer-version = Le fichier de la liste des combos a été créé par une version plus récente de l'application.
error-older-version = Le fichier utilise un ancien format de Beeftext (version { $version }). Exporter les combos depuis la dernière version de Beeftext, puis importer ce fichier.
error-invalid-groups = La liste des groupes est invalide.
error-invalid-group = La liste des groupes contient un groupe invalide.
error-invalid-combos = La liste des combos est invalide.
error-nothing-to-import = Le fichier ne contient aucune donnée importable.
error-registry = Le registre Windows n'a pas pu être mis à jour (erreur { $code }).
platform-hook-failed = Impossible d'intercepter le clavier : les substitutions sont désactivées.

## Key names (shown in shortcuts)

key-shift = Maj
key-enter = Entrée
key-space = Espace
key-backspace = Retour arrière
key-escape = Échap
key-insert = Inser
key-delete = Suppr
key-home = Origine
key-end = Fin
key-page-up = Pg préc
key-page-down = Pg suiv
key-left = Gauche
key-right = Droite
key-up = Haut
key-down = Bas
key-print = Impr. écran

## Emoji list

menu-emoji-list = Liste des emojis
emoji-list-title = Shortcodes d'emojis
emoji-list-intro = Tapez le code entre les délimiteurs, par exemple { $example }. Les noms sont en anglais.
emoji-list-disabled = La substitution des emojis est désactivée : activez-la dans Préférences › Emojis.
emoji-list-search = Rechercher par code ou par nom...
emoji-list-count =
    { $count ->
        [one] { $count } emoji
       *[other] { $count } emojis
    }
emoji-list-no-match = Aucun emoji ne correspond à la recherche.
emoji-list-click-hint = Cliquez sur une ligne pour copier son code.
emoji-list-copied = Copié : { $code }
emoji-group-all = Toutes les catégories
emoji-group-smileys = Smileys et émotions
emoji-group-people = Personnes et corps
emoji-group-animals = Animaux et nature
emoji-group-food = Nourriture et boissons
emoji-group-travel = Voyages et lieux
emoji-group-activities = Activités
emoji-group-objects = Objets
emoji-group-symbols = Symboles
emoji-group-flags = Drapeaux
