use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Remove all files (not sub dirs and their files) that are not flagged as keep
/// from the given directory
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
/// let cleaner = DirCleaner::new(&PathBuf::from("/froboz"));
/// cleaner.keep(&PathBuf::from("foo.svg"));
/// fs::write(&PathBuf::from("/froboz/newfile.png"), "");
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
    pub fn new(img_path: &PathBuf) -> DirCleaner {
        DirCleaner {
            files: DirCleaner::get_files(img_path),
        }
    }

    pub fn keep(&mut self, img_path: &PathBuf) {
        info!("DirCleaner - Keeping {}", img_path.to_string_lossy());
        self.files.remove(img_path);
    }

    fn get_files(img_path: &PathBuf) -> HashSet<PathBuf> {
        let mut files = HashSet::new();
        match std::fs::read_dir(img_path.as_path()) {
            Err(e) => {
                error!(
                    "DirCleaner - Failed to list directory contents of {} ({}).",
                    img_path.to_string_lossy(),
                    e
                );
            }
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        // Here, `entry` is a `DirEntry`.
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                files.insert(entry.path());
                                info!(
                                    "DirCleaner - Found existing file {}",
                                    entry.path().to_string_lossy()
                                );
                            }
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
            if let Err(e) = fs::remove_file(&file) {
                error!(
                    "DirCleaner - Failed to remove obsolete image file '{}' ({}).",
                    file.to_string_lossy(),
                    e
                );
            } else {
                debug!("DirCleaner - Removed file {}", file.to_string_lossy());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    fn get_file_path(target_path: &PathBuf, filename: &PathBuf) -> PathBuf {
        let mut p = target_path.clone();
        p.push(filename);
        p
    }

    fn seed_dir(target_path: &PathBuf) -> HashSet<PathBuf> {
        let mut created_files = HashSet::new();
        let mut create_file = |filename: &PathBuf, skip: bool| {
            let p = get_file_path(target_path, filename);
            if !skip {
                created_files.insert(p.clone());
            }

            fs::write(&p, "").is_ok()
        };

        //Preparation
        assert!(create_file(&PathBuf::from("foo.txt"), false));
        assert!(create_file(&PathBuf::from("bar.txt"), false));
        assert!(create_file(&PathBuf::from("baz.txt"), false));
        assert!(std::fs::create_dir(get_file_path(target_path, &PathBuf::from("skipped"))).is_ok());
        assert!(create_file(&PathBuf::from("skipped/skippedfile.txt"), true));

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

            let mut keep = |file_name: PathBuf| {
                let p = get_file_path(&target_path, &file_name);
                cleaner.keep(&p);
                expected_files.insert(p);
            };

            keep(PathBuf::from("foo.txt"));
            keep(PathBuf::from("baz.txt"));
        }

        // The directory should now be empty
        assert_eq!(expected_files, DirCleaner::get_files(&target_path));
    }
}
