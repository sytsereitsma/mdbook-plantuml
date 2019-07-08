use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use failure::Error;
use tempfile::tempdir;

use plantumlconfig::PlantUMLConfig;

pub trait PlantUMLBackend {
    /// Render a PlantUML string and return the SVG diagram as a String
    fn render_svg_from_string(&self, s: &String) -> Result<String, Error>;
}

/// Create an instance of the PlantUMLBackend
/// For now only a PlantUMLShell instance is created, later server support will be added
pub fn create(cfg: &PlantUMLConfig) -> Box<PlantUMLBackend> {
    let cmd = match &cfg.plantuml_cmd {
        Some(s) => s.clone(),
        None => {
            if cfg!(target_os = "windows") {
                String::from("java -jar plantuml.jar")
            } else {
                String::from("/usr/bin/plantuml")
            }
        }
    };

    Box::new(PlantUMLShell { plantuml_cmd: cmd })
}

pub struct PlantUMLShell {
    plantuml_cmd: String,
}

/// Invokes PlantUML as a shell/cmd program.
impl PlantUMLShell {
    /// Get the command line for rendering the given source entry
    fn get_cmd_arguments(&self, file: PathBuf) -> Result<Vec<String>, Error> {
        let mut args: Vec<String> = Vec::new();
        args.push(self.plantuml_cmd.clone());
        args.push(String::from("-tsvg"));
        args.push(String::from("-nometadata"));
        match file.to_str() {
            Some(s) => args.push(String::from(s)),
            None => {
                bail!("Failed to stringify temporary PlantUML file path.");
            }
        }

        Ok(args)
    }

    /// Render a single file. PlantUML will create the rendered diagram next to the specified file.
    // The rendered diagram file has the same basename as the source file.
    fn render_file(&self, file: PathBuf) -> Result<(), Error> {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C");
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c");
            cmd
        };

        let args = self.get_cmd_arguments(file)?;
        debug!("Executing '{}'", args.join(" "));
        debug!(
            "Working dir '{}'",
            env::current_dir().unwrap_or(PathBuf::from(".")).display()
        );

        let output = cmd
            // We're invoking through the shell, so call it like this:
            // ```sh -c "<args>"```
            // If not done this way sh -c will ignore all data after the first
            // argument (e.g. ```sh -c plantuml source.puml``` will become
            // ```sh -c plantuml```.
            .arg(args.join(" "))
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
}

impl PlantUMLBackend for PlantUMLShell {
    fn render_svg_from_string(&self, plantuml_code: &String) -> Result<String, Error> {
        let dir = tempdir().or_else(|e| {
            bail!("Failed to create temp dir for inline diagram ({}).", e);
        })?;

        // Write diagram file for rendering
        let file_path = dir.path().join("source.puml");
        fs::write(file_path, plantuml_code.as_str()).or_else(|e| {
            bail!("Failed to create temp file for inline diagram ({}).", e);
        })?;

        // Render the diagram
        let file_path = dir.path().join("source.puml");
        self.render_file(file_path).or_else(|e| {
            bail!("Failed to render inline diagram ({}).", e);
        })?;

        // Read the SVG data
        let file_path = dir.path().join("source.svg");
        let file_data = fs::read(file_path).or_else(|e| {
            bail!("Failed to read the generated inline diagram file ({})\nPossibly you forgot to wrap the diagram text in a @startuml/@enduml block (see PlantUML manual).", e);
        })?;

        let svg = String::from_utf8(file_data).or_else(|e| {
            bail!("Failed to decode generated inline diagram file ({}).", e);
        })?;

        Ok(format!("<div class='plantuml'>{}</div>", svg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn shell_command_line_arguments() {
        let shell = PlantUMLShell {
            plantuml_cmd: String::from("plantumlcmd"),
        };
        let file = PathBuf::from("froboz.puml");
        assert_eq!(
            vec![
                String::from("plantumlcmd"),
                String::from("-tsvg"),
                String::from("-nometadata"),
                String::from("froboz.puml")
            ],
            shell.get_cmd_arguments(file).unwrap()
        );
    }

    #[test]
    fn command_failure() {
        let shell = PlantUMLShell {
            plantuml_cmd: String::from("invalid-plantuml-executable"),
        };

        match shell.render_svg_from_string(&String::from("@startuml\nA--|>B\n@enduml")) {
            Ok(_svg) => assert!(false, "Expected the command to fail"),
            Err(_) => (),
        };
    }
}
