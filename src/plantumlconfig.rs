/// The configuration options available with this backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PlantUMLConfig {
    /// By default it is assumed plantuml.jar is on the path
    /// Use plantuml_cmd if it is not on the path, or if you
    /// have some additional parameters.
    pub plantuml_cmd: Option<String>,
    /// When true the cache is enabled (for now it defaults to false)
    pub enable_cache: Option<bool>,
    /// The directory where to store the cache (defaults to book dir)
    /// This option can be used to have a central location for use with multiple
    /// books (e.g. same book on different version branches).
    /// Protection for parallel use of the cache with a shared cache dir is
    /// something  on the todo list. So when sharing the cache dir between
    /// projects make sure you do not generate the books in parallel, this will
    ///  likely corrupt the cache.
    pub cache_dir: Option<String>,
    /// When true (default), unused entries are removed from the cache when the
    /// application closes.
    /// You'd typically set this option to false when you have cache directory
    /// that is used for multiple books (i.e. you override the cache dir)
    pub clean_cache: Option<bool>,
}

impl Default for PlantUMLConfig {
    fn default() -> PlantUMLConfig {
        PlantUMLConfig {
            plantuml_cmd: None,
            cache_dir: None,
            enable_cache: Some(false),
            clean_cache: Some(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn default() {
        let cfg = PlantUMLConfig::default();
        assert_eq!(cfg.plantuml_cmd, None);

        assert!(cfg.enable_cache.is_some());
        assert!(!cfg.enable_cache.unwrap());

        assert!(cfg.clean_cache.is_some());
        assert!(cfg.clean_cache.unwrap());
    }
}
