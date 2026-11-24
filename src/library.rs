//! Combos, groups and the combo library.
//!
//! The on-disk JSON format is the one used by the latest Beeftext (file format version 10), so that
//! Beeftext libraries, exports and backups can be loaded as-is. Older versions are refused.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{Local, NaiveDateTime};
use serde_json::{Map, Value, json};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::i18n::tr;
use crate::prefs::Preferences;

/// The version of the combo list file format written by Mantice (same as Beeftext 12+).
pub const FILE_FORMAT_VERSION: i64 = 10;
/// The default name of the library file.
pub const LIBRARY_FILE_NAME: &str = "comboList.json";
/// The name of the file storing last-use date/times (kept apart so that the library is not rewritten at each use).
pub const LAST_USE_FILE_NAME: &str = "comboLastUse.json";

const PLACEHOLDER_MAX_LEN: usize = 50;
const DATE_FMT_MS: &str = "%Y-%m-%dT%H:%M:%S%.3f";

pub fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

pub fn format_json_date(dt: &NaiveDateTime) -> String {
    dt.format(DATE_FMT_MS).to_string()
}

/// Parse a Qt `ISODate`/`ISODateWithMs` date string (with or without milliseconds and offset).
pub fn parse_json_date(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Local).naive_local());
    }
    for fmt in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M:%S%.f"] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt);
        }
    }
    None
}

// ---------------------------------------------------------------------------------------------------------------------
// Enumerations
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MatchingMode {
    /// Use the default matching mode defined in the preferences.
    #[default]
    Default,
    /// The typed text must be exactly the keyword.
    Strict,
    /// The typed text must end with the keyword (the combo can be triggered in the middle of a word).
    Loose,
}

impl MatchingMode {
    pub fn to_int(self) -> i64 {
        match self {
            Self::Default => 0,
            Self::Strict => 1,
            Self::Loose => 2,
        }
    }
    pub fn from_int(v: i64) -> Self {
        match v {
            1 => Self::Strict,
            2 => Self::Loose,
            _ => Self::Default,
        }
    }
    pub fn resolve(self, prefs: &Preferences) -> Self {
        match self {
            Self::Default => match prefs.default_matching_mode {
                Self::Default => Self::Strict,
                m => m,
            },
            m => m,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CaseSensitivity {
    #[default]
    Default,
    CaseSensitive,
    CaseInsensitive,
}

impl CaseSensitivity {
    pub fn to_int(self) -> i64 {
        match self {
            Self::Default => 0,
            Self::CaseSensitive => 1,
            Self::CaseInsensitive => 2,
        }
    }
    pub fn from_int(v: i64) -> Self {
        match v {
            1 => Self::CaseSensitive,
            2 => Self::CaseInsensitive,
            _ => Self::Default,
        }
    }
    pub fn resolve(self, prefs: &Preferences) -> Self {
        match self {
            Self::Default => match prefs.default_case_sensitivity {
                Self::Default => Self::CaseSensitive,
                c => c,
            },
            c => c,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Group
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub enabled: bool,
}

impl Group {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let n = now();
        Self { uuid: Uuid::new_v4(), name: name.into(), description: description.into(), created: n, modified: n, enabled: true }
    }

    pub fn touch(&mut self) {
        self.modified = now();
    }

    fn from_json(obj: &Map<String, Value>) -> Result<Self> {
        let uuid = obj.get("uuid").and_then(Value::as_str).and_then(parse_uuid).ok_or_else(|| anyhow!(tr("error-invalid-group")))?;
        let n = now();
        Ok(Self {
            uuid,
            name: str_prop(obj, "name"),
            description: str_prop(obj, "description"),
            created: obj.get("creationDateTime").and_then(Value::as_str).and_then(parse_json_date).unwrap_or(n),
            modified: obj.get("modificationDateTime").and_then(Value::as_str).and_then(parse_json_date).unwrap_or(n),
            enabled: obj.get("enabled").and_then(Value::as_bool).unwrap_or(true),
        })
    }

    fn to_json(&self) -> Value {
        json!({
            "uuid": format_uuid(&self.uuid),
            "name": self.name,
            "description": self.description,
            "creationDateTime": format_json_date(&self.created),
            "modificationDateTime": format_json_date(&self.modified),
            "enabled": self.enabled,
        })
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Combo
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Combo {
    pub uuid: Uuid,
    pub name: String,
    pub keyword: String,
    pub snippet: String,
    pub description: String,
    pub matching_mode: MatchingMode,
    pub case_sensitivity: CaseSensitivity,
    pub group: Option<Uuid>,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub last_use: Option<NaiveDateTime>,
    pub enabled: bool,
}

impl Default for Combo {
    fn default() -> Self {
        let n = now();
        Self {
            uuid: Uuid::new_v4(),
            name: String::new(),
            keyword: String::new(),
            snippet: String::new(),
            description: String::new(),
            matching_mode: MatchingMode::Default,
            case_sensitivity: CaseSensitivity::Default,
            group: None,
            created: n,
            modified: n,
            last_use: None,
            enabled: true,
        }
    }
}

impl Combo {
    pub fn new(name: &str, keyword: &str, snippet: &str) -> Self {
        Self { name: name.into(), keyword: keyword.into(), snippet: snippet.into(), ..Default::default() }
    }

    pub fn touch(&mut self) {
        self.modified = now();
    }

    pub fn placeholder_name(keyword: &str, snippet: &str) -> String {
        let snippet = snippet.replace(['\r', '\n'], " ");
        let result = if keyword.is_empty() { snippet } else { format!("[{keyword}]: {snippet}") };
        if result.chars().count() <= PLACEHOLDER_MAX_LEN {
            return result;
        }
        let mut s: String = result.chars().take(PLACEHOLDER_MAX_LEN - 3).collect();
        s.push_str("...");
        s
    }

    /// The name shown in the UI: the name if defined, otherwise a placeholder built from keyword and snippet.
    pub fn display_name(&self) -> String {
        if self.name.trim().is_empty() { Self::placeholder_name(&self.keyword, &self.snippet) } else { self.name.clone() }
    }

    /// A duplicate has a new UUID and an empty keyword (two combos should not share a keyword).
    pub fn duplicate(&self) -> Self {
        Self {
            name: self.name.clone(),
            snippet: self.snippet.clone(),
            description: self.description.clone(),
            matching_mode: self.matching_mode,
            case_sensitivity: self.case_sensitivity,
            group: self.group,
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.uuid.is_nil() && !self.snippet.is_empty()
    }

    /// Does the combo match the text typed by the user?
    pub fn matches_input(&self, input: &str, prefs: &Preferences) -> bool {
        if self.keyword.is_empty() {
            return false;
        }
        let insensitive = self.case_sensitivity.resolve(prefs) == CaseSensitivity::CaseInsensitive;
        match self.matching_mode.resolve(prefs) {
            MatchingMode::Loose => ends_with(input, &self.keyword, insensitive),
            _ => {
                if insensitive {
                    input.to_lowercase() == self.keyword.to_lowercase()
                } else {
                    input == self.keyword
                }
            }
        }
    }

    /// Number of user-perceived characters of the keyword, i.e. the number of backspaces needed to erase it.
    pub fn keyword_len(&self) -> usize {
        self.keyword.graphemes(true).count()
    }

    fn from_json(obj: &Map<String, Value>, groups: &[Group]) -> Result<Self> {
        let n = now();
        let date = |key| obj.get(key).and_then(Value::as_str).and_then(parse_json_date).unwrap_or(n);
        let combo = Self {
            uuid: obj.get("uuid").and_then(Value::as_str).and_then(parse_uuid).unwrap_or_else(Uuid::new_v4),
            name: str_prop(obj, "name"),
            keyword: str_prop(obj, "keyword"),
            snippet: str_prop(obj, "snippet"),
            description: str_prop(obj, "description"),
            matching_mode: MatchingMode::from_int(obj.get("matchingMode").and_then(Value::as_i64).unwrap_or(1)),
            case_sensitivity: CaseSensitivity::from_int(obj.get("caseSensitivity").and_then(Value::as_i64).unwrap_or(0)),
            group: obj.get("group").and_then(Value::as_str).and_then(parse_uuid).filter(|g| groups.iter().any(|x| &x.uuid == g)),
            created: date("creationDateTime"),
            modified: date("modificationDateTime"),
            last_use: None,
            enabled: obj.get("enabled").and_then(Value::as_bool).unwrap_or(true),
        };
        if !combo.is_valid() {
            bail!("invalid combo (empty snippet)");
        }
        Ok(combo)
    }

    pub fn to_json(&self, include_group: bool) -> Value {
        let mut obj = Map::new();
        obj.insert("uuid".into(), json!(format_uuid(&self.uuid)));
        obj.insert("name".into(), json!(self.name));
        obj.insert("keyword".into(), json!(self.keyword));
        obj.insert("snippet".into(), json!(self.snippet));
        obj.insert("description".into(), json!(self.description));
        obj.insert("matchingMode".into(), json!(self.matching_mode.to_int()));
        obj.insert("caseSensitivity".into(), json!(self.case_sensitivity.to_int()));
        obj.insert("creationDateTime".into(), json!(format_json_date(&self.created)));
        obj.insert("modificationDateTime".into(), json!(format_json_date(&self.modified)));
        obj.insert("enabled".into(), json!(self.enabled));
        if include_group && let Some(g) = &self.group {
            obj.insert("group".into(), json!(format_uuid(g)));
        }
        Value::Object(obj)
    }
}

fn ends_with(input: &str, keyword: &str, insensitive: bool) -> bool {
    if !insensitive {
        return input.ends_with(keyword);
    }
    let n = keyword.chars().count();
    let total = input.chars().count();
    if total < n {
        return false;
    }
    let tail: String = input.chars().skip(total - n).collect();
    tail.to_lowercase() == keyword.to_lowercase()
}

fn str_prop(obj: &Map<String, Value>, key: &str) -> String {
    obj.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// Beeftext writes UUIDs using Qt's format, with curly braces.
pub fn format_uuid(u: &Uuid) -> String {
    format!("{{{}}}", u.hyphenated())
}

pub fn parse_uuid(s: &str) -> Option<Uuid> {
    Uuid::parse_str(s.trim().trim_start_matches('{').trim_end_matches('}')).ok().filter(|u| !u.is_nil())
}

// ---------------------------------------------------------------------------------------------------------------------
// Library
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Library {
    pub combos: Vec<Combo>,
    pub groups: Vec<Group>,
}

/// The result of parsing a library file.
pub struct ParsedLibrary {
    pub library: Library,
    pub skipped: usize,
}

impl Library {
    pub fn group(&self, uuid: &Uuid) -> Option<&Group> {
        self.groups.iter().find(|g| &g.uuid == uuid)
    }

    pub fn group_mut(&mut self, uuid: &Uuid) -> Option<&mut Group> {
        self.groups.iter_mut().find(|g| &g.uuid == uuid)
    }

    pub fn combo(&self, uuid: &Uuid) -> Option<&Combo> {
        self.combos.iter().find(|c| &c.uuid == uuid)
    }

    pub fn combo_mut(&mut self, uuid: &Uuid) -> Option<&mut Combo> {
        self.combos.iter_mut().find(|c| &c.uuid == uuid)
    }

    pub fn group_name(&self, uuid: &Option<Uuid>) -> String {
        uuid.as_ref().and_then(|u| self.group(u)).map(|g| g.name.clone()).unwrap_or_default()
    }

    /// A combo is usable if it is enabled and its group is enabled.
    pub fn is_usable(&self, combo: &Combo) -> bool {
        combo.enabled && combo.group.as_ref().and_then(|g| self.group(g)).map(|g| g.enabled).unwrap_or(true)
    }

    /// Make sure there is at least one group and that every combo belongs to an existing group.
    /// Returns true if the library was modified.
    /// Groups still called by the default name of another language ("Default Group" created before the
    /// language was changed) take the default name of the current language.
    pub fn localize_default_group_names(&mut self, default_names: &[String], current: &str) -> bool {
        let mut modified = false;
        for g in &mut self.groups {
            if g.name != current && default_names.contains(&g.name) {
                g.name = current.to_string();
                g.modified = now();
                modified = true;
            }
        }
        modified
    }

    pub fn ensure_correct_grouping(&mut self, default_group_name: &str) -> bool {
        let mut modified = false;
        if self.groups.is_empty() {
            self.groups.push(Group::new(default_group_name, ""));
            modified = true;
        }
        let first = self.groups[0].uuid;
        let ids: Vec<Uuid> = self.groups.iter().map(|g| g.uuid).collect();
        for combo in &mut self.combos {
            if !combo.group.map(|g| ids.contains(&g)).unwrap_or(false) {
                combo.group = Some(first);
                modified = true;
            }
        }
        modified
    }

    pub fn to_json_string(&self, include_groups: bool) -> String {
        let doc = json!({
            "fileFormatVersion": FILE_FORMAT_VERSION,
            "combos": self.combos.iter().map(|c| c.to_json(include_groups)).collect::<Vec<_>>(),
            "groups": if include_groups { self.groups.iter().map(Group::to_json).collect::<Vec<_>>() } else { vec![] },
        });
        serde_json::to_string_pretty(&doc).unwrap_or_default()
    }

    pub fn from_json_str(text: &str) -> Result<ParsedLibrary> {
        let text = text.trim_start_matches('\u{feff}');
        let doc: Value = serde_json::from_str(text).context(tr("error-not-json"))?;
        let root = doc.as_object().ok_or_else(|| anyhow!(tr("error-invalid-list")))?;
        let version = root.get("fileFormatVersion").and_then(Value::as_f64).ok_or_else(|| anyhow!(tr("error-no-version")))? as i64;
        if version > FILE_FORMAT_VERSION {
            bail!(tr("error-newer-version"));
        }
        if version < FILE_FORMAT_VERSION {
            bail!(tr!("error-older-version", version = version));
        }
        let mut library = Library::default();
        let groups = root.get("groups").and_then(Value::as_array).ok_or_else(|| anyhow!(tr("error-invalid-groups")))?;
        for g in groups {
            let obj = g.as_object().ok_or_else(|| anyhow!(tr("error-invalid-group")))?;
            let group = Group::from_json(obj)?;
            if library.group(&group.uuid).is_none() {
                library.groups.push(group);
            }
        }
        let combos = root.get("combos").and_then(Value::as_array).ok_or_else(|| anyhow!(tr("error-invalid-combos")))?;
        let mut skipped = 0;
        for c in combos {
            let Some(obj) = c.as_object() else {
                skipped += 1;
                continue;
            };
            match Combo::from_json(obj, &library.groups) {
                Ok(mut combo) => {
                    if library.combo(&combo.uuid).is_some() {
                        combo.uuid = Uuid::new_v4();
                    }
                    library.combos.push(combo);
                }
                Err(e) => {
                    log::warn!("Skipping a combo while parsing the combo list: {e}");
                    skipped += 1;
                }
            }
        }
        Ok(ParsedLibrary { library, skipped })
    }

    pub fn load(path: &Path) -> Result<ParsedLibrary> {
        let text = std::fs::read_to_string(path).with_context(|| tr!("error-open-file", path = path.display().to_string()))?;
        Self::from_json_str(&text)
    }

    pub fn save(&self, path: &Path, include_groups: bool) -> Result<()> {
        write_atomic(path, self.to_json_string(include_groups).as_bytes())
    }

    // ---- CSV ------------------------------------------------------------------------------------------------------

    /// Export in the TextExpander-compatible CSV format: keyword, snippet, name.
    pub fn export_csv<'a>(combos: impl Iterator<Item = &'a Combo>, path: &Path) -> Result<()> {
        let mut w = csv::WriterBuilder::new().has_headers(false).from_path(path)?;
        for c in combos {
            w.write_record([&c.keyword, &c.snippet, &c.name])?;
        }
        w.flush()?;
        Ok(())
    }

    pub fn parse_csv(text: &str) -> Result<Vec<Combo>> {
        let text = text.trim_start_matches('\u{feff}');
        let mut r = csv::ReaderBuilder::new().has_headers(false).flexible(true).from_reader(text.as_bytes());
        let mut result = Vec::new();
        for rec in r.records() {
            let rec = rec?;
            if rec.len() < 2 {
                continue;
            }
            let keyword = rec.get(0).unwrap_or_default().to_string();
            let snippet = rec.get(1).unwrap_or_default().to_string();
            if snippet.is_empty() {
                continue;
            }
            let mut name = rec.get(2).unwrap_or_default().to_string();
            if name.is_empty() {
                name = keyword.clone();
            }
            result.push(Combo::new(&name, &keyword, &snippet));
        }
        Ok(result)
    }

    // ---- Last use -------------------------------------------------------------------------------------------------

    pub fn last_use_json(&self) -> String {
        let items: Vec<Value> = self
            .combos
            .iter()
            .filter_map(|c| c.last_use.map(|d| json!({ "uuid": format_uuid(&c.uuid), "dateTime": format_json_date(&d) })))
            .collect();
        serde_json::to_string_pretty(&json!({ "fileFormatVersion": 1, "dateTimes": items })).unwrap_or_default()
    }

    pub fn apply_last_use_json(&mut self, text: &str) {
        let Ok(doc) = serde_json::from_str::<Value>(text) else { return };
        let Some(items) = doc.get("dateTimes").and_then(Value::as_array) else { return };
        let map: HashMap<Uuid, NaiveDateTime> = items
            .iter()
            .filter_map(|v| {
                let u = v.get("uuid").and_then(Value::as_str).and_then(parse_uuid)?;
                let d = v.get("dateTime").and_then(Value::as_str).and_then(parse_json_date)?;
                Some((u, d))
            })
            .collect();
        for c in &mut self.combos {
            if let Some(d) = map.get(&c.uuid) {
                c.last_use = Some(*d);
            }
        }
    }
}

/// Write a file atomically: write a temporary sibling, then rename it.
pub fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    // the engine and the user interface can save the same file at the same moment, with the same temporary file
    static WRITE_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
    let _one_at_a_time = WRITE_LOCK.lock();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data).with_context(|| tr!("error-write-file", path = tmp.display().to_string()))?;
    std::fs::rename(&tmp, path).with_context(|| tr!("error-write-file", path = path.display().to_string()))?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictPolicy {
    /// Conflicting combos (same keyword as an existing one) are not imported.
    #[default]
    Skip,
    /// Conflicting combos replace the existing ones.
    Overwrite,
    /// Conflicting combos replace the existing ones only if they were modified more recently.
    OverwriteOlder,
}

#[derive(Debug, Default)]
pub struct ImportAnalysis {
    pub new_combos: Vec<Combo>,
    /// Imported combos that are more recent than the existing ones with the same keyword.
    pub conflicting_newer: Vec<Combo>,
    pub conflicting_older: Vec<Combo>,
}

impl ImportAnalysis {
    pub fn importable_count(&self, policy: ConflictPolicy) -> usize {
        self.new_combos.len()
            + match policy {
                ConflictPolicy::Skip => 0,
                ConflictPolicy::OverwriteOlder => self.conflicting_newer.len(),
                ConflictPolicy::Overwrite => self.conflicting_newer.len() + self.conflicting_older.len(),
            }
    }
    pub fn conflict_count(&self) -> usize {
        self.conflicting_newer.len() + self.conflicting_older.len()
    }
}

/// Read a file to import: a Beeftext/Mantice JSON file, or a CSV file.
pub fn read_import_file(path: &Path) -> Result<Vec<Combo>> {
    let bytes = std::fs::read(path).with_context(|| tr!("error-open-file", path = path.display().to_string()))?;
    let text = String::from_utf8_lossy(&bytes);
    match Library::from_json_str(&text) {
        Ok(parsed) => Ok(parsed.library.combos),
        // a JSON file the parser refuses (newer format...) would give garbage combos if read as CSV
        Err(json_err) if text.trim_start_matches('\u{feff}').trim_start().starts_with('{') => Err(json_err),
        Err(json_err) => {
            let combos = Library::parse_csv(&text).map_err(|_| json_err)?;
            if combos.is_empty() {
                bail!(tr("error-nothing-to-import"));
            }
            Ok(combos)
        }
    }
}

pub fn analyze_import(library: &Library, candidates: Vec<Combo>) -> ImportAnalysis {
    // a table of the keywords, because a package of the espanso hub can have a hundred thousand combos
    let mut existing: std::collections::HashMap<&str, &Combo> = std::collections::HashMap::new();
    for combo in library.combos.iter().filter(|c| !c.keyword.is_empty()) {
        existing.entry(combo.keyword.as_str()).or_insert(combo);
    }
    let mut a = ImportAnalysis::default();
    for mut combo in candidates {
        combo.uuid = Uuid::new_v4();
        match existing.get(combo.keyword.as_str()) {
            None => a.new_combos.push(combo),
            Some(existing) if combo.modified > existing.modified => a.conflicting_newer.push(combo),
            Some(_) => a.conflicting_older.push(combo),
        }
    }
    a
}

/// The combos of an espanso package (see [`crate::espanso`]). The matches are moved out of the package.
pub fn combos_from_espanso(package: &mut crate::espanso::Package) -> Vec<Combo> {
    std::mem::take(&mut package.matches)
        .into_iter()
        .map(|m| Combo {
            name: m.label,
            keyword: m.keyword,
            snippet: m.snippet,
            // espanso finds its triggers at the end of what is typed, unless `word` asks for a whole word
            matching_mode: if m.whole_word { MatchingMode::Strict } else { MatchingMode::Loose },
            case_sensitivity: if m.ignore_case { CaseSensitivity::CaseInsensitive } else { CaseSensitivity::CaseSensitive },
            ..Default::default()
        })
        .collect()
}

/// Apply an import. Returns the number of imported combos.
pub fn apply_import(library: &mut Library, analysis: ImportAnalysis, policy: ConflictPolicy, group: Option<Uuid>) -> usize {
    let mut count = 0;
    let mut replace = Vec::new();
    match policy {
        ConflictPolicy::Skip => {}
        ConflictPolicy::OverwriteOlder => replace.extend(analysis.conflicting_newer),
        ConflictPolicy::Overwrite => {
            replace.extend(analysis.conflicting_newer);
            replace.extend(analysis.conflicting_older);
        }
    }
    for mut combo in replace {
        if let Some(existing) = library.combos.iter_mut().find(|c| !c.keyword.is_empty() && c.keyword == combo.keyword) {
            combo.uuid = existing.uuid;
            combo.group = group.or(existing.group);
            combo.last_use = existing.last_use;
            *existing = combo;
            count += 1;
        }
    }
    for mut combo in analysis.new_combos {
        combo.group = group;
        library.combos.push(combo);
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    const BEEFTEXT_V10: &str = r#"{
        "combos": [
            {"caseSensitivity": 0, "creationDateTime": "2023-01-10T10:00:00.000", "description": "", "enabled": true,
             "group": "{2f0b6d36-2a1d-4c5f-9f3a-7a4b2c3d4e5f}", "keyword": "btsig", "matchingMode": 1,
             "modificationDateTime": "2023-01-11T10:00:00.000", "name": "Signature", "snippet": "Regards,\nJohn",
             "uuid": "{8c1b4a2e-1111-4c5f-9f3a-7a4b2c3d4e5f}"},
            {"caseSensitivity": 2, "creationDateTime": "2023-01-10T10:00:00.000", "description": "arrow", "enabled": false,
             "keyword": "==>", "matchingMode": 2, "modificationDateTime": "2023-01-12T10:00:00.000", "name": "",
             "snippet": "➜", "uuid": "{8c1b4a2e-2222-4c5f-9f3a-7a4b2c3d4e5f}"}
        ],
        "fileFormatVersion": 10,
        "groups": [
            {"creationDateTime": "2023-01-10T10:00:00.000", "description": "", "enabled": true,
             "modificationDateTime": "2023-01-10T10:00:00.000", "name": "Default Group",
             "uuid": "{2f0b6d36-2a1d-4c5f-9f3a-7a4b2c3d4e5f}"}
        ]
    }"#;

    #[test]
    fn espanso_matches_become_combos() {
        use crate::espanso::{Match, Package};
        let m = |keyword: &str, whole_word, ignore_case| Match {
            label: "L".into(),
            keyword: keyword.into(),
            snippet: "S".into(),
            whole_word,
            ignore_case,
        };
        let mut package = Package { matches: vec![m(":a", false, false), m("b", true, true)], ..Default::default() };
        let combos = combos_from_espanso(&mut package);
        assert!(package.matches.is_empty());
        let shape: Vec<_> =
            combos.iter().map(|c| (c.keyword.as_str(), c.name.as_str(), c.snippet.as_str(), c.matching_mode, c.case_sensitivity)).collect();
        assert_eq!(
            shape,
            [
                (":a", "L", "S", MatchingMode::Loose, CaseSensitivity::CaseSensitive),
                ("b", "L", "S", MatchingMode::Strict, CaseSensitivity::CaseInsensitive)
            ]
        );
        // what is already in the library is a conflict, the rest is new
        let mut lib = Library::default();
        lib.combos.push(Combo::new("", ":a", "mine"));
        let analysis = analyze_import(&lib, combos);
        assert_eq!((analysis.new_combos.len(), analysis.conflict_count()), (1, 1));
    }

    #[test]
    fn default_group_follows_the_language() {
        let mut lib = Library::from_json_str(BEEFTEXT_V10).unwrap().library;
        lib.groups.push(Group::new("Work", ""));
        let defaults: Vec<String> = ["Default Group", "Gruppo predefinito", "Groupe par défaut"].map(String::from).into();
        assert!(lib.localize_default_group_names(&defaults, "Gruppo predefinito"));
        assert_eq!(lib.groups[0].name, "Gruppo predefinito");
        // groups the user named are left alone, and nothing changes the second time
        assert_eq!(lib.groups[1].name, "Work");
        assert!(!lib.localize_default_group_names(&defaults, "Gruppo predefinito"));
    }

    #[test]
    fn parse_beeftext_v10() {
        let p = Library::from_json_str(BEEFTEXT_V10).unwrap();
        assert_eq!(p.library.groups.len(), 1);
        assert_eq!(p.library.combos.len(), 2);
        let c = &p.library.combos[0];
        assert_eq!(c.keyword, "btsig");
        assert_eq!(c.group, Some(p.library.groups[0].uuid));
        assert_eq!(c.matching_mode, MatchingMode::Strict);
        let c2 = &p.library.combos[1];
        assert_eq!(c2.matching_mode, MatchingMode::Loose);
        assert_eq!(c2.case_sensitivity, CaseSensitivity::CaseInsensitive);
        assert!(!c2.enabled);
        assert_eq!(c2.group, None);
    }

    #[test]
    fn roundtrip() {
        let mut p = Library::from_json_str(BEEFTEXT_V10).unwrap().library;
        p.ensure_correct_grouping("Default Group");
        let text = p.to_json_string(true);
        let p2 = Library::from_json_str(&text).unwrap().library;
        assert_eq!(p.combos.len(), p2.combos.len());
        assert_eq!(p.combos[1].group, p2.combos[1].group);
        assert_eq!(p.combos[0].snippet, p2.combos[0].snippet);
        assert_eq!(p.combos[0].modified, p2.combos[0].modified);
    }

    #[test]
    fn reject_other_formats() {
        assert!(Library::from_json_str(r#"{"fileFormatVersion": 11, "combos": [], "groups": []}"#).is_err());
        let old = Library::from_json_str(r#"{"fileFormatVersion": 9, "combos": [], "groups": []}"#).err().unwrap();
        assert!(old.to_string().contains('9'), "{old}");
    }

    #[test]
    fn matching() {
        let prefs = Preferences::default();
        let mut c = Combo::new("", "==>", "➜");
        assert!(c.matches_input("==>", &prefs));
        assert!(!c.matches_input("First==>", &prefs));
        c.matching_mode = MatchingMode::Loose;
        assert!(c.matches_input("First==>", &prefs));
        let mut c = Combo::new("", "Sig", "x");
        assert!(!c.matches_input("sig", &prefs));
        c.case_sensitivity = CaseSensitivity::CaseInsensitive;
        assert!(c.matches_input("sIG", &prefs));
        c.matching_mode = MatchingMode::Loose;
        assert!(c.matches_input("àèSIG", &prefs));
        assert!(!c.matches_input("ig", &prefs));
    }

    #[test]
    fn placeholder() {
        assert_eq!(Combo::placeholder_name("kw", "hello"), "[kw]: hello");
        let long = "a".repeat(80);
        let p = Combo::placeholder_name("", &long);
        assert_eq!(p.chars().count(), 50);
        assert!(p.ends_with("..."));
    }

    #[test]
    fn csv_import() {
        let combos = Library::parse_csv("kw1,\"Hello, world\",Name 1\nkw2,second,\n").unwrap();
        assert_eq!(combos.len(), 2);
        assert_eq!(combos[0].snippet, "Hello, world");
        assert_eq!(combos[1].name, "kw2");
    }

    #[test]
    fn refused_json_is_not_imported_as_csv() {
        let path = std::env::temp_dir().join(format!("mantice-import-test-{}.json", std::process::id()));
        let newer = r#"{
    "combos": [ {"keyword": "addr", "snippet": "Via Roma 1, Napoli", "uuid": "{8c1b4a2e-1111-4c5f-9f3a-7a4b2c3d4e5f}"} ],
    "fileFormatVersion": 11,
    "groups": []
}"#;
        std::fs::write(&path, newer).unwrap();
        assert!(read_import_file(&path).is_err());
        std::fs::write(&path, "kw,\"Via Roma 1, Napoli\",Address\n").unwrap();
        assert_eq!(read_import_file(&path).unwrap()[0].snippet, "Via Roma 1, Napoli");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn import_conflicts() {
        let mut lib = Library::from_json_str(BEEFTEXT_V10).unwrap().library;
        let mut newer = Combo::new("New sig", "btsig", "new");
        newer.modified = parse_json_date("2030-01-01T00:00:00").unwrap();
        let mut older = Combo::new("Old arrow", "==>", "old");
        older.modified = parse_json_date("2000-01-01T00:00:00").unwrap();
        let fresh = Combo::new("Fresh", "fr", "fresh");
        let a = analyze_import(&lib, vec![newer, older, fresh]);
        assert_eq!(a.new_combos.len(), 1);
        assert_eq!(a.conflicting_newer.len(), 1);
        assert_eq!(a.conflicting_older.len(), 1);
        assert_eq!(a.importable_count(ConflictPolicy::OverwriteOlder), 2);
        let n = apply_import(&mut lib, a, ConflictPolicy::OverwriteOlder, None);
        assert_eq!(n, 2);
        assert_eq!(lib.combos.len(), 3);
        assert_eq!(lib.combos.iter().find(|c| c.keyword == "btsig").unwrap().snippet, "new");
        assert_eq!(lib.combos.iter().find(|c| c.keyword == "==>").unwrap().snippet, "➜");
    }

    #[test]
    fn last_use_roundtrip() {
        let mut lib = Library::from_json_str(BEEFTEXT_V10).unwrap().library;
        lib.combos[0].last_use = Some(parse_json_date("2024-05-05T12:00:00.000").unwrap());
        let text = lib.last_use_json();
        let mut lib2 = Library::from_json_str(BEEFTEXT_V10).unwrap().library;
        lib2.apply_last_use_json(&text);
        assert_eq!(lib2.combos[0].last_use, lib.combos[0].last_use);
        assert_eq!(lib2.combos[1].last_use, None);
    }
}
