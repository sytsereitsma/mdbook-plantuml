use failure::Error;
use serde_json::json;
use sha1;
use std::cell::Cell;
use std::clone::Clone;
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::string::String;

pub struct Cache {
    /// A map of (hashed plantuml code, used flag) pairs. The key is generated
    ///by format_key function, the sued flag is used to remove enries that are
    ///no longer referenced.
    entries: HashMap<String, Cell<bool>>,
    /// The directory where to store all the cache files
    cache_path: PathBuf,
    /// When true (default), removes unused entries from the cache when saving it
    clean_on_save: bool,
}

/// Formats the key name by computiong the sh1 checksum of the code
fn format_key(code_block: &String) -> String {
    sha1::Sha1::from(&code_block).hexdigest()
}

impl Cache {
    /// Creates a Cache instance using the provided path. If the path does not
    /// exist it is created.
    /// When the path exists the cache entries are loaded from the cache file.
    /// # Arguments
    /// * `cache_path` - The path where the cache should be stored/loaded from, typically <book_root_dir>/.mdbook-plantuml-cache
    pub fn new(cache_path: &PathBuf, clean_on_save: bool) -> Result<Cache, Error> {
        let mut cache = Cache {
            entries: HashMap::new(),
            cache_path: cache_path.clone(),
            clean_on_save: clean_on_save,
        };

        if !cache_path.exists() {
            if let Err(e) = fs::create_dir_all(cache_path) {
                bail!(
                    "Failed to create cache directory '{}' ({}).",
                    cache_path.to_string_lossy(),
                    e
                );
            }
        } else {
            cache.load_cache();
        }

        Ok(cache)
    }

    fn load_cache(&mut self) {
        let cache_file = self.cache_file_path();

        match fs::File::open(&cache_file) {
            Err(e) => error!(
                "Failed to open cache file '{}' ({})",
                cache_file.to_string_lossy(),
                e
            ),
            Ok(f) => {
                let reader = BufReader::new(f);
                match serde_json::from_reader(reader) {
                    Err(e) => error!(
                        "Failed to parse cache file '{}' ({})",
                        cache_file.to_string_lossy(),
                        e
                    ),
                    Ok(entries) => {
                        let vec_entries: Vec<String> = entries;
                        self.entries.clear();
                        for key in vec_entries {
                            self.entries.insert(key, Cell::new(false));
                        }
                    }
                }
            }
        }
    }

    /// Returns the full path to the cache (table) file, cache.json
    fn cache_file_path(&self) -> PathBuf {
        let mut p = self.cache_path.clone();
        p.push("cache.json");
        p
    }

    /// Get the full path of the cached image, returns None if the file does not
    /// exist
    /// # Arguments
    /// * `filename` - The filename of the cached image file
    fn get_image_path(&self, filename: &String) -> Option<PathBuf> {
        let img_path = {
            let mut p = self.cache_path.clone();
            p.push(filename);
            p
        };

        if img_path.is_file() {
            return Some(img_path);
        }

        None
    }

    /// Returns the path to the cached image for the given code block, or None when the image is not cached, or the code
    /// has changed.
    ///
    /// # Arguments
    /// * `code_block_src` - The present source of the code block, if it does not match with the cached code block None is returned
    pub fn get_entry(&self, code_block_src: &String) -> Option<PathBuf> {
        let key = format_key(code_block_src);
        if let Some(entry) = self.entries.get(&key) {
            let image_path = self.get_image_path(&key);
            entry.set(image_path.is_some());

            return image_path;
        }

        None
    }

    /// Adds a new entry to the cache, or updates an existing one.
    /// # Arguments
    /// * `code_block_src` - The present source of the code block, if it does not match with the cached code block None is returned
    /// * `image_path` - The path to the image to cache (a copy of the file will be saved in the cache directory)
    pub fn add_entry(&mut self, code_block_src: &String, image_path: &PathBuf) -> bool {
        let key = format_key(code_block_src);

        let cache_path = {
            let mut p = self.cache_path.clone();
            p.push(&key);
            p
        };

        let result;
        if let Err(e) = fs::copy(&image_path, &cache_path) {
            eprintln!(
                "Failed to copy source image file '{}' to '{}' ({}).",
                image_path.to_string_lossy(),
                cache_path.to_string_lossy(),
                e
            );
            result = false;
        } else {
            self.entries.insert(key, Cell::new(true));
            result = true;
        }

        result
    }

    /// Saves the cache file (simple JSON array of all the files)
    /// Removes the unused entries and their respective files
    fn save(&self) {
        let remove_entry_file = |file_path: Option<PathBuf>| {
            if let Some(p) = file_path {
                if let Err(e) = fs::remove_file(&p) {
                    error!(
                        "Failed to remove cache file '{}' ({}).",
                        p.to_string_lossy(),
                        e
                    );
                }
            }
        };

        let used_entries = {
            let mut entries = Vec::new();
            for (key, entry) in &self.entries {
                if entry.get() || !self.clean_on_save {
                    entries.push(key);
                } else {
                    remove_entry_file(self.get_image_path(key));
                }
            }
            entries
        };

        let cache_file = self.cache_file_path();
        if let Err(e) = fs::write(&cache_file, json!(used_entries).to_string()) {
            error!(
                "Failed to save cache to '{}' ({}).",
                cache_file.to_string_lossy(),
                e
            );
        }
    }
}

impl Drop for Cache {
    /// Save the cache to disk (cache.json in the cache dir)
    fn drop(&mut self) {
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::{tempdir, TempDir};

    struct TestContext {
        cached_image_dir: TempDir,
    }

    impl TestContext {
        fn new() -> TestContext {
            TestContext {
                cached_image_dir: tempdir().unwrap(),
            }
        }

        /// Create a cache bypassing the constructor
        fn create_cache(&self) -> Cache {
            Cache {
                cache_path: self.path_buf(),
                entries: HashMap::new(),
                clean_on_save: true,
            }
        }

        fn path_buf(&self) -> PathBuf {
            self.cached_image_dir.path().to_path_buf()
        }

        fn create_cache_entry(&self, cache: &mut Cache, code: &String) -> bool {
            //Create the source image file (this file is overwritten each time
            //create_cache_entry is called!)
            let test_img_path = self.test_img_path();
            assert!(fs::write(&test_img_path, code).is_ok());

            cache.add_entry(code, &test_img_path)
        }

        fn test_img_path(&self) -> PathBuf {
            let mut p = self.path_buf();
            p.push("image.txt");
            p
        }
    }

    macro_rules! assert_is_cache_entry {
        ($ctx:expr,
            $cache: expr,
            $code: expr) => {{
            let key = format_key($code);
            assert!($cache.entries.contains_key(&key));

            let cached_img_filename = {
                let mut p = $ctx.path_buf();
                p.push(&key);
                p
            };
            assert!(cached_img_filename.is_file());
            let cached_image_data = match fs::read(&cached_img_filename) {
                Ok(u8_data) => String::from(String::from_utf8_lossy(&u8_data)),
                Err(e) => format!("Failed to read cache image file ({})", e),
            };

            assert_eq!($code, &cached_image_data);
        }};
    }

    #[test]
    fn empty_cache_returns_none() {
        let ctx = TestContext::new();
        let cache = ctx.create_cache();
        assert!(cache.get_entry(&String::from("some code")).is_none())
    }

    #[test]
    fn add_entry_fails_when_file_cannot_be_copied() {
        let ctx = TestContext::new();
        let mut cache = ctx.create_cache();

        //Add an entry with a non existing source file
        assert!(!cache.add_entry(&String::from("code block"), &PathBuf::from("nonexisting")));
    }

    #[test]
    fn add_entry() {
        let ctx = TestContext::new();
        let mut cache = ctx.create_cache();

        assert!(ctx.create_cache_entry(&mut cache, &String::from("code block")));
        assert_is_cache_entry!(&ctx, &cache, &String::from("code block"));
    }

    #[test]
    fn add_multiple_entries() {
        let ctx = TestContext::new();
        let mut cache = ctx.create_cache();

        assert!(ctx.create_cache_entry(&mut cache, &String::from("totemizer")));

        assert!(ctx.create_cache_entry(&mut cache, &String::from("wizard repellant")));

        assert_is_cache_entry!(&ctx, &cache, &String::from("totemizer"));

        assert_is_cache_entry!(&ctx, &cache, &String::from("wizard repellant"));
    }

    #[test]
    fn get_existing_entry() {
        let ctx = TestContext::new();
        let cache = {
            let mut cache = ctx.create_cache();

            assert!(ctx.create_cache_entry(&mut cache, &String::from("totemizer")));

            assert!(ctx.create_cache_entry(&mut cache, &String::from("wizard repellant")));

            cache
        };

        let froboz_entry = cache.get_entry(&String::from("totemizer"));
        assert!(froboz_entry.is_some());

        let electric_entry = cache.get_entry(&String::from("wizard repellant"));
        assert!(electric_entry.is_some());
    }

    #[test]
    fn get_uncached_entry() {
        let ctx = TestContext::new();
        let cache = {
            let mut cache = ctx.create_cache();

            assert!(ctx.create_cache_entry(&mut cache, &String::from("totemizer")));
            cache
        };

        assert!(cache.get_entry(&String::from("not cached")).is_none());
    }

    #[test]
    fn get_with_missing_cached_image() {
        let ctx = TestContext::new();
        let cache = {
            let mut cache = ctx.create_cache();

            assert!(ctx.create_cache_entry(&mut cache, &String::from("totemizer")));
            cache
        };

        let cached_image_path = cache.get_entry(&String::from("totemizer"));
        assert!(cached_image_path.is_some());

        //Delete the cached image, this should also yield None
        assert!(fs::remove_file(cached_image_path.unwrap()).is_ok());
        assert!(cache.get_entry(&String::from("totemizer")).is_none());
    }

    #[test]
    fn save_and_load() {
        let ctx = TestContext::new();
        {
            //Create the cache (and save on drop when exiting this scope)
            let mut cache = ctx.create_cache();
            assert!(ctx.create_cache_entry(&mut cache, &String::from("totemizer")));
        }

        let cache = Cache::new(&ctx.path_buf(), true);
        assert!(cache.is_ok());
        let cache = cache.unwrap();
        assert_is_cache_entry!(&ctx, &cache, &String::from("totemizer"));

        assert!(cache.get_entry(&String::from("totemizer")).is_some());
    }

    #[test]
    fn cache_dir_creation_failure_returns_error() {
        let ctx = TestContext::new();
        let invalid_dirname = {
            let mut p = ctx.path_buf();
            p.push("\0");
            p
        };

        let cache = Cache::new(&invalid_dirname, true);
        assert!(cache.is_err());
    }

    #[test]
    fn unused_entries_are_removed_on_save() {
        let ctx = TestContext::new();
        let file_to_be_removed;
        {
            let mut cache = ctx.create_cache();
            assert!(ctx.create_cache_entry(&mut cache, &String::from("keep me")));

            assert!(ctx.create_cache_entry(&mut cache, &String::from("remove me")));

            let entry_to_be_removed = cache.get_entry(&String::from("remove me"));

            assert!(entry_to_be_removed.is_some());
            file_to_be_removed = entry_to_be_removed.unwrap();
        }

        assert!(file_to_be_removed.is_file());

        // Reload the cache, only reference the keep_me entry, the other one should be discarded
        {
            let cache = Cache::new(&ctx.path_buf(), true);
            assert!(cache.is_ok());
            assert!(cache.unwrap().get_entry(&String::from("keep me")).is_some());
        }

        assert!(!file_to_be_removed.is_file());
    }

    #[test]
    fn unused_entries_are_kept_when_clean_on_save_is_false() {
        let ctx = TestContext::new();
        let cache_file;
        {
            let mut cache = ctx.create_cache();
            assert!(ctx.create_cache_entry(&mut cache, &String::from("keep me")));
            let entry = cache.get_entry(&String::from("keep me"));
            assert!(entry.is_some());
            cache_file = entry.unwrap();
        }

        assert!(cache_file.is_file());

        // Create the cache, with the clean_on_save flag set to false
        //Do not reference any file
        {
            let cache = Cache::new(&ctx.path_buf(), false);
            assert!(cache.is_ok());
        }

        assert!(cache_file.is_file());
    }
}
