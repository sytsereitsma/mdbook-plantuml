use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Remove all files (not sub dirs and their files) that are not flagged as keep
/// from the given directory. Used for removing stale cached image files.
///
/// # Example:
/// Given the contents of the directory froboz is the following
/// froboz/
/// ├── foo.svg
/// ├── bar.png
/// ├── baz.txt
/// └── sub/
///     └── some.svg
///
/// Then, after running the following code:
///
/// ```rust,ignore
/// let cleaner = DirCleaner::new(Path::new("/froboz"));
/// cleaner.keep(Path::new("foo.svg"));
/// fs::write(Path::new("/froboz/newfile.png"), "");
/// ```
/// The directory contents will be the following:
/// froboz/
/// ├── foo.svg
/// ├── newfile.png
/// └── sub/
///     └── some.svg
pub struct DirCleaner {
    files: HashSet<PathBuf>,
}

impl DirCleaner {
    pub fn new(img_path: &Path) -> Self {
        Self {
            files: Self::get_files(img_path),
        }
    }

    pub fn keep(&mut self, img_path: &Path) {
        log::debug!("DirCleaner - Keeping {}", img_path.to_string_lossy());
        self.files.remove(img_path);
    }

    fn get_files(img_path: &Path) -> HashSet<PathBuf> {
        let mut files = HashSet::new();
        match std::fs::read_dir(img_path) {
            Err(e) => {
                log::error!(
                    "DirCleaner - Failed to list directory contents of {} ({}).",
                    img_path.to_string_lossy(),
                    e
                );
            }
            Ok(entries) => {
                for entry in entries.flatten() {
                    // Here, `entry` is a `DirEntry`.
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            files.insert(entry.path());
                            log::debug!(
                                "DirCleaner - Found existing file {}",
                                entry.path().to_string_lossy()
                            );
                        }
                    }
                }
            }
        }

        files
    }
}

impl Drop for DirCleaner {
    fn drop(&mut self) {
        for file in &self.files {
            if let Err(e) = fs::remove_file(file) {
                log::error!(
                    "DirCleaner - Failed to remove obsolete image file '{}' ({}).",
                    file.to_string_lossy(),
                    e
                );
            } else {
                log::debug!("DirCleaner - Removed file {}", file.to_string_lossy());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    fn get_file_path(target_path: &Path, filename: &Path) -> PathBuf {
        let mut p = target_path.to_path_buf();
        p.push(filename);
        p
    }

    fn seed_dir(target_path: &Path) -> HashSet<PathBuf> {
        let mut created_files = HashSet::new();
        let mut create_file = |filename: &Path, skip: bool| {
            let p = get_file_path(target_path, filename);
            if !skip {
                created_files.insert(p.clone());
            }

            fs::write(&p, "").is_ok()
        };

        // Preparation
        assert!(create_file(Path::new("foo.txt"), false));
        assert!(create_file(Path::new("bar.txt"), false));
        assert!(create_file(Path::new("baz.txt"), false));
        assert!(std::fs::create_dir(get_file_path(target_path, Path::new("skipped"))).is_ok());
        assert!(create_file(Path::new("skipped/skippedfile.txt"), true));

        created_files
    }

    #[test]
    fn initializes_file_list_with_files_from_input_dir() {
        let dir = tempdir().unwrap();
        let target_path = dir.path().to_path_buf();
        let expected_files = seed_dir(&target_path);

        let cleaner = DirCleaner::new(&target_path);
        assert_eq!(expected_files, cleaner.files);
    }

    #[test]
    fn removes_unused_files() {
        let dir = tempdir().unwrap();
        let target_path = dir.path().to_path_buf();

        {
            seed_dir(&target_path);
            DirCleaner::new(&target_path);
        }

        // The directory should now be empty
        assert!(DirCleaner::get_files(&target_path).is_empty());
    }

    #[test]
    fn keeps_used_files() {
        let dir = tempdir().unwrap();
        let target_path = dir.path().to_path_buf();
        let mut expected_files = HashSet::new();

        {
            seed_dir(&target_path);
            let mut cleaner = DirCleaner::new(&target_path);

            let mut keep = |file_name: &Path| {
                let p = get_file_path(&target_path, file_name);
                cleaner.keep(&p);
                expected_files.insert(p);
            };

            keep(Path::new("foo.txt"));
            keep(Path::new("baz.txt"));
        }

        // The directory should now be empty
        assert_eq!(expected_files, DirCleaner::get_files(&target_path));
    }
}
