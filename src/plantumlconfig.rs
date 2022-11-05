use serde::{Deserialize, Serialize};

/// The configuration options available with this backend.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct PlantUMLConfig {
    /// By default it is assumed plantuml.jar is on the path
    /// Use plantuml_cmd if it is not on the path, or if you
    /// have some additional parameters.
    pub plantuml_cmd: Option<String>,
    /// PlantUML images become clickable for zoom by setting this flag to `true`.
    /// This is convenient for large diagrams which are hard to see in the book.
    /// The default value is `false`.
    pub clickable_img: bool,
    /// Instead of creating inlined links to image files use data URIs
    pub use_data_uris: bool,
    /// Verbose logging (debug level)
    pub verbose: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn default() {
        let cfg = PlantUMLConfig::default();
        assert_eq!(cfg.plantuml_cmd, None);
        assert_eq!(cfg.clickable_img, false);
        assert_eq!(cfg.use_data_uris, false);
        assert_eq!(cfg.verbose, false);
    }
}
