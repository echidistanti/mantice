//! Evaluation of snippet variables (`#{clipboard}`, `#{dateTime:...}`, `#{combo:...}`, `#{input:...}` and so on).
//!
//! The behaviour follows the Beeftext documentation (https://github.com/xmichelo/Beeftext/wiki/Variables).
//! `#{cursor}`, `#{key:}`, `#{shortcut:}` and `#{delay:}` are not evaluated here: they are kept in the text and
//! processed later by the [`crate::fragments`] module.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local, Locale};

use crate::qtdate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateKind {
    Date,
    Time,
    DateTime,
}

/// Services needed to evaluate variables. Implemented by the engine, and by mocks in the tests.
pub trait VariableHost {
    fn clipboard_text(&mut self) -> String;
    /// Ask the user for a value. Returns None if the user cancelled.
    fn prompt_input(&mut self, description: &str) -> Option<String>;
    /// The snippets of all the combos whose keyword is exactly `keyword` (enabled or not, like Beeftext).
    fn snippets_for_keyword(&self, keyword: &str) -> Vec<String>;
    fn env_var(&self, name: &str) -> String {
        std::env::var(name).unwrap_or_default()
    }
    fn run_powershell(&mut self, script_path: &str, timeout_ms: u64) -> String;
    fn now(&self) -> DateTime<Local> {
        Local::now()
    }
    fn locale(&self) -> Locale {
        Locale::en_US
    }
    fn system_date(&self, dt: &DateTime<Local>, kind: DateKind) -> String {
        match kind {
            DateKind::Date => qtdate::default_date(dt, self.locale()),
            DateKind::Time => qtdate::default_time(dt, self.locale()),
            DateKind::DateTime => qtdate::default_date_time(dt, self.locale()),
        }
    }
    fn random_index(&mut self, len: usize) -> usize;
}

/// Put before each `{` of a text that comes from outside the snippet (clipboard, input, environment variable,
/// PowerShell), so that [`crate::fragments`] does not run the `#{key:}`, `#{shortcut:}`, `#{delay:}` or
/// `#{cursor}` it may contain: copied text could otherwise press keys. Even a `#` written by the snippet just
/// before the value cannot form a `#{` with it. Removed before the text is inserted.
pub const LITERAL_MARK: char = '\u{E000}';

fn literal(text: String) -> String {
    text.replace('{', "\u{E000}{")
}

/// State shared across the (possibly recursive) evaluation of a snippet.
#[derive(Debug, Default)]
pub struct EvalState {
    /// Values already typed by the user for `#{input:}` variables, keyed by description.
    pub known_inputs: HashMap<String, String>,
}

impl EvalState {
    pub fn prompted_user(&self) -> bool {
        !self.known_inputs.is_empty()
    }
}

/// Locate the next variable starting at byte offset `from`. Returns (start, end, inner text).
/// Like Beeftext's `#\{((.*?)(?<!\\))\}`: the variable ends at the first `}` that is not preceded by a backslash,
/// and cannot span several lines.
pub fn find_variable(text: &str, from: usize) -> Option<(usize, usize, &str)> {
    let mut search = from;
    while let Some(rel) = text.get(search..)?.find("#{") {
        let start = search + rel;
        let inner_start = start + 2;
        let mut prev: Option<char> = None;
        for (off, c) in text[inner_start..].char_indices() {
            if c == '\n' {
                break;
            }
            if c == '}' && prev != Some('\\') {
                let end = inner_start + off;
                return Some((start, end + 1, &text[inner_start..end]));
            }
            prev = Some(c);
        }
        search = start + 1;
    }
    None
}

/// Replace `\\` by `\` and `\}` by `}` in a variable parameter.
pub fn resolve_escaping(param: &str) -> String {
    param.replace(r"\\", r"\").replace(r"\}", "}")
}

/// Evaluate all the variables of a snippet. Returns None if the user cancelled an input dialog.
pub fn evaluate_snippet(
    snippet: &str,
    host: &mut dyn VariableHost,
    forbidden_sub_combos: &HashSet<String>,
    state: &mut EvalState,
) -> Option<String> {
    let mut result = String::with_capacity(snippet.len());
    let mut pos = 0;
    while let Some((start, end, inner)) = find_variable(snippet, pos) {
        result.push_str(&snippet[pos..start]);
        let variable = inner.replace(r"\}", "}");
        result.push_str(&evaluate_variable(&variable, host, forbidden_sub_combos, state)?);
        pos = end;
    }
    result.push_str(&snippet[pos..]);
    Some(result)
}

#[derive(Clone, Copy)]
enum CaseChange {
    None,
    Upper,
    Lower,
    Trim,
}

fn evaluate_variable(variable: &str, host: &mut dyn VariableHost, forbidden: &HashSet<String>, state: &mut EvalState) -> Option<String> {
    let unchanged = || Some(format!("#{{{variable}}}"));
    match variable {
        "clipboard" => return Some(literal(host.clipboard_text())),
        "discordemoji" => return Some(discord_emojis(&host.clipboard_text())),
        "date" => return Some(host.system_date(&host.now(), DateKind::Date)),
        "time" => return Some(host.system_date(&host.now(), DateKind::Time)),
        "dateTime" => return Some(host.system_date(&host.now(), DateKind::DateTime)),
        _ => {}
    }
    if let Some(param) = variable.strip_prefix("dateTime:") {
        let (shift, format) = qtdate::split_shift_and_format(param);
        let now = host.now();
        let dt = match shift {
            Some(s) => qtdate::apply_shift(now, s),
            None => now,
        };
        let format = resolve_escaping(format);
        return Some(if format.is_empty() {
            host.system_date(&dt, DateKind::DateTime)
        } else {
            qtdate::format_qt(&dt, &format, host.locale())
        });
    }
    for (prefix, change) in
        [("combo:", CaseChange::None), ("upper:", CaseChange::Upper), ("lower:", CaseChange::Lower), ("trim:", CaseChange::Trim)]
    {
        if let Some(param) = variable.strip_prefix(prefix) {
            let keyword = resolve_escaping(param);
            if forbidden.contains(&keyword) {
                return unchanged();
            }
            let snippets = host.snippets_for_keyword(&keyword);
            if snippets.is_empty() {
                return unchanged();
            }
            let index = if snippets.len() > 1 { host.random_index(snippets.len()) % snippets.len() } else { 0 };
            let mut sub_forbidden = forbidden.clone();
            sub_forbidden.insert(keyword);
            let text = evaluate_snippet(&snippets[index], host, &sub_forbidden, state)?;
            return Some(match change {
                CaseChange::None => text,
                CaseChange::Upper => text.to_uppercase(),
                CaseChange::Lower => text.to_lowercase(),
                CaseChange::Trim => text.trim().to_string(),
            });
        }
    }
    if let Some(description) = variable.strip_prefix("input:") {
        if let Some(v) = state.known_inputs.get(description) {
            return Some(literal(v.clone()));
        }
        let value = host.prompt_input(&resolve_escaping(description))?;
        state.known_inputs.insert(description.to_string(), value.clone());
        return Some(literal(value));
    }
    if let Some(name) = variable.strip_prefix("envVar:") {
        return Some(literal(host.env_var(name)));
    }
    if let Some(param) = variable.strip_prefix("powershell:") {
        let (path, timeout) = split_powershell_param(param);
        if path.is_empty() {
            return Some(String::new());
        }
        return Some(literal(host.run_powershell(&resolve_escaping(path), timeout)));
    }
    unchanged()
}

/// `path[:timeoutMs]`. The default timeout is 10 seconds; 0 means no timeout.
pub fn split_powershell_param(param: &str) -> (&str, u64) {
    if let Some(idx) = param.rfind(':') {
        let (path, digits) = (&param[..idx], &param[idx + 1..]);
        if !path.is_empty() && !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
            return (path, digits.parse().unwrap_or(10_000));
        }
    }
    (param, 10_000)
}

/// The "secret" `#{discordemoji}` variable: the clipboard text spelled with Discord emojis.
pub fn discord_emojis(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        let l = c.to_ascii_lowercase();
        if l.is_ascii_lowercase() {
            out.push_str(&format!(":regional_indicator_{l}: "));
            continue;
        }
        let s = match c {
            '0' => ":zero: ",
            '1' => ":one: ",
            '2' => ":two: ",
            '3' => ":three: ",
            '4' => ":four: ",
            '5' => ":five: ",
            '6' => ":six: ",
            '7' => ":seven: ",
            '8' => ":eight: ",
            '9' => ":nine: ",
            '!' => ":exclamation: ",
            '?' => ":question: ",
            _ => "        ",
        };
        out.push_str(s);
    }
    out
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chrono::TimeZone;

    #[derive(Default)]
    pub struct MockHost {
        pub clipboard: String,
        pub combos: Vec<(String, String)>,
        pub inputs: Vec<Option<String>>,
        pub prompts: Vec<String>,
    }

    impl VariableHost for MockHost {
        fn clipboard_text(&mut self) -> String {
            self.clipboard.clone()
        }
        fn prompt_input(&mut self, description: &str) -> Option<String> {
            self.prompts.push(description.to_string());
            if self.inputs.is_empty() { Some(String::new()) } else { self.inputs.remove(0) }
        }
        fn snippets_for_keyword(&self, keyword: &str) -> Vec<String> {
            self.combos.iter().filter(|(k, _)| k == keyword).map(|(_, s)| s.clone()).collect()
        }
        fn run_powershell(&mut self, script_path: &str, timeout_ms: u64) -> String {
            format!("ps({script_path},{timeout_ms})")
        }
        fn now(&self) -> DateTime<Local> {
            Local.with_ymd_and_hms(2018, 1, 30, 9, 5, 7).unwrap()
        }
        fn random_index(&mut self, _len: usize) -> usize {
            0
        }
    }

    fn eval(host: &mut MockHost, s: &str) -> Option<String> {
        evaluate_snippet(s, host, &HashSet::new(), &mut EvalState::default())
    }

    #[test]
    fn find_variables() {
        assert_eq!(find_variable("a#{b}c", 0), Some((1, 5, "b")));
        assert_eq!(find_variable("#{a\\}b}", 0), Some((0, 7, "a\\}b")));
        assert_eq!(find_variable("#{a\nb} #{c}", 0), Some((7, 11, "c")));
        assert_eq!(find_variable("no var", 0), None);
        assert_eq!(find_variable("#{}", 0), Some((0, 3, "")));
    }

    #[test]
    fn documentation_example() {
        let mut h = MockHost { combos: vec![("xxc".into(), "Lausanne".into())], ..Default::default() };
        assert_eq!(eval(&mut h, "#{dateTime:yyyy-MM-dd}, #{combo:xxc}, Switzerland").unwrap(), "2018-01-30, Lausanne, Switzerland");
    }

    #[test]
    fn combos_and_case() {
        let mut h = MockHost {
            combos: vec![("xxfn".into(), "John Fitzgerald".into()), ("xxln".into(), "Kennedy".into()), ("sp".into(), "  x  ".into())],
            ..Default::default()
        };
        assert_eq!(eval(&mut h, "#{lower:xxfn} #{upper:xxln}").unwrap(), "john fitzgerald KENNEDY");
        assert_eq!(eval(&mut h, "[#{trim:sp}]").unwrap(), "[x]");
        assert_eq!(eval(&mut h, "#{combo:nope}").unwrap(), "#{combo:nope}");
    }

    #[test]
    fn recursion_is_stopped() {
        let mut h = MockHost { combos: vec![("a".into(), "A#{combo:b}".into()), ("b".into(), "B#{combo:a}".into())], ..Default::default() };
        assert_eq!(eval(&mut h, "#{combo:a}").unwrap(), "AB#{combo:a}");
    }

    #[test]
    fn unknown_and_special_variables_are_kept() {
        let mut h = MockHost::default();
        assert_eq!(eval(&mut h, "#{datetime} #{cursor} #{key:tab:2}").unwrap(), "#{datetime} #{cursor} #{key:tab:2}");
    }

    #[test]
    fn clipboard_and_env() {
        let mut h = MockHost { clipboard: "Hi!".into(), ..Default::default() };
        assert_eq!(eval(&mut h, "<#{clipboard}>").unwrap(), "<Hi!>");
        assert_eq!(eval(&mut h, "#{discordemoji}").unwrap(), ":regional_indicator_h: :regional_indicator_i: :exclamation: ");
        unsafe { std::env::set_var("MANTICE_TEST_VAR", "value") };
        assert_eq!(eval(&mut h, "#{envVar:MANTICE_TEST_VAR}").unwrap(), "value");
    }

    #[test]
    fn inputs_are_asked_once() {
        let mut h = MockHost { inputs: vec![Some("Ada".into())], ..Default::default() };
        let mut state = EvalState::default();
        let r = evaluate_snippet("#{input:Name} and #{input:Name\\}}", &mut h, &HashSet::new(), &mut state);
        assert_eq!(h.prompts, vec!["Name".to_string(), "Name}".to_string()]);
        assert_eq!(r.unwrap(), "Ada and ");
        let mut h = MockHost { inputs: vec![Some("Ada".into())], ..Default::default() };
        assert_eq!(eval(&mut h, "#{input:Name} #{input:Name}").unwrap(), "Ada Ada");
        assert_eq!(h.prompts.len(), 1);
        let mut h = MockHost { inputs: vec![None], ..Default::default() };
        assert_eq!(eval(&mut h, "#{input:Name}"), None);
    }

    #[test]
    fn date_shift() {
        let mut h = MockHost::default();
        assert_eq!(eval(&mut h, "#{dateTime:+1d-1y+2h:yyyy-MM-dd hh:mm:ss}").unwrap(), "2017-01-31 11:05:07");
        assert_eq!(eval(&mut h, "#{dateTime:'Week' ww}").unwrap(), "Week 05");
    }

    #[test]
    fn powershell_param() {
        assert_eq!(split_powershell_param(r"C:\Temp\script.ps1"), (r"C:\Temp\script.ps1", 10_000));
        assert_eq!(split_powershell_param(r"C:\Temp\script.ps1:5000"), (r"C:\Temp\script.ps1", 5000));
        assert_eq!(split_powershell_param(r"C:\Temp\script.ps1:0"), (r"C:\Temp\script.ps1", 0));
        let mut h = MockHost::default();
        assert_eq!(eval(&mut h, r"#{powershell:C:\\x.ps1:20}").unwrap(), r"ps(C:\x.ps1,20)");
    }
}
