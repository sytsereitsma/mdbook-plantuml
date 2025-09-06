use serde::{Deserialize, Serialize};

/// Workaround for serde's lack of support for default = "true"
fn bool_true() -> bool {
    true
}
fn bool_false() -> bool {
    false
}

/// The configuration options available with this backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    /// By default it is assumed plantuml.jar is on the path
    /// Use plantuml_cmd if it is not on the path, or if you
    /// have some additional parameters.
    pub plantuml_cmd: Option<String>,
    /// When the PlantUML shell is called this option enables piped mode, meaning no temporary directories
    /// and files are needed for image generation (defaults to true).
    /// This also allows using the `!include` and `!includesub` directives in plantuml. The working directory
    /// for this command is the markdown file's directory (meaning using `!include foo.puml` from `bar.md` expects
    /// `foo.puml` to be in the same directory as `bar.md`).
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
    /// Suppress error messages and output the image generation errors in the generated document
    /// instead. This is useful when you want to avoid aborting the book build due to PlantUML errors.
    /// If set to true, errors are logged and the book build will fail upon the first rendering error
    /// The default is false.
    /// The MDBOOK_PLANTUML_FAIL_ON_ERROR=1 (or 0) environment variable to overrides this setting.
    #[serde(default = "bool_false")]
    pub fail_on_error: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plantuml_cmd: None,
            piped: true,
            clickable_img: false,
            use_data_uris: true,
            verbose: false,
            fail_on_error: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn default() {
        let cfg = Config::default();
        assert_eq!(cfg.plantuml_cmd, None);
        assert_eq!(cfg.piped, true);
        assert_eq!(cfg.clickable_img, false);
        assert_eq!(cfg.use_data_uris, true);
        assert_eq!(cfg.verbose, false);
        assert_eq!(cfg.fail_on_error, false);
    }
}
