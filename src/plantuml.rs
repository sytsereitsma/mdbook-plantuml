use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use failure::Error;
use tempfile::tempdir;

pub trait PlantUMLRenderer {
    /// Render files given a list of files, direcotories, globs, etc.
    /// Simply the same file arguments you'd use invoke plantuml from the command line.
    fn render_files(&self, files: &Vec<String>, output_dir: Option<PathBuf>) -> Result<(), Error>;

    /// Render a string and return the SVG diagram as a String
    fn render_svg_from_string(&self, s: &String) -> Result<String, Error>;
}

pub struct PlantUML {
    plantuml_cmd: String,
}

impl PlantUML {
    // Another static method, taking two arguments:
    pub fn new(plantuml_cmd: &Option<String>) -> PlantUML {
        let plantuml_cmd = match plantuml_cmd {
            Some(s) => s.clone(),
            None => String::from("java -jar plantuml.jar"),
        };

        PlantUML { plantuml_cmd }
    }

    /// Get the command line for rendering the given source entry
    fn get_cmd_arguments(
        &self,
        files: &Vec<String>,
        output_dir: Option<PathBuf>,
    ) -> Result<Vec<String>, Error> {
        let mut args: Vec<String> = Vec::new();
        args.push(self.plantuml_cmd.clone());
        args.push(String::from("-tsvg"));
        if output_dir.is_some() {
            let path_str = output_dir.unwrap();
            let path_str = path_str.to_str().expect("Failed to get output dir");
            args.push(String::from("-o"));
            args.push(String::from(path_str));
        }

        for f in files {
            args.push(f.clone());
        }

        Ok(args)
    }
}

impl PlantUMLRenderer for PlantUML {
    fn render_files(&self, files: &Vec<String>, output_dir: Option<PathBuf>) -> Result<(), Error> {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C");
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c");
            cmd
        };

        let args = self.get_cmd_arguments(files, output_dir)?;
        debug!("Executing '{}'", args.join(" "));
        debug!(
            "Working dir '{}'",
            env::current_dir().unwrap_or(PathBuf::from(".")).display()
        );

        let output = cmd
            .args(args)
            .output()
            .expect("Failed to start PlantUML application");

        if output.status.success() {
            info!("Successfully generated PlantUML diagrams.");
            debug!(
                "stdout: {}",
                String::from_utf8(output.stdout).unwrap_or(String::from(""))
            );
            debug!(
                "stderr: {}",
                String::from_utf8(output.stderr).unwrap_or(String::from(""))
            );
        } else {
            let msg = format!(
                "Failed to generate PlantUML diagrams, PlantUML exited with code {} ({}).",
                output.status.code().unwrap_or(-9999),
                String::from_utf8(output.stderr).unwrap_or(String::from(""))
            );
            bail!(msg);
        }

        Ok(())
    }

    fn render_svg_from_string(&self, s: &String) -> Result<String, Error> {
        let dir = tempdir().or_else(|e| {
            bail!("Failed to create temp dir for inline diagram ({}).", e);
        })?;

        // Write diagram file for rendering
        let file_path = dir.path().join("source.puml");
        fs::write(file_path, s.as_str()).or_else(|e| {
            bail!("Failed to create temp file for inline diagram ({}).", e);
        })?;

        // Render the diagram
        let file_path = dir.path().join("source.puml");
        let str_file_path = file_path.to_str().unwrap();
        self.render_files(&vec![String::from(str_file_path)], None)
            .or_else(|e| {
                bail!("Failed to render inline diagram ({}).", e);
            })?;

        // Read the SVG data
        let file_path = dir.path().join("source.svg");
        let svg = String::from_utf8(fs::read(file_path)?).or_else(|e| {
            bail!("Failed to read generated inline diagram file ({}).", e);
        })?;

        Ok(svg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_initializes_command() {
        let plant = PlantUML::new(&None::<String>);
        assert_eq!(plant.plantuml_cmd, "java -jar plantuml.jar");

        let plant = PlantUML::new(&Some(String::from("froboz electric")));
        assert_eq!(plant.plantuml_cmd, "froboz electric");
    }
}
