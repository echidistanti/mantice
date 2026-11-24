//! Automatic backups of the combo list (like Beeftext: one file per save, the 50 most recent are kept).

use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;

pub const MAX_BACKUP_FILES: usize = 50;

fn is_backup_file_name(name: &str) -> bool {
    // yyyyMMdd_HHmmsszzz_backup.json
    let Some(stem) = name.strip_suffix("_backup.json") else { return false };
    let parts: Vec<&str> = stem.split('_').collect();
    parts.len() == 2 && parts[0].len() == 8 && parts[1].len() == 9 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

pub fn backup_date(path: &Path) -> Option<NaiveDateTime> {
    let name = path.file_name()?.to_str()?;
    if !is_backup_file_name(name) {
        return None;
    }
    NaiveDateTime::parse_from_str(&name[..18], "%Y%m%d_%H%M%S%3f").ok()
}

/// Backup files, oldest first.
pub fn list(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.file_name().and_then(|n| n.to_str()).map(is_backup_file_name).unwrap_or(false))
        .collect();
    files.sort();
    files
}

/// Copy the current library file to the backup folder, then remove the oldest backups.
pub fn archive(library_file: &Path, dir: &Path) {
    if !library_file.exists() {
        return;
    }
    if let Err(e) = std::fs::create_dir_all(dir) {
        log::warn!("Could not create the backup folder '{}': {e}", dir.display());
        return;
    }
    let name = format!("{}_backup.json", chrono::Local::now().format("%Y%m%d_%H%M%S%3f"));
    let dst = dir.join(name);
    match std::fs::copy(library_file, &dst) {
        Ok(_) => log::info!("Backed up combo file to {}", dst.display()),
        Err(e) => log::warn!("Could not archive the combo file to {}: {e}", dst.display()),
    }
    cleanup(dir);
}

pub fn cleanup(dir: &Path) {
    let files = list(dir);
    if files.len() > MAX_BACKUP_FILES {
        for f in &files[..files.len() - MAX_BACKUP_FILES] {
            if let Err(e) = std::fs::remove_file(f) {
                log::warn!("Could not remove backup file {}: {e}", f.display());
            }
        }
    }
}

pub fn remove_all(dir: &Path) -> usize {
    list(dir).iter().filter(|f| std::fs::remove_file(f).is_ok()).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert!(is_backup_file_name("20240101_101010123_backup.json"));
        assert!(!is_backup_file_name("20240101_1010101_backup.json"));
        assert!(!is_backup_file_name("comboList.json"));
        let d = backup_date(Path::new("/x/20240102_030405678_backup.json")).unwrap();
        assert_eq!(d.format("%Y-%m-%d %H:%M:%S%.3f").to_string(), "2024-01-02 03:04:05.678");
    }

    #[test]
    fn archive_and_cleanup() {
        let dir = std::env::temp_dir().join(format!("mantice-backup-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let lib = dir.join("comboList.json");
        std::fs::write(&lib, "{}").unwrap();
        for i in 0..(MAX_BACKUP_FILES + 5) {
            std::fs::write(dir.join(format!("20200101_{:09}_backup.json", i)), "{}").unwrap();
        }
        archive(&lib, &dir);
        let files = list(&dir);
        assert_eq!(files.len(), MAX_BACKUP_FILES);
        assert!(backup_date(files.last().unwrap()).unwrap().format("%Y").to_string() != "2020");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
