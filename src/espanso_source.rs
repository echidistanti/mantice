//! Where an espanso package comes from, and the work of getting its files: a folder, a `.zip` or a `.yml` file on
//! this computer, or the hub (https://hub.espanso.org) by the name of the package or by a link.
//!
//! [`load`] does everything, from the text typed by the user to the translated [`Package`]; it blocks (files, network,
//! YAML), so the interface calls it from a thread of its own.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;

use crate::espanso::{self, Package, ReadError};
use crate::i18n::tr;

/// The list of the packages of the hub (espanso reads the same file).
const INDEX_URL: &str = "https://github.com/espanso/hub/releases/latest/download/package_index.json";
/// What can be downloaded (the biggest package of the hub is 11 MB of text, a few MB zipped).
const MAX_DOWNLOAD: usize = 64 << 20;
/// What a single file of a package can weigh, and what all of them can weigh together.
const MAX_FILE: u64 = 64 << 20;
const MAX_TOTAL: u64 = 256 << 20;

#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    /// A folder, a `.zip` file or a `.yml` file of this computer.
    Path(PathBuf),
    /// A package of the hub, by name.
    HubPackage(String),
    /// A `.zip` or `.yml` file to download.
    Url(String),
}

/// What the text typed by the user refers to. It can be the name of a package (`basic-emojis`), the command that
/// the hub shows (`espanso install basic-emojis`), a link to the page of a package, a link to a `.zip` or `.yml`
/// file, or the path of a file or a folder.
pub fn parse_source(input: &str) -> Result<Source> {
    let input = input.trim().trim_matches('"').trim();
    let input = input.strip_prefix("espanso install ").map_or(input, str::trim);
    let path = Path::new(input);
    if path.is_absolute() && path.exists() {
        return Ok(Source::Path(path.to_path_buf()));
    }
    if let Some(rest) = input.strip_prefix("https://") {
        return parse_link(rest);
    }
    if input.starts_with("http://") {
        bail!(tr("error-espanso-not-https"));
    }
    if is_package_name(input) {
        return Ok(Source::HubPackage(input.to_ascii_lowercase()));
    }
    bail!(tr!("error-espanso-unknown-source", input = input))
}

/// The names of the packages of the hub are lower case letters, digits, `-` and `_`.
fn is_package_name(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphanumeric()) && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `rest` is a link without its `https://`.
fn parse_link(rest: &str) -> Result<Source> {
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    let host = host.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let hub_package = |name: &str| match is_package_name(name) {
        true => Ok(Source::HubPackage(name.to_ascii_lowercase())),
        false => Err(anyhow!(tr("error-espanso-unsupported-link"))),
    };

    // hub.espanso.org/basic-emojis
    if host == "hub.espanso.org" {
        return match segments.first() {
            None | Some(&"search") => bail!(tr("error-espanso-search-page")),
            Some(name) => hub_package(name),
        };
    }
    // github.com/espanso/hub/tree/main/packages/basic-emojis/0.1.0
    if host == "github.com"
        && let [owner, repo, ..] = segments.as_slice()
        && owner.eq_ignore_ascii_case("espanso")
        && repo.eq_ignore_ascii_case("hub")
        && let Some(at) = segments.iter().position(|s| *s == "packages")
        && let Some(name) = segments.get(at + 1)
    {
        return hub_package(name);
    }
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".zip") || lower.ends_with(".yml") || lower.ends_with(".yaml") {
        // the page of a file of GitHub shows it inside HTML: the file is on raw.githubusercontent.com
        if host == "github.com"
            && let [owner, repo, "blob", rest @ ..] = segments.as_slice()
            && !rest.is_empty()
        {
            return Ok(Source::Url(format!("https://raw.githubusercontent.com/{owner}/{repo}/{}", rest.join("/"))));
        }
        return Ok(Source::Url(format!("https://{host}/{}", path.trim_start_matches('/'))));
    }
    bail!(tr("error-espanso-unsupported-link"))
}

/// From the text typed by the user to the package.
pub fn load(input: &str) -> Result<Package> {
    let raw = match parse_source(input)? {
        Source::Path(path) => read_path(&path)?,
        Source::HubPackage(name) => {
            let url = archive_url(&download(INDEX_URL)?, &name)?;
            read_zip(&download(&url)?, &name)?
        }
        Source::Url(url) => {
            let name = url.rsplit('/').next().unwrap_or_default().rsplit_once('.').map_or("", |(stem, _)| stem).to_string();
            let bytes = download(&url)?;
            if url.to_ascii_lowercase().ends_with(".zip") {
                read_zip(&bytes, &name)?
            } else {
                let mut files = Collector::default();
                files.add("package.yml", &bytes)?;
                files.finish(&name)?
            }
        }
    };
    if raw.files.is_empty() {
        bail!(tr("error-espanso-no-files"));
    }
    espanso::read_package(raw.manifest.as_deref(), &raw.files, &raw.name).map_err(|e| match e {
        ReadError::Yaml(detail) => anyhow!(tr!("error-espanso-yaml", detail = detail)),
        ReadError::NotAMatchFile => anyhow!(tr("error-espanso-no-matches")),
    })
}

fn download(url: &str) -> Result<Vec<u8>> {
    if !url.starts_with("https://") {
        bail!(tr("error-espanso-not-https"));
    }
    crate::platform::http_get(url, MAX_DOWNLOAD)
}

#[derive(Deserialize)]
struct Index {
    packages: Vec<IndexEntry>,
}

#[derive(Deserialize)]
struct IndexEntry {
    name: String,
    archive_url: String,
}

/// The link to the archive of a package, from the list of the hub. A name that is not there is answered with
/// the names that contain it.
fn archive_url(index: &[u8], name: &str) -> Result<String> {
    let index: Index = serde_json::from_slice(index).with_context(|| tr("error-espanso-index"))?;
    if let Some(entry) = index.packages.iter().find(|e| e.name.eq_ignore_ascii_case(name)) {
        return Ok(entry.archive_url.clone());
    }
    let similar: Vec<&str> = index.packages.iter().map(|e| e.name.as_str()).filter(|n| n.contains(name)).take(5).collect();
    if similar.is_empty() {
        bail!(tr!("error-espanso-not-in-hub", name = name))
    }
    bail!(tr!("error-espanso-not-in-hub-similar", name = name, similar = similar.join(", ")))
}

// ---------------------------------------------------------------------------------------------------------------------
// The files of a package
// ---------------------------------------------------------------------------------------------------------------------

struct Raw {
    name: String,
    manifest: Option<String>,
    /// The match files, as (path inside the package, text).
    files: Vec<(String, String)>,
}

#[derive(Default)]
struct Collector {
    manifest: Option<String>,
    files: Vec<(String, String)>,
    total: u64,
}

impl Collector {
    /// Is it the `_manifest.yml` or a match file? Espanso loads every `.yml` that does not start with `_`.
    fn wanted(path: &str) -> bool {
        let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
        name == "_manifest.yml" || (!name.starts_with('_') && (name.ends_with(".yml") || name.ends_with(".yaml")))
    }

    fn add(&mut self, path: &str, bytes: &[u8]) -> Result<()> {
        self.total += bytes.len() as u64;
        if bytes.len() as u64 > MAX_FILE || self.total > MAX_TOTAL {
            bail!(tr("error-espanso-too-big"));
        }
        let text = String::from_utf8_lossy(bytes);
        let text = text.trim_start_matches('\u{feff}').to_string();
        if path.rsplit('/').next().is_some_and(|n| n.eq_ignore_ascii_case("_manifest.yml")) {
            self.manifest.get_or_insert(text);
        } else {
            self.files.push((path.to_string(), text));
        }
        Ok(())
    }

    /// `package.yml` comes first (when two files have the same trigger the first one is the one that counts).
    fn finish(mut self, name: &str) -> Result<Raw> {
        self.files.sort_by(|a, b| {
            let first = |p: &str| p.rsplit('/').next() != Some("package.yml");
            (first(&a.0), &a.0).cmp(&(first(&b.0), &b.0))
        });
        Ok(Raw { name: name.to_string(), manifest: self.manifest, files: self.files })
    }
}

fn read_path(path: &Path) -> Result<Raw> {
    let stem = |p: &Path| p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    if path.is_dir() {
        return read_dir(path, &path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default());
    }
    let open = || tr!("error-open-file", path = path.display().to_string());
    if std::fs::metadata(path).with_context(open)?.len() > MAX_FILE {
        bail!(tr("error-espanso-too-big"));
    }
    let bytes = std::fs::read(path).with_context(open)?;
    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("zip")) {
        return read_zip(&bytes, &stem(path));
    }
    // a single match file: the manifest next to it, if there is one, tells what the package is
    let folder = path.parent();
    let mut files = Collector::default();
    if let Some(manifest) = folder.map(|f| f.join("_manifest.yml")).and_then(|m| std::fs::read(m).ok()) {
        files.add("_manifest.yml", &manifest)?;
    }
    files.add("package.yml", &bytes)?;
    let name = match stem(path).as_str() {
        "package" => folder.and_then(Path::file_name).map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
        other => other.to_string(),
    };
    files.finish(&name)
}

fn read_dir(root: &Path, name: &str) -> Result<Raw> {
    let mut files = Collector::default();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let entries = std::fs::read_dir(&folder).with_context(|| tr!("error-open-file", path = folder.display().to_string()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            // links are not followed (they can loop)
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_dir() {
                folders.push(path);
                continue;
            }
            let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
            if !kind.is_file() || !Collector::wanted(&relative) {
                continue;
            }
            let file = std::fs::File::open(&path).with_context(|| tr!("error-open-file", path = path.display().to_string()))?;
            let mut bytes = Vec::new();
            file.take(MAX_FILE + 1).read_to_end(&mut bytes).with_context(|| tr!("error-open-file", path = path.display().to_string()))?;
            files.add(&relative, &bytes)?;
        }
    }
    files.finish(name)
}

fn read_zip(bytes: &[u8], name: &str) -> Result<Raw> {
    let zip_error = |e: zip::result::ZipError| anyhow!(tr!("error-espanso-zip", detail = e.to_string()));
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(zip_error)?;
    let mut files = Collector::default();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(zip_error)?;
        let path = entry.name().map_err(zip_error)?.replace('\\', "/");
        if entry.is_dir() || !Collector::wanted(&path) {
            continue;
        }
        let mut data = Vec::new();
        Read::take(&mut entry, MAX_FILE + 1)
            .read_to_end(&mut data)
            .map_err(|e| anyhow!(tr!("error-espanso-zip", detail = e.to_string())))?;
        files.add(&path, &data)?;
    }
    files.finish(name)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn url(text: &str) -> Source {
        parse_source(text).unwrap()
    }

    #[test]
    fn what_the_user_types() {
        assert_eq!(url("basic-emojis"), Source::HubPackage("basic-emojis".into()));
        assert_eq!(url("  Basic-Emojis "), Source::HubPackage("basic-emojis".into()));
        // the command that the hub shows
        assert_eq!(url("espanso install basic-emojis"), Source::HubPackage("basic-emojis".into()));
        assert_eq!(url("https://hub.espanso.org/basic-emojis"), Source::HubPackage("basic-emojis".into()));
        assert_eq!(url("https://hub.espanso.org/basic-emojis/?x=1#top"), Source::HubPackage("basic-emojis".into()));
        assert_eq!(url("https://github.com/espanso/hub/tree/main/packages/lorem/0.1.0"), Source::HubPackage("lorem".into()));
        assert_eq!(url("https://example.org/pack/my.zip?raw=1"), Source::Url("https://example.org/pack/my.zip".into()));
        // a file of GitHub is read from its raw version
        assert_eq!(
            url("https://github.com/me/snippets/blob/main/match/base.yml"),
            Source::Url("https://raw.githubusercontent.com/me/snippets/main/match/base.yml".into())
        );
    }

    #[test]
    fn what_cannot_be_loaded() {
        for bad in [
            "https://hub.espanso.org/search",
            "https://hub.espanso.org/",
            "http://example.org/a.zip",
            "https://example.org/page",
            "not a name!",
            "../x",
        ] {
            assert!(parse_source(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_path_that_exists_wins_over_a_name() {
        let dir = std::env::temp_dir();
        assert_eq!(parse_source(&format!("\"{}\"", dir.display())).unwrap(), Source::Path(dir));
    }

    #[test]
    fn the_package_is_found_in_the_index() {
        let index = br#"{"last_update": 1, "packages": [
            {"name": "basic-emojis", "archive_url": "https://github.com/espanso/hub/releases/latest/download/basic-emojis-0.1.0.zip", "version": "0.1.0"},
            {"name": "emojis-extra", "archive_url": "https://example.org/e.zip"}]}"#;
        assert!(archive_url(index, "BASIC-emojis").unwrap().ends_with("basic-emojis-0.1.0.zip"));
        let missing = archive_url(index, "emoji").unwrap_err().to_string();
        assert!(missing.contains("basic-emojis, emojis-extra"), "{missing}");
        assert!(archive_url(b"[]", "x").is_err());
    }

    fn zip_with(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, text) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(text.as_bytes()).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn a_zip_is_read_like_the_package_it_contains() {
        let manifest = "name: lorem\ntitle: Lorem\nversion: 0.2.0\n";
        let main = "\u{feff}matches:\n  - {trigger: ':a', replace: one}\n";
        let more = "matches:\n  - {trigger: ':b', replace: two}\n";
        let bytes = zip_with(&[
            ("more.yml", more),
            ("_manifest.yml", manifest),
            ("package.yml", main),
            ("README.md", "# x"),
            ("_private.yml", "x: ["),
        ]);
        let raw = read_zip(&bytes, "fallback").unwrap();
        // package.yml first, the manifest apart, and nothing else
        assert_eq!(raw.files.iter().map(|f| f.0.as_str()).collect::<Vec<_>>(), ["package.yml", "more.yml"]);
        let package = espanso::read_package(raw.manifest.as_deref(), &raw.files, &raw.name).unwrap();
        assert_eq!((package.name.as_str(), package.version.as_str()), ("lorem", "0.2.0"));
        assert_eq!(package.matches.iter().map(|m| m.keyword.as_str()).collect::<Vec<_>>(), [":a", ":b"]);
    }

    #[test]
    fn a_folder_and_a_file_are_read_too() {
        let root = std::env::temp_dir().join(format!("mantice-espanso-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("_manifest.yml"), "name: from-folder\n").unwrap();
        std::fs::write(root.join("package.yml"), "matches:\n  - {trigger: ':a', replace: one}\n").unwrap();
        std::fs::write(root.join("sub").join("other.yaml"), "matches:\n  - {trigger: ':b', replace: two}\n").unwrap();
        std::fs::write(root.join("notes.txt"), "not a match file").unwrap();

        let folder = read_path(&root).unwrap();
        assert_eq!(folder.files.len(), 2);
        assert!(folder.manifest.is_some());
        let file = read_path(&root.join("package.yml")).unwrap();
        // the manifest next to a package.yml tells what the package is; its name is the folder when there is none
        assert_eq!((file.files.len(), file.manifest.is_some()), (1, true));
        std::fs::remove_file(root.join("_manifest.yml")).unwrap();
        assert_eq!(read_path(&root.join("package.yml")).unwrap().name, root.file_name().unwrap().to_string_lossy());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn what_is_too_big_is_refused() {
        let mut files = Collector::default();
        assert!(files.add("package.yml", &vec![b' '; (MAX_FILE + 1) as usize]).is_err());
    }
}
