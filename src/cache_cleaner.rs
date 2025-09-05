use std::collections::HashSet;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

/// To prevent the cache dir to contain obsolete files, we use this struct to track
/// which files should be kept. When this struct is dropped, all files in the
/// directory that are not marked as kept will be removed (non recursive).
///
/// # Example:
/// Given the contents of the directory .mdbook-plantuml-images is the following
/// .mdbook-plantuml-cache/
/// ├── foo.svg
/// ├── bar.png
/// ├── baz.txt
/// └── sub/
///     └── some.svg
///
/// Then, after running the following code:
///
/// ```rust, ignore
/// let mut cleaner = CacheCleaner::new(Path::new("/froboz"));
/// cleaner.keep(Path::new("foo.svg"));
/// cleaner.drop();
/// ```
/// The directory contents will be the following:
/// froboz/
/// ├── foo.svg
/// └── sub/
///     └── some.svg
pub struct CacheCleaner {
    /// All files we want to keep
    files_to_keep: HashSet<PathBuf>,
    /// The directory we're cleaning (only at root level, no recursion)
    dir: PathBuf,
}

impl CacheCleaner {
    pub fn new(img_path: &Path) -> Self {
        Self {
            files_to_keep: HashSet::new(),
            dir: img_path.to_path_buf(),
        }
    }

    pub fn keep(&mut self, img_path: &Path) {
        log::debug!("CacheCleaner - Keeping {:?}", img_path);
        self.files_to_keep.insert(img_path.to_path_buf());
    }
}

impl Drop for CacheCleaner {
    fn drop(&mut self) {
        // List all files at the directory and remove those that are not in self.files_to_keep
        // Note: we do not recurse into sub dirs
        match std::fs::read_dir(&self.dir) {
            Err(e) => {
                log::error!(
                    "CacheCleaner - Failed to list directory contents of {:?} ({}).",
                    self.dir,
                    e
                );
            }
            Ok(entries) => {
                // Filter all directory entries to only files that are not in self.files_to_keep
                let should_delete = |entry: std::io::Result<DirEntry>| {
                    let entry = match entry {
                        Err(e) => {
                            log::error!(
                                "CacheCleaner - Failed to process directory entry in {:?} ({}).",
                                self.dir,
                                e
                            );
                            return None;
                        }
                        Ok(e) => e,
                    };

                    if let Ok(file_type) = entry.file_type()
                        && file_type.is_file()
                        && !self.files_to_keep.contains(&entry.path())
                    {
                        Some(entry)
                    } else {
                        None
                    }
                };

                for entry in entries.filter_map(should_delete) {
                    if let Err(e) = fs::remove_file(entry.path()) {
                        log::error!(
                            "CacheCleaner - Failed to remove obsolete image file '{:?}' ({}).",
                            entry.path(),
                            e
                        );
                    } else {
                        log::debug!("CacheCleaner - Removed file {:?}", entry.path());
                    }
                }
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

    fn file_path(target_path: &Path, filename: &Path) -> PathBuf {
        let mut p = target_path.to_path_buf();
        p.push(filename);
        p
    }

    fn seed_dir(target_path: &Path) -> HashSet<PathBuf> {
        let mut created_files = HashSet::new();
        let mut create_file = |filename: &Path, skip: bool| {
            let p = file_path(target_path, filename);
            if !skip {
                created_files.insert(p.clone());
            }

            fs::write(&p, "").is_ok()
        };

        // Preparation
        assert!(create_file(Path::new("foo.txt"), false));
        assert!(create_file(Path::new("bar.txt"), false));
        assert!(create_file(Path::new("baz.txt"), false));
        assert!(std::fs::create_dir(file_path(target_path, Path::new("skipped"))).is_ok());
        assert!(create_file(Path::new("skipped/skippedfile.txt"), true));

        created_files
    }

    fn remaining_files_in_dir(target_path: &Path) -> HashSet<PathBuf> {
        let mut found_files = HashSet::new();

        if let Ok(entries) = std::fs::read_dir(target_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        // Strip the target_path prefix
                        let path = entry.path();
                        let stripped = path.strip_prefix(target_path).unwrap();
                        found_files.insert(stripped.to_path_buf());
                    }
                }
            }
        }

        found_files
    }

    #[test]
    fn removes_unused_files() {
        let dir = tempdir().unwrap();
        let target_path = dir.path().to_path_buf();

        {
            seed_dir(&target_path);
            CacheCleaner::new(&target_path);
        }

        // The directory should now be empty
        assert!(remaining_files_in_dir(&target_path).is_empty());
    }

    #[test]
    fn keeps_used_files() {
        let dir = tempdir().unwrap();
        let target_path = dir.path().to_path_buf();
        let mut expected_files = HashSet::new();

        {
            seed_dir(&target_path);
            let mut cleaner = CacheCleaner::new(&target_path);

            let mut keep = |file_name: &Path| {
                let p = file_path(&target_path, file_name);
                cleaner.keep(&p);
                expected_files.insert(file_name.to_path_buf());
            };

            keep(Path::new("foo.txt"));
            keep(Path::new("baz.txt"));
        }

        // The directory should now be empty
        assert_eq!(expected_files, remaining_files_in_dir(&target_path));
    }
}
