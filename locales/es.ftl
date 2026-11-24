# Mantice user interface: Spanish.
# Message ids are shared by all the locales; en.ftl is the reference.
# Plural forms follow the CLDR rules of the language ([one], [other]...).

## General

app-default-group = Grupo predeterminado
app-error = Error
app-warning = Advertencia
app-information = Información
app-confirmation = Confirmación
app-yes = Sí
app-ok = Aceptar
app-cancel = Cancelar
app-browse = Examinar...
app-reset = Restablecer
app-clear = Borrar
app-optional = Opcional
app-enabled = Activado
app-default-with-value = Predeterminado ({ $value })
app-portable-edition = Edición portátil
app-status-counts =
    { $combos ->
        [one] { $combos } combo
       *[other] { $combos } combos
    } · { $groups ->
        [one] { $groups } grupo
       *[other] { $groups } grupos
    }

## Main window: menus

menu-file = Archivo
label-combo = Combo
label-group = Grupo
menu-tools = Herramientas
menu-help = Ayuda
menu-import = Importar combos...
menu-import-espanso = Importar desde espanso...
menu-export-all = Exportar todos los combos...
menu-export-selected = Exportar los combos seleccionados...
menu-preferences = Preferencias...
menu-hide-window = Ocultar la ventana
menu-quit = Salir
menu-new-group = Nuevo grupo...
menu-edit-group = Editar el grupo...
menu-toggle-group = Activar / desactivar el grupo
menu-delete-group = Eliminar el grupo...
menu-backup = Hacer una copia de seguridad de la lista...
menu-restore = Restaurar la lista de combos...
menu-restore-auto = Restaurar una copia de seguridad automática...
menu-open-data-folder = Abrir la carpeta de datos
menu-open-log = Abrir el archivo de registro
menu-variables-reference = Referencia de variables
menu-about = Acerca de Mantice

## Main window: combo actions

action-new-combo = Nuevo combo...
action-edit = Editar...
action-duplicate = Duplicar
action-copy-snippet = Copiar el fragmento al portapapeles
action-enable = Activar
action-disable = Desactivar
action-matching-mode = Modo de coincidencia
action-case-sensitivity = Mayúsculas y minúsculas
action-move-to-group = Mover al grupo
action-delete = Eliminar...

## Main window: groups, table, status bar

groups-title = Grupos
groups-all = Todos los combos
group-enable = Activar el grupo
group-disable = Desactivar el grupo
group-move-up = Subir
group-move-down = Bajar
combos-search = Buscar combos...
label-new-combo = Nuevo combo
label-name = Nombre
label-keyword = Palabra clave
label-snippet = Fragmento
column-last-used = Último uso
combos-empty-title = Todavía no hay combos
combos-empty-text = Un combo asocia una palabra clave corta a un fragmento. Escribe la palabra clave en cualquier aplicación y se sustituye por el fragmento.
combos-create-first = Crear tu primer combo
combos-no-match = Ningún combo coincide con la búsqueda.
status-active = Activo
status-paused = En pausa
status-toggle-hint = Haz clic para activar o desactivar Mantice

## Matching and case sensitivity

matching-strict = Estricta
matching-loose = Flexible
case-sensitive = Distingue mayúsculas
case-insensitive = No distingue mayúsculas

## Main window: confirmations and messages

confirm-delete-combos =
    { $count ->
        [one] ¿Eliminar el combo seleccionado?
       *[other] ¿Eliminar los { $count } combos seleccionados?
    }
confirm-delete-group = ¿Eliminar el grupo «{ $name }»?
confirm-delete-group-with-combos =
    { $count ->
        [one] ¿Eliminar el grupo «{ $name }» y el combo que contiene?
       *[other] ¿Eliminar el grupo «{ $name }» y los { $count } combos que contiene?
    }
confirm-replace-library = Todos los combos y grupos existentes se sustituirán por el contenido del archivo de copia de seguridad. ¿Continuar?
msg-last-group = El último grupo no se puede eliminar.
filter-combo-files = Archivos de combos
filter-all-files = Todos los archivos
filter-csv-textexpander = CSV (compatible con TextExpander)
file-name-backup = Copia de seguridad Mantice { $date }
msg-import-failed = No se ha podido importar el archivo.
msg-exported = Se han exportado los combos.
msg-export-failed = No se han podido exportar los combos.
msg-backup-saved = Se ha guardado la copia de seguridad.
msg-load-failed = No se ha podido cargar el archivo.
msg-combos-restored =
    { $count ->
        [one] Se ha restaurado { $count } combo.
       *[other] Se han restaurado { $count } combos.
    }
msg-combos-imported =
    { $count ->
        [one] Se ha importado { $count } combo.
       *[other] Se han importado { $count } combos.
    }
msg-backups-removed =
    { $count ->
        [one] Se ha eliminado { $count } copia de seguridad.
       *[other] Se han eliminado { $count } copias de seguridad.
    }
msg-library-load-failed = No se ha podido cargar la lista de combos.
msg-library-copy-saved = Se ha guardado una copia del archivo como: { $path }
msg-library-save-failed = No se ha podido guardar la lista de combos.
msg-library-reloaded = Otro programa u otro equipo ha modificado la lista de combos y se ha vuelto a cargar. El último cambio no se ha guardado: vuelve a hacerlo.
msg-invalid-combos-ignored =
    { $count ->
        [one] Se ha ignorado { $count } combo no válido al cargar la lista.
       *[other] Se han ignorado { $count } combos no válidos al cargar la lista.
    }

## Combo editor

editor-edit-title = Editar el combo
editor-keyword-hint = p. ej. btsig
editor-matching = Coincidencia
editor-description = Descripción
editor-combo-enabled = Activado
editor-snippet-hint = El texto que sustituye a la palabra clave
editor-snippet-empty = El fragmento no puede estar vacío.
editor-warn-empty-keyword = Sin palabra clave, el combo solo se puede insertar con el selector de combos.
editor-warn-short-keyword = Las palabras clave cortas pueden activarse por error.
editor-warn-duplicate-keyword = Esta palabra clave también la usa «{ $name }»: se elegirá uno de ellos al azar.
editor-insert-variable = Insertar variable
editor-recording-shortcut = Pulsa el atajo que quieres insertar (Esc para cancelar)...
var-clipboard = Contenido del portapapeles
var-cursor = Posición del cursor
var-date = Fecha
var-time = Hora
var-date-time = Fecha y hora
var-custom-date-time = Fecha/hora personalizada
var-shifted-date-time = Fecha/hora desplazada
var-combo-upper = Combo en mayúsculas
var-combo-lower = Combo en minúsculas
var-combo-trim = Combo sin espacios en los extremos
var-input = Entrada del usuario
var-env = Variable de entorno
var-powershell = Script de PowerShell
var-delay = Pausa (ms)
var-key = Tecla
var-shortcut-record = Atajo... (grabar)

## Group editor

group-new-title = Nuevo grupo
group-edit-title = Editar el grupo

## Import

import-title = Importar combos
import-new-combos = Combos nuevos: { $count }
import-conflicts =
    { $count ->
        [one] { $count } combo tiene la misma palabra clave que un combo existente.
       *[other] { $count } combos tienen la misma palabra clave que un combo existente.
    }
import-skip = Omitir los combos en conflicto
import-overwrite = Sobrescribir los combos existentes
import-overwrite-older = Sobrescribir solo los combos existentes más antiguos ({ $count })
import-into-group = Importar en el grupo
import-button = Importar ({ $count })
import-action = Importar

## Import of espanso packages

espanso-title = Importar un paquete de espanso
espanso-help = Escribe el nombre de un paquete del hub de espanso (hub.espanso.org) o pega su enlace. También puedes elegir un archivo .zip o .yml, o una carpeta, de este equipo.
espanso-source-hint = Nombre o enlace del paquete, por ejemplo basic-emojis
espanso-choose-file = Archivo...
espanso-choose-folder = Carpeta...
espanso-file-filter = Paquetes de espanso
espanso-load = Cargar
espanso-loading = Cargando el paquete...
espanso-by-author = de { $author }
espanso-new-group = Grupo nuevo
espanso-existing-group = Grupo existente
espanso-left-out = Omitidos: { $count }
espanso-approximated = Combos que se comportan de forma ligeramente distinta a espanso: { $count }. Mantice siempre escribe el texto tal como está, sin seguir las mayúsculas de la palabra clave, y las fechas usan su propio idioma.
espanso-duplicates = Disparadores repetidos, importados una sola vez: { $count }
espanso-unreadable = Archivos que no son YAML válido, omitidos: { $files }
espanso-large = Es un paquete muy grande: la lista de combos se hará muy larga.
espanso-reason-regex = Activado por una expresión regular
espanso-reason-no-trigger = Sin disparador
espanso-reason-form = Formulario
espanso-reason-image = Imagen
espanso-reason-rich-text = Texto con formato (Markdown, HTML)
espanso-reason-command = Ejecuta un comando o un script
espanso-reason-choice = Pide elegir un valor (choice, list, random)
espanso-reason-variable = Variable que Mantice no tiene
espanso-reason-date-format = Formato de fecha que Mantice no sabe escribir
espanso-reason-nested = Usa un combo que se ha omitido
espanso-reason-no-text = Nada que escribir
espanso-reason-control = Disparador con Tab o Intro
msg-espanso-imported =
    { $count ->
        [one] Se ha importado { $count } combo en el grupo «{ $group }».
       *[other] Se han importado { $count } combos en el grupo «{ $group }».
    }
error-espanso-search-page = Esta es la página de búsqueda del hub. Abre un paquete y pega su enlace, o escribe su nombre.
error-espanso-unknown-source = «{ $input }» no es el nombre de un paquete, ni un enlace, ni un archivo o carpeta de este equipo.
error-espanso-unsupported-link = Este enlace no es un paquete del hub ni un archivo .zip o .yml.
error-espanso-not-https = Solo se pueden descargar enlaces https.
error-espanso-not-in-hub = El hub no tiene ningún paquete llamado «{ $name }».
error-espanso-not-in-hub-similar = El hub no tiene ningún paquete llamado «{ $name }». Nombres parecidos: { $similar }.
error-espanso-index = No se ha podido leer la lista de paquetes del hub.
error-espanso-zip = No se ha podido leer el archivo .zip: { $detail }
error-espanso-yaml = El paquete no es YAML válido: { $detail }
error-espanso-no-matches = Los archivos no contienen coincidencias de espanso.
error-espanso-no-files = No hay archivos .yml.
error-espanso-too-big = El paquete es demasiado grande para importarlo.
error-espanso-interrupted = La carga se ha interrumpido de forma inesperada.
error-download-failed = La descarga ha fallado (error de Windows { $code }). Comprueba la conexión a internet.
error-download-status = El servidor ha respondido con el error { $status }.

## Backups

restore-auto-title = Restaurar una copia de seguridad automática
restore-auto-info = Cada vez que se guarda la lista de combos se crea una copia de seguridad.
restore-no-backup = No hay copias de seguridad disponibles.
restore-remove-all = Eliminar todas las copias de seguridad...
restore-action = Restaurar
confirm-remove-backups = ¿Eliminar todas las copias de seguridad automáticas?

## Application lists

applist-excluded-title = Aplicaciones excluidas
applist-excluded-help = Mantice no realiza ninguna sustitución en estas aplicaciones.
applist-sensitive-title = Aplicaciones sensibles
applist-sensitive-help = Estas aplicaciones no admiten pegar con el teclado: los fragmentos se escriben carácter a carácter.
applist-emoji-title = Aplicaciones excluidas de la sustitución de emojis
applist-emoji-help = Los códigos cortos de emoji no se sustituyen en estas aplicaciones.
applist-format = Un nombre de proceso por línea (p. ej. notepad.exe). Se aceptan los comodines * y ?.
applist-example = Ejemplo de nombre de proceso: { $name }

## Beeftext migration

migration-title = Biblioteca de Beeftext encontrada
migration-text = Se ha encontrado una lista de combos de Beeftext en este equipo. Mantice puede importarla con todos sus grupos.

## About

about-version = Versión { $version }
about-tagline = Un expansor de texto escrito en Rust. Escribe una palabra clave y obtén un fragmento.
about-credits-beeftext = Inspirado en Beeftext, de Xavier Michelon (licencia MIT). Los archivos de combos son compatibles con Beeftext.
about-credits-emoji = Códigos cortos de emoji de gemoji (licencia MIT).
about-data-folder = Carpeta de datos: { $path }

## Variables reference

help-intro = Las variables se evalúan cada vez que se activa un combo. Usa { $escaped_brace } y { $escaped_backslash } para escribir { $brace } y { $backslash } dentro de una variable.
help-clipboard = El texto del portapapeles.
help-cursor = Posición del cursor de texto después de la sustitución.
help-date = Fecha, hora o ambas, en el formato del sistema.
help-custom-format = Fecha/hora actuales con un formato personalizado (d dd ddd dddd M MM MMM MMMM yy yyyy h hh H HH m mm s ss zzz AP w ww).
help-shift = Fecha/hora desplazada: unidades y M w d h m s z, el signo es obligatorio. El formato puede quedar vacío; los dos puntos finales son obligatorios.
help-combo = El fragmento de otro combo (también { $upper }, { $lower } y { $trim }).
help-input = Pide un valor al usuario. Cada descripción se pregunta una sola vez.
help-env = El valor de una variable de entorno.
help-powershell = La salida de un script de PowerShell (tiempo límite de 10 s por defecto, 0 = sin límite).
help-key = Pulsa una tecla (tab, enter, up, f5...), opcionalmente varias veces.
help-shortcut = Pulsa un atajo de teclado.
help-delay = Pausa la sustitución.

## Preferences

prefs-title = Preferencias
prefs-tab-behavior = Comportamiento
prefs-tab-appearance = Apariencia
prefs-tab-combos = Combos
prefs-tab-emojis = Emojis
prefs-tab-advanced = Avanzado
prefs-section-general = General
prefs-enabled = Mantice está activado
prefs-autostart = Iniciar Mantice al iniciar sesión
prefs-portable-unavailable = No disponible en la edición portátil.
prefs-play-sound = Reproducir un sonido al activar un combo
prefs-custom-sound = Sonido personalizado
prefs-section-substitution = Sustitución
prefs-automatic = Automática: el combo se activa en cuanto se escribe la palabra clave
prefs-manual = Manual: escribe la palabra clave y pulsa el atajo de activación
prefs-trigger-shortcut = Atajo de activación
prefs-trigger-on-space = Activar los combos con un espacio escrito después de la palabra clave
prefs-keep-space = Conservar el espacio después del fragmento
prefs-section-picker = Selector de combos
prefs-enable-picker = Activar el selector de combos
prefs-shortcut = Atajo
prefs-section-toggle = Atajo para activar / desactivar
prefs-use-toggle = Usar un atajo para activar o desactivar Mantice
prefs-recording = Pulsa un atajo... (Esc para cancelar)
prefs-record-hint = Haz clic y pulsa el nuevo atajo
prefs-section-language = Idioma
prefs-system-language = Idioma del sistema ({ $language })
prefs-section-theme = Tema
prefs-theme-system = Como el sistema
prefs-theme-light = Claro
prefs-theme-dark = Oscuro
prefs-section-defaults = Valores predeterminados de los combos nuevos
prefs-matching-help = Estricta: el texto escrito debe coincidir exactamente con la palabra clave. Flexible: el combo puede activarse en medio de una palabra.
prefs-section-warnings = Advertencias
prefs-warn-short = Avisar de las palabras clave de menos de 3 caracteres
prefs-warn-empty = Avisar de las palabras clave vacías
prefs-section-combo-file = Archivo de la lista de combos
prefs-change-folder = Cambiar de carpeta...
prefs-default-folder = Usar la carpeta predeterminada
prefs-sync-folder-hint = Una carpeta sincronizada (OneDrive, Dropbox...) permite compartir los combos entre varios equipos.
msg-combo-list-moved =
    La lista de combos se guarda ahora en:
    { $path }
prefs-section-emoji = Códigos cortos de emoji
prefs-emoji-enable = Usar los códigos cortos de emoji
prefs-left-delimiter = Delimitador izquierdo
prefs-right-delimiter = Delimitador derecho
prefs-example = Ejemplo:
prefs-emoji-list = Lista de emojis...
prefs-excluded-apps = Aplicaciones excluidas...
prefs-sensitive-apps = Aplicaciones sensibles...
prefs-show-emojis-in-picker = Mostrar emojis en el selector de combos (a partir de 3 caracteres)
prefs-section-text-insertion = Inserción del texto
prefs-restore-clipboard = Restaurar el portapapeles después de una sustitución
prefs-shift-insert = Pegar con Mayús+Insert en lugar de Ctrl+V
prefs-keystroke-delay = Retardo entre pulsaciones al escribir (ms)
prefs-section-backup = Copia de seguridad
prefs-auto-backup = Hacer copias de seguridad automáticas de la lista de combos
prefs-custom-location = Ubicación personalizada
prefs-custom-executable = Ejecutable personalizado
prefs-powershell-hint = Por ejemplo, pwsh.exe (PowerShell 7) en lugar de Windows PowerShell.
prefs-section-diagnostics = Diagnóstico
prefs-write-log = Escribir un archivo de registro
prefs-reset = Restablecer las preferencias...
prefs-confirm-reset = ¿Restablecer todas las preferencias a sus valores predeterminados?
msg-autostart-failed = No se ha podido cambiar el inicio automático.

## Combo picker and input dialog

picker-title = Mantice - Selector de combos
picker-search = Buscar un combo...
picker-no-match = Ningún combo coincide.
picker-help = ↑↓ para elegir · Intro para insertar · Esc para cerrar

## System tray

tray-open = Abrir Mantice
tray-tooltip-paused = Mantice (en pausa)

## Errors

error-open-file = No se ha podido abrir el archivo «{ $path }».
error-write-file = No se ha podido escribir el archivo «{ $path }».
error-not-json = El archivo no es un JSON válido.
error-invalid-list = El archivo de la lista de combos no es válido.
error-no-version = El archivo de la lista de combos no indica su número de versión.
error-newer-version = El archivo de la lista de combos se creó con una versión más reciente de la aplicación.
error-older-version = El archivo usa un formato antiguo de Beeftext (versión { $version }). Exportar los combos desde la última versión de Beeftext e importar ese archivo.
error-invalid-groups = La lista de grupos no es válida.
error-invalid-group = La lista de grupos contiene un grupo no válido.
error-invalid-combos = La lista de combos no es válida.
error-nothing-to-import = El archivo no contiene datos importables.
error-registry = No se ha podido actualizar el registro de Windows (error { $code }).
platform-hook-failed = No se ha podido interceptar el teclado: las sustituciones están desactivadas.

## Key names (shown in shortcuts)

key-shift = Mayús
key-enter = Intro
key-space = Espacio
key-backspace = Retroceso
key-escape = Esc
key-insert = Insert
key-delete = Supr
key-home = Inicio
key-end = Fin
key-page-up = RePág
key-page-down = AvPág
key-left = Izquierda
key-right = Derecha
key-up = Arriba
key-down = Abajo
key-print = ImprPant

## Emoji list

menu-emoji-list = Lista de emojis
emoji-list-title = Códigos cortos de emoji
emoji-list-intro = Escribe el código entre los delimitadores, por ejemplo { $example }. Los nombres están en inglés.
emoji-list-disabled = La sustitución de emojis está desactivada: actívala en Preferencias › Emojis.
emoji-list-search = Buscar por código o nombre...
emoji-list-count =
    { $count ->
        [one] { $count } emoji
       *[other] { $count } emojis
    }
emoji-list-no-match = Ningún emoji coincide con la búsqueda.
emoji-list-click-hint = Haz clic en una fila para copiar su código.
emoji-list-copied = Copiado: { $code }
emoji-group-all = Todas las categorías
emoji-group-smileys = Caras y emociones
emoji-group-people = Personas y cuerpo
emoji-group-animals = Animales y naturaleza
emoji-group-food = Comida y bebida
emoji-group-travel = Viajes y lugares
emoji-group-activities = Actividades
emoji-group-objects = Objetos
emoji-group-symbols = Símbolos
emoji-group-flags = Banderas
