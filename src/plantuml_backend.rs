use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use failure::Error;
use plantumlconfig::PlantUMLConfig;
use uuid::Uuid;

pub trait PlantUMLBackend {
    /// Render a PlantUML string and return the diagram file path (as a String)
    /// for use in an anchor tag
    fn render_from_string(&self, s: &String) -> Result<String, Error>;
}

/// Create an instance of the PlantUMLBackend
/// For now only a PlantUMLShell instance is created, later server support will be added
pub fn create(cfg: &PlantUMLConfig, book_root: &PathBuf) -> Box<PlantUMLBackend> {
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

    let mut img_root = book_root.clone();
    img_root.push("img");

    fs::create_dir_all(&img_root).expect("Failed to create image output dir.");

    Box::new(PlantUMLShell {
        plantuml_cmd: cmd,
        img_root: img_root,
    })
}

/// A trait class for wrapping the actual rendering command
/// Only here to make unit testing the renderer possbile, this is cheating a
/// bit, but the other option is not testing it at all, or partially through
/// integration tests
trait CommandExecutor {
    fn execute(&self, args: &Vec<String>) -> Result<(), Error>;
}

struct RealCommandExecutor;

impl CommandExecutor for RealCommandExecutor {
    fn execute(&self, args: &Vec<String>) -> Result<(), Error> {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C");
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c");
            cmd
        };

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

pub struct PlantUMLShell {
    plantuml_cmd: String,
    img_root: PathBuf,
}

/// Invokes PlantUML as a shell/cmd program.
impl PlantUMLShell {
    /// Get the command line for rendering the given source entry
    fn get_cmd_arguments(&self, file: &PathBuf, extension: &String) -> Result<Vec<String>, Error> {
        let mut args: Vec<String> = Vec::new();
        args.push(self.plantuml_cmd.clone());
        args.push(format!("-t{}", extension));
        args.push(String::from("-nometadata"));
        match file.to_str() {
            Some(s) => args.push(String::from(s)),
            None => {
                bail!("Failed to stringify temporary PlantUML file path.");
            }
        }

        Ok(args)
    }

    /// Create the source and image names with the appropriate extensions
    /// The file base names are a UUID to avoid collisions with exsisting
    /// files
    fn get_filenames(&self, extension: &String) -> (PathBuf, PathBuf) {
        let mut output_file = self.img_root.clone();
        output_file.push(Uuid::new_v4().to_string());
        output_file.set_extension(extension);

        let mut source_file = output_file.clone();
        source_file.set_extension("puml");

        (source_file, output_file)
    }

    fn render_from_string(
        &self,
        plantuml_code: &String,
        command_executor: &CommandExecutor,
    ) -> Result<String, Error> {
        let extension = get_extension(plantuml_code);
        let (source_file, output_file) = self.get_filenames(&extension);

        // Write diagram source file for rendering
        fs::write(source_file.as_path(), plantuml_code.as_str()).or_else(|e| {
            bail!("Failed to create temp file for inline diagram ({}).", e);
        })?;

        // Render the diagram, PlantUML will create a file with the same base
        // name, and the image extension
        let args = self.get_cmd_arguments(&source_file, &extension)?;
        command_executor.execute(&args).or_else(|e| {
            bail!("Failed to render inline diagram ({}).", e);
        })?;

        if !output_file.exists() {
            bail!(
                "PlantUML did not generate an image, did you forget the @startuml, @enduml block?"
            );
        }

        Ok(output_file.to_str().unwrap().replace("\\", "/"))
    }
}

impl PlantUMLBackend for PlantUMLShell {
    fn render_from_string(&self, plantuml_code: &String) -> Result<String, Error> {
        let executor = RealCommandExecutor {};
        self.render_from_string(plantuml_code, &executor)
    }
}

fn get_extension(plantuml_code: &String) -> String {
    if plantuml_code.contains("@startditaa") {
        String::from("png")
    } else {
        String::from("svg")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::err_msg;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    struct FakeCommandExecutor {
        error: bool,
        create_file: bool,
    }

    impl CommandExecutor for FakeCommandExecutor {
        fn execute(&self, args: &Vec<String>) -> Result<(), Error> {
            if self.error {
                Err(err_msg("Whoops"))
            } else {
                // Last argument is file name
                if self.create_file {
                    let mut filename = PathBuf::from(args.last().unwrap());
                    let source = &fs::read(filename.clone())?;

                    //Simply copy the contents of source to the output file
                    filename.set_extension("svg");
                    fs::write(filename.as_path(), source)?;
                }
                Ok(())
            }
        }
    }

    #[test]
    fn shell_command_line_arguments() {
        let shell = PlantUMLShell {
            plantuml_cmd: String::from("plantumlcmd"),
            img_root: PathBuf::from(""),
        };
        let file = PathBuf::from("froboz.puml");
        assert_eq!(
            vec![
                String::from("plantumlcmd"),
                String::from("-tsome_supported_extension"),
                String::from("-nometadata"),
                String::from("froboz.puml")
            ],
            shell
                .get_cmd_arguments(&file, &String::from("some_supported_extension"))
                .unwrap()
        );
    }

    #[test]
    fn command_failure() {
        let output_dir = tempdir().unwrap();
        let shell = PlantUMLShell {
            plantuml_cmd: String::from(""),
            img_root: output_dir.into_path(),
        };

        let executor = FakeCommandExecutor {
            error: true,
            create_file: false,
        };
        match shell.render_from_string(&String::from("@startuml\nA--|>B\n@enduml"), &executor) {
            Ok(_svg) => assert!(false, "Expected the command to fail"),
            Err(e) => assert!(
                e.to_string().contains("Failed to render inline diagram"),
                "Wrong error returned"
            ),
        };
    }

    #[test]
    fn no_image_file_created() {
        let output_dir = tempdir().unwrap();
        let shell = PlantUMLShell {
            plantuml_cmd: String::from(""),
            img_root: output_dir.into_path(),
        };

        let executor = FakeCommandExecutor {
            error: false,
            create_file: false,
        };
        match shell.render_from_string(&String::from("@startuml\nA--|>B\n@enduml"), &executor) {
            Ok(_svg) => assert!(false, "Expected the command to fail"),
            Err(e) => assert!(
                e.to_string().contains("PlantUML did not generate an image"),
                "Wrong error returned"
            ),
        };
    }

    #[test]
    fn returns_image_file_path_on_success() {
        let output_dir = tempdir().unwrap();
        let shell = PlantUMLShell {
            plantuml_cmd: String::from(""),
            img_root: output_dir.into_path(),
        };

        let executor = FakeCommandExecutor {
            error: false,
            create_file: true,
        };
        let source = String::from("@startuml\nA--|>B\n@enduml");
        match shell.render_from_string(&source, &executor) {
            Ok(filename) => {
                let raw_source = fs::read(filename).unwrap();
                let copied_source = String::from_utf8_lossy(&raw_source);
                assert_eq!(source, copied_source)
            }
            Err(e) => assert!(false, e.to_string()),
        };
    }
}
