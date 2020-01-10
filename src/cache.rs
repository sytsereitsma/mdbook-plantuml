use failure::Error;
use serde_json::json;
use sha1;
use std::collections::HashMap;
use std::path::PathBuf;
use std::string::String;
use tempfile::{tempdir, TempDir, NamedTempFile};


pub struct Cache {
    entries: HashMap<String, String>, //Key, signature pair
    cache_path: PathBuf,
}

fn format_key(chapter_path: &PathBuf, idx: u32) -> Option<String> {
    let chapter_path_str = chapter_path.to_str();
    if chapter_path_str.is_none() {
        return None;
    }

    Some(format!("{}_{}", chapter_path_str.unwrap(), idx))
}

impl Cache {
    /// Creates a Cache instance using the provided path. If the path does not
    /// exist it is created.
    /// When the path exists the cache entries are loaded from the cache file.
    /// # Arguments
    /// * `cache_path` - The path where the cache should be stored/loaded from
    pub fn new(cache_path: &PathBuf) -> Cache {
        Cache {
            entries: HashMap::new(),
            cache_path: cache_path.clone(),
        }
    }

    pub fn save(&self) {}

    /// Get the path of the cached image, returns None if the file does not
    /// exist
    /// # Arguments
    /// * `signature` - The signature of the code block being looked up
    fn get_image_path(&self, signature: &String) -> Option<PathBuf> {
        let img_path = {
            let mut p = self.cache_path.clone();
            p.push(signature);
            p
        };

        if img_path.is_file() {
            return Some(img_path)
        }

        None
    }

    /// Returns the path to the cached image for the given code block, or None when the image is not cached, or the code
    /// has changed.
    ///
    /// # Arguments
    /// * `chapter_path` - The chapter path in the book (mdBook BookItem::Chapter::path value)
    /// * `idx` - A unique (chapter scope) index for the code block to get the cache for (there may be more than one code block in a single chapter)
    /// * `code_block_src` - The present source of the code block, if it does not match with the cached code block None is returned
    pub fn get_cached_image(
        &self,
        chapter_path: &PathBuf,
        idx: u32,
        code_block_src: &String,
    ) -> Option<PathBuf> {
        let key = format_key(chapter_path, idx);
        if key.is_none() {
            //TODO log error
            return None;
        }

        let expected_signature = sha1::Sha1::from(&code_block_src).hexdigest();
        if let Some(signature) = self.entries.get(&key.unwrap()) {
            if &expected_signature == signature {
                return self.get_image_path(signature);
            }
        }

        None
    }

    /// Adds a new entry to the cache, or updates an existing one.
    /// # Arguments
    /// * `chapter_path` - The chapter path in the book (mdBook BookItem::Chapter::path value)
    /// * `idx` - A unique (chapter scope) index for the code block to get the cache for (there may be more than one code block in a single chapter)
    /// * `code_block_src` - The present source of the code block, if it does not match with the cached code block None is returned
    /// * `image_path` - The path to the image to cache (a copy of the file will be saved in the cache directory)
    fn add_entry(
        &mut self,
        chapter_path: &PathBuf,
        idx: u32,
        code_block_src: &String,
        image_path: &PathBuf,
    ) -> Result<(), Error> {
        let key = format_key(chapter_path, idx);
        if key.is_none() {
            bail!("Failed to format key");
        }

        let signature = sha1::Sha1::from(code_block_src).hexdigest();
        self.entries.insert(
            key.unwrap(),
            signature
        );

        Ok(())
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
        cache_entries: HashMap<String, CacheEntry>,
    }

    impl TestContext {
        fn new() -> TestContext {
            TestContext {
                cached_image_dir: tempdir().unwrap(),
                cache_entries: HashMap::new(),
            }
        }

        /// Add a cache entry
        fn add_cache_entry(
            &mut self,
            key: String,
            code: &String,
            image_filename: &String,
            create_file: bool,
        ) {
            let image_path = self
                .cached_image_dir
                .path()
                .to_path_buf()
                .join(image_filename);
            if create_file {
                fs::write(image_path.as_path(), code.as_str())
                    .or_else(|e| {
                        bail!("Failed to create image file for testing ({:?}).", e);
                    })
                    .unwrap();
            }

            self.cache_entries.insert(
                key,
                CacheEntry {
                    signature: sha1::Sha1::from(&code).hexdigest(),
                    image_path: image_path,
                },
            );
        }
    }

    #[test]
    fn empty_cache() {
        let cache = Cache {
            entries: HashMap::new(),
            cache_path: PathBuf::from(""),
        };
        assert!(cache
            .get_cached_image(&PathBuf::from("chapter"), 0, &String::new())
            .is_none())
    }

    #[test]
    fn cache_entry_with_valid_file() {
        let mut ctx = TestContext::new();
        let code = String::from("No statement can catch a ChuckNorrisException");
        ctx.add_cache_entry(
            String::from("chapter_1"),
            &code,
            &String::from("cachehit.txt"),
            true,
        );

        let cache = Cache {
            entries: ctx.cache_entries,
            cache_path: PathBuf::from(""),
        };

        let cached_img_path = cache.get_cached_image(&PathBuf::from("chapter"), 1, &code);
        assert!(cached_img_path.is_some());
        assert_eq!(
            cached_img_path.unwrap().file_name().unwrap(),
            "cachehit.txt"
        );

        //Code changes should result in a miss
        let changed_code = String::from("When Chuck Norris stares at the sun, the sun blinks");
        let cached_img_path = cache.get_cached_image(&PathBuf::from("chapter"), 1, &changed_code);
        assert!(cached_img_path.is_none());
    }

    #[test]
    fn missing_cache_file_is_cache_miss() {
        let mut ctx = TestContext::new();
        let code = String::from("No statement can catch a ChuckNorrisException");
        ctx.add_cache_entry(
            String::from("chapter_1"),
            &code,
            &String::from("not_existing.txt"),
            false,
        );

        let cache = Cache {
            entries: ctx.cache_entries,
            cache_path: PathBuf::from(""),
        };

        let cached_img_path = cache.get_cached_image(&PathBuf::from("chapter"), 1, &code);
        assert!(cached_img_path.is_none());
    }

    #[test]
    fn multiple_cache_entries() {
        let mut ctx = TestContext::new();
        let code1 = String::from("No statement can catch a ChuckNorrisException");
        ctx.add_cache_entry(
            String::from("chapter_1"),
            &code1,
            &String::from("hit1.txt"),
            true,
        );

        let code2 = String::from("When Chuck Norris stares at the sun, the sun blinks");
        ctx.add_cache_entry(
            String::from("chapter_2"),
            &code2,
            &String::from("hit2.txt"),
            true,
        );

        let cache = Cache {
            entries: ctx.cache_entries,
            cache_path: PathBuf::from(""),
        };

        let cached_img_path = cache.get_cached_image(&PathBuf::from("chapter"), 1, &code1);
        assert!(cached_img_path.is_some());
        assert_eq!(cached_img_path.unwrap().file_name().unwrap(), "hit1.txt");

        let cached_img_path = cache.get_cached_image(&PathBuf::from("chapter"), 2, &code2);
        assert!(cached_img_path.is_some());
        assert_eq!(cached_img_path.unwrap().file_name().unwrap(), "hit2.txt");

        // Code change (swap the code between the chapter[1] and chapter[2] blocks)
        let cached_img_path = cache.get_cached_image(&PathBuf::from("chapter"), 1, &code2);
        assert!(cached_img_path.is_none());
        let cached_img_path = cache.get_cached_image(&PathBuf::from("chapter"), 2, &code1);
        assert!(cached_img_path.is_none());
    }

    #[test]
    fn add_entry() {
        let tmp_dir = TempDir::new();
        assert!(tmp_dir.is_ok());

        let tmp_dir = tmp_dir.unwrap();
        let tmp_path = tmp_dir.path().to_path_buf();
        let test_img_path = {
            let mut p = tmp_path.clone();
            p.push("image.txt");
            p
        };
        assert!(fs::write(&test_img_path, "Lorem ipsum").is_ok());

        let mut cache = Cache::new(&tmp_path);

        match cache.add_entry(
            &PathBuf::from("Some chapter"),
            1,
            &String::from("Code block"),
            &test_img_path,
        ) {
            Ok(_) => (),
            Err(e) => assert!(false, e.to_string()),
        };
    }

    // #[test]
    // fn saves_cache_table_on_add() {
    //     let tmp_dir = TempDir::new()?;
    //     let tmp_path = tmp_dir.unwrap.path().to_path_buf();
    //     let cache = Cache::new(tmp_path);
    // }
}
