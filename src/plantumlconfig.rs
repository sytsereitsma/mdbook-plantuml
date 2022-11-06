use serde::{Deserialize, Serialize};

/// Workaround for serde's lack of support for default = "true"
fn bool_true() -> bool {
    true
}

/// The configuration options available with this backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PlantUMLConfig {
    /// By default it is assumed plantuml.jar is on the path
    /// Use plantuml_cmd if it is not on the path, or if you
    /// have some additional parameters.
    pub plantuml_cmd: Option<String>,
    /// When the PlantUML shell is called this option enables piped mode, meaning no temporary directories
    /// and files are needed for image generation (defaults to false).
    #[serde(default = "bool_true")]
    pub piped: bool,
    /// PlantUML images become clickable for zoom by setting this flag to `true`.
    /// This is convenient for large diagrams which are hard to see in the book.
    /// The default value is `false`.
    pub clickable_img: bool,
    /// Instead of creating inlined links to image files use data URIs (defaults to true)
    #[serde(default = "bool_true")]
    pub use_data_uris: bool,
    /// Verbose logging (debug level)
    pub verbose: bool,
}

impl Default for PlantUMLConfig {
    fn default() -> Self {
        PlantUMLConfig {
            plantuml_cmd: None,
            piped: true,
            clickable_img: false,
            use_data_uris: true,
            verbose: false,            
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
        assert_eq!(cfg.piped, true);
        assert_eq!(cfg.clickable_img, false);
        assert_eq!(cfg.use_data_uris, true);
        assert_eq!(cfg.verbose, false);
    }
}
