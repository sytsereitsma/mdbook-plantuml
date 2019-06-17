/// The configuration options available with this backend.
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PlantUMLSource {
    /// List of files, or glob patterns for the files to process (e.g. "../foo/*.uml"), path is
    /// relative to book root dir. The pattern is directly forwarded
    /// to the PlantUML executable \(see the [command line reference](http://plantuml.com/command-line)\).
    pub src: Vec<String>,
    /// Path relative to the book output dir, this path given to the PlantUML
    /// executable as the -o argument.
    pub output_dir: PathBuf,
}

impl Default for PlantUMLSource {
    fn default() -> PlantUMLSource {
        PlantUMLSource {
            src: Vec::new(),
            output_dir: PathBuf::from("src"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PlantUMLConfig {
    /// The additional resources to copy
    pub sources: Vec<PlantUMLSource>,
    /// By default it is assumed plantuml.jar is on the path
    /// Use plantuml_cmd if it is not on the path, or if you
    /// have some additional parameters.
    pub plantuml_cmd: Option<String>,
    pub extra_flags: Vec<String>,
}

impl Default for PlantUMLConfig {
    fn default() -> PlantUMLConfig {
        PlantUMLConfig {
            sources: Vec::new(),
            plantuml_cmd: None::<String>,
            extra_flags: Vec::new(),
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
        assert_eq!(cfg.sources.len(), 0);
        assert_eq!(cfg.plantuml_cmd, None);
    }
}
