//! Location of the data files. The Portable Edition (a `Portable.bin` file next to the executable) stores
//! everything in a `Data` sub-folder, like Beeftext.

use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "Mantice";
/// The former name of the program: its data folder is moved to the new one on the first launch.
const LEGACY_APP_NAME: &str = "ExpanderCraft";
pub const PORTABLE_BEACON: &str = "Portable.bin";

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub portable: bool,
}

impl AppPaths {
    pub fn detect() -> Self {
        let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
        if let Some(dir) = &exe_dir
            && dir.join(PORTABLE_BEACON).exists()
        {
            return Self { data_dir: dir.join("Data"), portable: true };
        }
        let base = directories::BaseDirs::new()
            .map(|b| b.data_local_dir().to_path_buf())
            .unwrap_or_else(|| exe_dir.clone().unwrap_or_else(|| PathBuf::from(".")));
        Self { data_dir: base.join(APP_NAME), portable: false }
    }

    /// Moves the data folder of ExpanderCraft (the former name of the program) to the folder of Mantice, once:
    /// nothing happens when the new folder already exists, or in the portable edition. Call it before anything
    /// reads the data folder. Returns the old folder when it was moved or copied, `None` when there was nothing
    /// to do. The result is for the log, which is not open yet at that point.
    pub fn migrate_legacy_data_dir(&self) -> Option<std::io::Result<PathBuf>> {
        if self.portable {
            return None;
        }
        let old = self.data_dir.parent()?.join(LEGACY_APP_NAME);
        move_dir(&old, &self.data_dir).map(|result| result.map(|()| old))
    }

    pub fn ensure_exists(&self) {
        if let Err(e) = std::fs::create_dir_all(&self.data_dir) {
            eprintln!("Could not create the data folder '{}': {e}", self.data_dir.display());
        }
    }

    pub fn settings_file(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }
    pub fn log_file(&self) -> PathBuf {
        self.data_dir.join("log.txt")
    }
    pub fn last_use_file(&self) -> PathBuf {
        self.data_dir.join(crate::library::LAST_USE_FILE_NAME)
    }
    pub fn emoji_last_use_file(&self) -> PathBuf {
        self.data_dir.join("emojiLastUse.json")
    }
    pub fn excluded_apps_file(&self) -> PathBuf {
        self.data_dir.join("excludedApps.json")
    }
    pub fn sensitive_apps_file(&self) -> PathBuf {
        self.data_dir.join("sensitiveApps.json")
    }
    pub fn emoji_excluded_apps_file(&self) -> PathBuf {
        self.data_dir.join("emojiExcludedApps.json")
    }
    pub fn default_backup_dir(&self) -> PathBuf {
        self.data_dir.join("Backup")
    }
    pub fn instance_socket(&self) -> PathBuf {
        self.data_dir.join("mantice.sock")
    }
}

/// Possible locations of an existing Beeftext combo list, used to offer a migration on first launch.
pub fn beeftext_library_candidates() -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Some(b) = directories::BaseDirs::new() {
        for base in [b.data_local_dir(), b.data_dir()] {
            result.push(base.join("beeftext.org").join("Beeftext").join(crate::library::LIBRARY_FILE_NAME));
            result.push(base.join("Beeftext").join(crate::library::LIBRARY_FILE_NAME));
        }
    }
    result.into_iter().filter(|p| p.is_file()).collect()
}

/// Renames `old` to `new`. `None` when there is nothing to do: no `old` folder, or `new` exists already.
fn move_dir(old: &Path, new: &Path) -> Option<std::io::Result<()>> {
    if new.exists() || !old.is_dir() {
        return None;
    }
    // When the rename fails (a file of the old folder is open in a running ExpanderCraft) copy instead, and leave
    // the old folder where it is. A half copy is removed, so that the next launch tries again.
    Some(std::fs::rename(old, new).or_else(|_| {
        copy_dir(old, new).inspect_err(|_| {
            let _ = std::fs::remove_dir_all(new);
        })
    }))
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_folder_is_moved_once_and_copied_as_a_fallback() {
        let root = std::env::temp_dir().join(format!("mantice-paths-test-{}", std::process::id()));
        let (old, new, copy) = (root.join("old"), root.join("new"), root.join("copy"));
        std::fs::create_dir_all(old.join("Backup")).unwrap();
        std::fs::write(old.join("settings.json"), "{}").unwrap();
        std::fs::write(old.join("Backup").join("a.json"), "[]").unwrap();

        copy_dir(&old, &copy).unwrap();
        assert!(copy.join("settings.json").is_file() && copy.join("Backup").join("a.json").is_file());
        assert!(old.is_dir(), "a copy leaves the old folder in place");

        assert!(move_dir(&old, &new).unwrap().is_ok());
        assert!(new.join("settings.json").is_file() && new.join("Backup").join("a.json").is_file());
        assert!(!old.exists());

        assert!(move_dir(&old, &new).is_none(), "no old folder: nothing to do");
        std::fs::create_dir_all(&old).unwrap();
        assert!(move_dir(&old, &new).is_none(), "the new folder exists: nothing to do");

        std::fs::remove_dir_all(&root).unwrap();
    }
}
