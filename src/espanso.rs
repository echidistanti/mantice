//! Reading of espanso packages (the ones of https://hub.espanso.org): the `_manifest.yml` that describes the
//! package and the match files (`package.yml` and, in the bigger packages, other `.yml` files next to it: espanso
//! loads all of them), translated into what Mantice can do.
//!
//! What a match becomes:
//!
//! | espanso | Mantice |
//! |---|---|
//! | `trigger`, or every item of `triggers` | the keyword of a combo (one combo per trigger) |
//! | `replace` | the snippet |
//! | `label` | the name of the combo |
//! | `word: true` | strict matching (the keyword is a whole word); otherwise loose matching |
//! | `propagate_case: true` | case-insensitive matching (the text is inserted as written: see [`Package::approximated`]) |
//! | `$\|$` | `#{cursor}` |
//! | variable `echo` | its text, written into the snippet |
//! | variable `clipboard` | `#{clipboard}` |
//! | variable `date` | `#{dateTime:...}` (the `strftime` format is translated into a Qt one, `offset` into a shift) |
//! | variable `match` | `#{combo:...}` when the match it points to is imported too |
//!
//! Everything else is left out and listed in [`Package::skipped`] with the reason: regular expression triggers,
//! forms, images, rich text (`markdown`, `html`), variables that run commands (`shell`, `script`) or pick a value
//! (`form`, `choice`, `list`, `random`), what Mantice does not know how to format, and the triggers that end with a
//! key that is not a character (Tab).
//!
//! This module knows nothing about the rest of the program (the combos are built from a [`Match`] elsewhere), so
//! it can be tested with the real packages of the hub.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use serde_yaml_ng::Value;

/// A package read from its files.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Package {
    /// The name of the package, from `_manifest.yml` (the name of its folder or file when it has none).
    pub name: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub version: String,
    /// What can be imported.
    pub matches: Vec<Match>,
    /// What cannot, with the reason.
    pub skipped: Vec<Skipped>,
    /// The match files that are not valid YAML, so that nothing of them is imported.
    pub unreadable: Vec<String>,
    /// How many of the matches are imported with a difference: those with `propagate_case` (espanso also types
    /// "Hello" and "HELLO" for the trigger "hello"; Mantice matches the three but always types the text as written),
    /// and the dates with a `locale` (Mantice uses the language of the program).
    pub approximated: usize,
    /// Triggers that appear more than once in the package: the first one is imported, the others are not.
    pub duplicates: usize,
}

/// One trigger of an espanso match: it becomes one combo.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub label: String,
    pub keyword: String,
    pub snippet: String,
    /// `word: true`: the keyword has to be typed as a whole word.
    pub whole_word: bool,
    /// `propagate_case: true`.
    pub ignore_case: bool,
}

/// A match that was left out.
#[derive(Debug, Clone, PartialEq)]
pub struct Skipped {
    /// The trigger, or the regular expression, or the label: what lets the user recognise the match.
    pub what: String,
    pub reason: Reason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reason {
    /// The match is activated by a regular expression.
    Regex,
    /// No trigger at all.
    NoTrigger,
    /// A form (a match or a variable): espanso asks for values in a window.
    Form,
    /// The match types an image.
    Image,
    /// The match types rich text (`markdown`, `html`).
    RichText,
    /// A variable of type `shell` or `script`.
    Command,
    /// A variable that picks a value: `choice`, `list`, `random`.
    Choice,
    /// A variable of another type, or one that is not defined.
    Variable,
    /// A date format Mantice cannot write.
    DateFormat,
    /// A `match` variable that points to a match which is not imported.
    Nested,
    /// The match has nothing to type.
    NoText,
    /// The trigger contains a key that is not a character (Tab, Enter).
    Control,
}

impl Reason {
    /// All the reasons, in the order they are shown.
    pub const ALL: [Reason; 12] = [
        Reason::Regex,
        Reason::NoTrigger,
        Reason::Form,
        Reason::Image,
        Reason::RichText,
        Reason::Command,
        Reason::Choice,
        Reason::Variable,
        Reason::DateFormat,
        Reason::Nested,
        Reason::NoText,
        Reason::Control,
    ];
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadError {
    /// The text is not valid YAML.
    Yaml(String),
    /// Valid YAML, but there are no matches in it.
    NotAMatchFile,
}

/// Read a package. `manifest` is the text of `_manifest.yml` (a manifest that cannot be read is ignored: the
/// package keeps `fallback_name`); `files` are its match files, as (name, text). The files that are not match
/// files (other YAML that the package carries) are ignored, and the ones that are not valid YAML are listed in
/// [`Package::unreadable`].
pub fn read_package(manifest: Option<&str>, files: &[(String, String)], fallback_name: &str) -> Result<Package, ReadError> {
    let mut result = Package { name: fallback_name.to_string(), title: fallback_name.to_string(), ..Default::default() };
    if let Some(text) = manifest
        && let Ok(Some(m)) = load(text)
    {
        let field = |key: &str| scalar(&m[key]).map(|s| s.trim().to_string()).unwrap_or_default();
        if !field("name").is_empty() {
            result.name = field("name");
        }
        result.title = if field("title").is_empty() { result.name.clone() } else { field("title") };
        result.description = field("description");
        result.author = field("author");
        result.version = field("version");
    }

    // first pass: every match of every file, with its references to other matches still unchecked
    let mut translated: Vec<(Match, Vec<String>)> = Vec::new();
    let mut match_files = 0;
    let mut first_error = None;
    for (name, text) in files {
        let root = match load(text) {
            Ok(Some(root)) => root,
            Ok(None) => continue,
            Err(e) => {
                result.unreadable.push(name.clone());
                first_error.get_or_insert(e);
                continue;
            }
        };
        if !matches!(root, Value::Mapping(_)) || (root.get("matches").is_none() && root.get("global_vars").is_none()) {
            continue;
        }
        match_files += 1;
        let globals = variables_by_name(&root["global_vars"]);
        for entry in root["matches"].as_sequence().map(Vec::as_slice).unwrap_or_default() {
            translate_entry(entry, &globals, &mut translated, &mut result);
        }
    }
    if match_files == 0 {
        return Err(match first_error {
            Some(e) => ReadError::Yaml(e),
            None => ReadError::NotAMatchFile,
        });
    }
    // one combo per keyword: with two matches for the same trigger espanso uses the first one
    let mut seen = HashSet::new();
    translated.retain(|(m, _)| {
        let first = seen.insert(m.keyword.clone());
        result.duplicates += usize::from(!first);
        first
    });
    // second pass: a `match` variable needs the match it points to. Leaving one out can orphan another one.
    loop {
        let imported: HashSet<&str> = translated.iter().map(|(m, _)| m.keyword.as_str()).collect();
        let Some(i) = translated.iter().position(|(_, refs)| refs.iter().any(|r| !imported.contains(r.as_str()))) else {
            break;
        };
        let (dropped, _) = translated.remove(i);
        result.skipped.push(Skipped { what: dropped.keyword, reason: Reason::Nested });
    }
    result.matches = translated.into_iter().map(|(m, _)| m).collect();
    Ok(result)
}

/// The first YAML document of a text (`None` when there is none), with its `<<` merge keys applied.
fn load(text: &str) -> Result<Option<Value>, String> {
    let Some(document) = serde_yaml_ng::Deserializer::from_str(text).next() else { return Ok(None) };
    let mut value = Value::deserialize(document).map_err(|e| e.to_string())?;
    let _ = value.apply_merge();
    Ok(Some(value))
}

/// The text of a scalar (a string, a number or a boolean).
fn scalar(node: &Value) -> Option<String> {
    match node {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn variables_by_name(list: &Value) -> HashMap<String, &Value> {
    let mut map = HashMap::new();
    for var in list.as_sequence().map(Vec::as_slice).unwrap_or_default() {
        if let Some(name) = var["name"].as_str() {
            map.insert(name.to_string(), var);
        }
    }
    map
}

/// Translate one match of the file. Adds to `out` one entry per trigger, or to `package.skipped` the reason why
/// it cannot be imported.
fn translate_entry(entry: &Value, globals: &HashMap<String, &Value>, out: &mut Vec<(Match, Vec<String>)>, package: &mut Package) {
    let label = scalar(&entry["label"]).unwrap_or_default();
    let mut triggers: Vec<String> = Vec::new();
    if let Some(t) = scalar(&entry["trigger"]) {
        triggers.push(t);
    }
    if let Some(list) = entry["triggers"].as_sequence() {
        triggers.extend(list.iter().filter_map(scalar));
    }
    triggers.retain(|t| !t.trim().is_empty());

    if triggers.is_empty() {
        let regex = scalar(&entry["regex"]).or_else(|| entry["regexes"].as_sequence().and_then(|l| l.first()).and_then(scalar));
        let (what, reason) = match regex {
            Some(r) => (r, Reason::Regex),
            None => (label, Reason::NoTrigger),
        };
        package.skipped.push(Skipped { what, reason });
        return;
    }
    let what = triggers[0].clone();
    let mut skip = |reason| package.skipped.push(Skipped { what: what.clone(), reason });

    if !entry["form"].is_null() {
        return skip(Reason::Form);
    }
    if !entry["image_path"].is_null() {
        return skip(Reason::Image);
    }
    if !entry["markdown"].is_null() || !entry["html"].is_null() {
        return skip(Reason::RichText);
    }
    let Some(replace) = scalar(&entry["replace"]) else {
        return skip(Reason::NoText);
    };

    let mut locals = globals.clone();
    locals.extend(variables_by_name(&entry["vars"]));
    let mut references = Vec::new();
    let mut approximated = entry["propagate_case"].as_bool() == Some(true);
    let snippet = match translate_text(&replace, &locals, &mut references, &mut approximated) {
        Ok(snippet) => snippet,
        Err(reason) => return skip(reason),
    };

    // a trigger that ends with Tab or Enter cannot be typed as text: Mantice would never see it
    let (triggers, untypable): (Vec<String>, Vec<String>) = triggers.into_iter().partition(|t| !t.chars().any(char::is_control));
    for what in untypable {
        package.skipped.push(Skipped { what, reason: Reason::Control });
    }
    if triggers.is_empty() {
        return;
    }
    package.approximated += usize::from(approximated);
    let whole_word = entry["word"].as_bool() == Some(true);
    let ignore_case = entry["propagate_case"].as_bool() == Some(true);
    for keyword in triggers {
        out.push((Match { label: label.clone(), keyword, snippet: snippet.clone(), whole_word, ignore_case }, references.clone()));
    }
}

/// Translate the text of a `replace`: the `{{variables}}` and the cursor. `references` receives the triggers of
/// the matches that `match` variables point to; `approximated` is set when something is not exactly the same.
fn translate_text(
    replace: &str,
    vars: &HashMap<String, &Value>,
    references: &mut Vec<String>,
    approximated: &mut bool,
) -> Result<String, Reason> {
    let mut result = String::with_capacity(replace.len());
    let mut rest = replace;
    while let Some(open) = rest.find("{{") {
        let Some(close) = rest[open + 2..].find("}}") else { break };
        let name = rest[open + 2..open + 2 + close].trim();
        result.push_str(&rest[..open]);
        if is_variable_name(name) {
            // `form.field` is a field of the variable `form`
            let var = vars.get(name).or_else(|| name.split_once('.').and_then(|(base, _)| vars.get(base))).ok_or(Reason::Variable)?;
            result.push_str(&translate_variable(var, references, approximated)?);
        } else {
            // not a variable: braces in the text of the snippet
            result.push_str(&rest[open..open + 2 + close + 2]);
        }
        rest = &rest[open + 2 + close + 2..];
    }
    result.push_str(rest);
    Ok(result.replace("$|$", "#{cursor}"))
}

fn is_variable_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

fn translate_variable(var: &Value, references: &mut Vec<String>, approximated: &mut bool) -> Result<String, Reason> {
    let params = &var["params"];
    match var["type"].as_str().unwrap_or_default() {
        "echo" | "dummy" => Ok(scalar(&params["echo"]).unwrap_or_default()),
        "clipboard" => Ok("#{clipboard}".to_string()),
        "date" => {
            if !params["locale"].is_null() {
                *approximated = true;
            }
            date_variable(params)
        }
        "match" => {
            let trigger = scalar(&params["trigger"]).ok_or(Reason::Variable)?;
            let text = format!("#{{combo:{}}}", escape_parameter(&trigger));
            references.push(trigger);
            Ok(text)
        }
        "shell" | "script" => Err(Reason::Command),
        "form" => Err(Reason::Form),
        "choice" | "list" | "random" => Err(Reason::Choice),
        _ => Err(Reason::Variable),
    }
}

/// `#{dateTime:...}` for a `date` variable.
fn date_variable(params: &Value) -> Result<String, Reason> {
    let format = scalar(&params["format"]).unwrap_or_else(|| "%c".to_string());
    let shift = match &params["offset"] {
        Value::Null => String::new(),
        other => shift_text(other.as_i64().ok_or(Reason::DateFormat)?),
    };
    // the formats of the locale of the system have a variable of their own, which a shift does not apply to
    if shift.is_empty() {
        match format.as_str() {
            "%c" => return Ok("#{dateTime}".to_string()),
            "%x" => return Ok("#{date}".to_string()),
            "%X" => return Ok("#{time}".to_string()),
            _ => {}
        }
    }
    let qt = strftime_to_qt(&format).ok_or(Reason::DateFormat)?;
    if qt.is_empty() {
        return Err(Reason::DateFormat);
    }
    let shift = if shift.is_empty() { shift } else { format!("{shift}:") };
    Ok(format!("#{{dateTime:{shift}{}}}", escape_parameter(&qt)))
}

/// A number of seconds as the shift of a `dateTime` variable (`+1d+2h`, `-30m`...).
fn shift_text(seconds: i64) -> String {
    let sign = if seconds < 0 { '-' } else { '+' };
    let mut left = seconds.unsigned_abs();
    let mut out = String::new();
    for (unit, length) in [('d', 86_400), ('h', 3_600), ('m', 60), ('s', 1)] {
        if left >= length {
            out.push_str(&format!("{sign}{}{unit}", left / length));
            left %= length;
        }
    }
    out
}

/// In the parameter of a variable `}` and `\` have to be written `\}` and `\\`.
fn escape_parameter(text: &str) -> String {
    text.replace('\\', r"\\").replace('}', r"\}")
}

/// Translate a `strftime` format (the one of espanso) into a Qt one (the one of `#{dateTime:...}`). `None` when
/// it has something Qt cannot write.
fn strftime_to_qt(format: &str) -> Option<String> {
    let mut out = String::new();
    let mut literal = String::new();
    let (mut hour12, mut ampm) = (false, false);
    let mut chars = format.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            literal.push(c);
            continue;
        }
        let mut spec = chars.next()?;
        let mut flag = None;
        if matches!(spec, '-' | '_' | '0') {
            flag = Some(spec);
            spec = chars.next()?;
        }
        let unpadded = flag == Some('-');
        let token = match spec {
            '%' => {
                literal.push('%');
                continue;
            }
            'Y' => "yyyy",
            'y' => "yy",
            'm' if unpadded => "M",
            'm' => "MM",
            'd' if unpadded => "d",
            'd' => "dd",
            'e' => "d",
            'H' if unpadded => "H",
            'H' => "HH",
            'M' if unpadded => "m",
            'M' => "mm",
            'S' if unpadded => "s",
            'S' => "ss",
            'I' => {
                hour12 = true;
                if unpadded { "h" } else { "hh" }
            }
            'p' => {
                ampm = true;
                "AP"
            }
            'P' => {
                ampm = true;
                "ap"
            }
            'B' => "MMMM",
            'b' | 'h' => "MMM",
            'A' => "dddd",
            'a' => "ddd",
            'F' => "yyyy-MM-dd",
            'T' => "HH:mm:ss",
            'R' => "HH:mm",
            'D' => "MM/dd/yy",
            _ => return None,
        };
        flush_literal(&mut literal, &mut out)?;
        out.push_str(token);
    }
    flush_literal(&mut literal, &mut out)?;
    // in Qt the hours are those of a 12-hour clock only when the format has AM/PM
    if hour12 && !ampm {
        return None;
    }
    Some(out)
}

/// Move the literal text to the Qt format: between quotes when it has letters, which would be read as fields.
/// `None` for a line break, which a variable cannot hold.
fn flush_literal(literal: &mut String, out: &mut String) -> Option<()> {
    if literal.contains(['\n', '\r']) {
        return None;
    }
    if literal.chars().any(|c| c.is_ascii_alphabetic() || c == '\'') {
        out.push('\'');
        out.push_str(&literal.replace('\'', "''"));
        out.push('\'');
    } else {
        out.push_str(literal);
    }
    literal.clear();
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(manifest: Option<&str>, text: &str, name: &str) -> Result<Package, ReadError> {
        read_package(manifest, &[("package.yml".to_string(), text.to_string())], name)
    }

    fn read(package: &str) -> Package {
        one(None, package, "test").unwrap()
    }

    fn only(package: &Package) -> &Match {
        assert_eq!(package.matches.len(), 1, "{package:#?}");
        &package.matches[0]
    }

    #[test]
    fn plain_matches_and_their_flags() {
        let p = read(
            "matches:\n  - trigger: ':hi'\n    replace: 'Hello'\n  - trigger: 'btw'\n    replace: 'by the way'\n    word: true\n    propagate_case: true\n    label: Abbreviation\n",
        );
        assert_eq!(
            p.matches[0],
            Match { label: String::new(), keyword: ":hi".into(), snippet: "Hello".into(), whole_word: false, ignore_case: false }
        );
        assert_eq!(p.matches[1].keyword, "btw");
        assert!(p.matches[1].whole_word && p.matches[1].ignore_case);
        assert_eq!(p.matches[1].label, "Abbreviation");
        assert_eq!(p.approximated, 1);
        assert!(p.skipped.is_empty());
    }

    #[test]
    fn every_trigger_is_a_combo() {
        let p = read("matches:\n  - triggers: [':a', ':b']\n    trigger: ':c'\n    replace: x\n");
        let keywords: Vec<&str> = p.matches.iter().map(|m| m.keyword.as_str()).collect();
        assert_eq!(keywords, [":c", ":a", ":b"]);
    }

    #[test]
    fn numbers_and_multiline_text_are_text() {
        let p = read("matches:\n  - trigger: ':n'\n    replace: 42\n  - trigger: ':m'\n    replace: |\n      one\n      two\n");
        assert_eq!(p.matches[0].snippet, "42");
        assert_eq!(p.matches[1].snippet, "one\ntwo\n");
    }

    #[test]
    fn the_cursor_and_the_simple_variables() {
        let p = read(
            "matches:\n  - trigger: ':q'\n    replace: '\"$|$\" {{who}} {{ clip }}'\n    vars:\n      - name: who\n        type: echo\n        params:\n          echo: Ada\n      - name: clip\n        type: clipboard\n",
        );
        assert_eq!(only(&p).snippet, "\"#{cursor}\" Ada #{clipboard}");
    }

    #[test]
    fn braces_that_are_not_variables_stay() {
        let p = read("matches:\n  - trigger: ':j'\n    replace: 'fn() { {{ not a name }} }'\n");
        assert_eq!(only(&p).snippet, "fn() { {{ not a name }} }");
    }

    #[test]
    fn global_variables_are_visible_and_local_ones_win() {
        let p = read(
            "global_vars:\n  - name: g\n    type: echo\n    params:\n      echo: global\nmatches:\n  - trigger: ':a'\n    replace: '{{g}}'\n  - trigger: ':b'\n    replace: '{{g}}'\n    vars:\n      - name: g\n        type: echo\n        params:\n          echo: local\n",
        );
        assert_eq!(p.matches[0].snippet, "global");
        assert_eq!(p.matches[1].snippet, "local");
    }

    #[test]
    fn dates() {
        let date = |params: &str| {
            let text = format!(
                "matches:\n  - trigger: ':d'\n    replace: '{{{{d}}}}'\n    vars:\n      - name: d\n        type: date\n        params:\n{params}\n"
            );
            let p = read(&text);
            p.matches.first().map(|m| m.snippet.clone()).ok_or_else(|| p.skipped[0].reason)
        };
        assert_eq!(date("          format: '%Y-%m-%d'").unwrap(), "#{dateTime:yyyy-MM-dd}");
        assert_eq!(date("          format: '%A %-d %B %Y'").unwrap(), "#{dateTime:dddd d MMMM yyyy}");
        assert_eq!(date("          format: '%H:%M'\n          offset: 86400").unwrap(), "#{dateTime:+1d:HH:mm}");
        assert_eq!(date("          format: '%d/%m'\n          offset: -5400").unwrap(), "#{dateTime:-1h-30m:dd/MM}");
        assert_eq!(date("          format: '%I:%M %p'").unwrap(), "#{dateTime:hh:mm AP}");
        assert_eq!(date("          format: 'Today is %A.'").unwrap(), "#{dateTime:'Today is 'dddd.}");
        assert_eq!(date("          format: '%F %T'").unwrap(), "#{dateTime:yyyy-MM-dd HH:mm:ss}");
        assert_eq!(date("          format: '100%% %Y'").unwrap(), "#{dateTime:100% yyyy}");
        assert_eq!(date("          format: '%c'").unwrap(), "#{dateTime}");
        assert_eq!(date("          format: '%x'").unwrap(), "#{date}");
        assert_eq!(date("          {}").unwrap(), "#{dateTime}");
        assert_eq!(date("          format: '%j'").unwrap_err(), Reason::DateFormat);
        assert_eq!(date("          format: '%I:%M'").unwrap_err(), Reason::DateFormat, "12 hours without AM/PM");
        assert_eq!(date("          format: '%c'\n          offset: 60").unwrap_err(), Reason::DateFormat);
    }

    #[test]
    fn the_locale_of_a_date_is_an_approximation() {
        let p = read(
            "matches:\n  - trigger: ':d'\n    replace: '{{d}}'\n    vars:\n      - name: d\n        type: date\n        params:\n          format: '%A'\n          locale: es-ES\n",
        );
        assert_eq!(only(&p).snippet, "#{dateTime:dddd}");
        assert_eq!(p.approximated, 1);
    }

    #[test]
    fn nested_matches_need_their_target() {
        let nested = |target: &str| {
            format!(
                "matches:\n  - trigger: ':one'\n    replace: 'first'\n  - trigger: ':two'\n    replace: 'a {{{{o}}}} b'\n    vars:\n      - name: o\n        type: match\n        params:\n          trigger: '{target}'\n"
            )
        };
        let p = read(&nested(":one"));
        assert_eq!(p.matches[1].snippet, "a #{combo::one} b");
        let p = read(&nested(":nowhere"));
        assert_eq!(p.matches.len(), 1);
        assert_eq!(p.skipped, [Skipped { what: ":two".into(), reason: Reason::Nested }]);
    }

    #[test]
    fn leaving_out_a_target_leaves_out_the_matches_that_need_it() {
        let p = read(
            "matches:\n  - trigger: ':a'\n    replace: '{{s}}'\n    vars:\n      - name: s\n        type: shell\n  - trigger: ':b'\n    replace: '{{a}}'\n    vars:\n      - name: a\n        type: match\n        params:\n          trigger: ':a'\n",
        );
        assert!(p.matches.is_empty());
        let reasons: Vec<Reason> = p.skipped.iter().map(|s| s.reason).collect();
        assert_eq!(reasons, [Reason::Command, Reason::Nested]);
    }

    #[test]
    fn what_cannot_be_imported_says_why() {
        let p = read(
            "matches:
  - regex: ':h(\\\\d)'
    replace: x
  - label: nothing
    replace: x
  - trigger: ':f'
    form: 'Hi [[name]]'
  - trigger: ':i'
    image_path: /a.png
  - trigger: ':m'
    markdown: '**bold**'
  - trigger: ':h'
    html: '<b>bold</b>'
  - trigger: ':e'
  - trigger: ':s'
    replace: '{{o}}'
    vars: [{name: o, type: script}]
  - trigger: ':c'
    replace: '{{o}}'
    vars: [{name: o, type: choice}]
  - trigger: ':g'
    replace: '{{o.field}}'
    vars: [{name: o, type: form}]
  - trigger: ':u'
    replace: '{{o}}'
    vars: [{name: o, type: other}]
  - trigger: ':x'
    replace: '{{undefined}}'
",
        );
        assert!(p.matches.is_empty());
        let reasons: Vec<Reason> = p.skipped.iter().map(|s| s.reason).collect();
        use Reason::*;
        assert_eq!(reasons, [Regex, NoTrigger, Form, Image, RichText, RichText, NoText, Command, Choice, Form, Variable, Variable]);
        assert_eq!(p.skipped[0].what, ":h(\\\\d)");
        assert_eq!(p.skipped[1].what, "nothing");
    }

    #[test]
    fn the_manifest_names_the_package() {
        let manifest = "name: \"simple-package\"\ntitle: \"Simple Package\"\ndescription: A simple package\nversion: 0.1.0\nauthor: Ada\ntags: [a, b]\n";
        let p = one(Some(manifest), "matches:\n  - trigger: ':a'\n    replace: b\n", "folder").unwrap();
        assert_eq!((p.name.as_str(), p.title.as_str(), p.description.as_str()), ("simple-package", "Simple Package", "A simple package"));
        assert_eq!((p.version.as_str(), p.author.as_str()), ("0.1.0", "Ada"));
        // without a manifest, or with one that does not parse, the package is named after its folder
        let p = one(None, "matches: []", "folder").unwrap();
        assert_eq!((p.name.as_str(), p.title.as_str()), ("folder", "folder"));
        let p = one(Some("name: [unclosed"), "matches: []", "folder").unwrap();
        assert_eq!(p.title, "folder");
        let p = one(Some("name: x\n"), "matches: []", "folder").unwrap();
        assert_eq!((p.name.as_str(), p.title.as_str()), ("x", "x"));
    }

    #[test]
    fn a_trigger_used_twice_is_imported_once() {
        let files = [(
            "package.yml".to_string(),
            "matches:\n  - {trigger: ':a', replace: one}\n  - {triggers: [':b', ':a'], replace: two}\n".to_string(),
        )];
        let p = read_package(None, &files, "dup").unwrap();
        let kw: Vec<(&str, &str)> = p.matches.iter().map(|m| (m.keyword.as_str(), m.snippet.as_str())).collect();
        assert_eq!(kw, [(":a", "one"), (":b", "two")]);
        assert_eq!(p.duplicates, 1);
    }

    #[test]
    fn a_trigger_that_ends_with_tab_is_left_out() {
        let files = [("package.yml".to_string(), "matches:\n  - {triggers: [\"ab\\t\", ':ok'], replace: text}\n".to_string())];
        let p = read_package(None, &files, "tab").unwrap();
        assert_eq!(p.matches.iter().map(|m| m.keyword.as_str()).collect::<Vec<_>>(), [":ok"]);
        assert_eq!(p.skipped, [Skipped { what: "ab\t".into(), reason: Reason::Control }]);
    }

    #[test]
    fn files_that_are_not_packages() {
        assert!(matches!(one(None, "matches: [unclosed", "x"), Err(ReadError::Yaml(_))));
        assert_eq!(one(None, "just text", "x"), Err(ReadError::NotAMatchFile));
        assert_eq!(one(None, "name: only\nparent: default\n", "x"), Err(ReadError::NotAMatchFile));
        assert_eq!(one(None, "", "x"), Err(ReadError::NotAMatchFile));
        assert!(one(None, "matches:\n", "x").unwrap().matches.is_empty());
    }

    #[test]
    fn shifts() {
        assert_eq!(shift_text(86_400), "+1d");
        assert_eq!(shift_text(-86_400 * 2 - 7_200), "-2d-2h");
        assert_eq!(shift_text(3_661), "+1h+1m+1s");
        assert_eq!(shift_text(0), "");
    }
}
