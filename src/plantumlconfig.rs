/// The configuration options available with this backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PlantUMLConfig {
    /// By default it is assumed plantuml.jar is on the path
    /// Use plantuml_cmd if it is not on the path, or if you
    /// have some additional parameters.
    pub plantuml_cmd: Option<String>,
    pub use_cache: Option<bool>,
}

impl Default for PlantUMLConfig {
    fn default() -> PlantUMLConfig {
        PlantUMLConfig {
            plantuml_cmd: None::<String>,
            use_cache: Some(false),
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
        assert!(cfg.use_cache.is_some());
        assert!(!cfg.use_cache.unwrap());
    }
}
