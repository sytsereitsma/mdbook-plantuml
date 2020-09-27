use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::plantuml_backend::PlantUMLBackend;
use crate::util::get_extension;
use failure::Error;
use tempfile::{tempdir, TempDir};

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
    generation_dir: TempDir,
}

/// Invokes PlantUML as a shell/cmd program.
impl PlantUMLShell {
    pub fn new(plantuml_cmd: String) -> PlantUMLShell {
        PlantUMLShell {
            plantuml_cmd: plantuml_cmd,
            generation_dir: tempdir().unwrap(),
        }
    }

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

    /// Create the source and image names for the generation dir with the appropriate extensions
    fn get_filenames(&self, output_file: &PathBuf) -> (PathBuf, PathBuf) {
        let mut puml_image = self.generation_dir.path().to_path_buf();
        puml_image.push(output_file.file_name().unwrap());

        let mut puml_src = puml_image.clone();
        puml_src.set_extension("puml");

        (puml_src, puml_image)
    }

    ///Generate an image file from the given plantuml code.
    fn render_from_string(
        &self,
        plantuml_code: &String,
        output_file: &PathBuf,
        command_executor: &dyn CommandExecutor,
    ) -> Result<(), Error> {
        let (puml_src, puml_image) = self.get_filenames(output_file);
        // Write diagram source file for rendering
        fs::write(puml_src.as_path(), plantuml_code.as_str()).or_else(|e| {
            bail!("Failed to create temp file for inline diagram ({}).", e);
        })?;

        // Render the diagram, PlantUML will create a file with the same base
        // name, and the image extension
        let args = self.get_cmd_arguments(&puml_src, &get_extension(output_file))?;
        command_executor.execute(&args).or_else(|e| {
            bail!("Failed to render inline diagram ({}).", e);
        })?;

        if !puml_image.exists() {
            bail!(
                format!("PlantUML did not generate an image, did you forget the @startuml, @enduml block ({})?", args.join(" "))
            );
        }

        if let Err(e) = fs::copy(&puml_image, &output_file) {
            bail!(
                "Error copying the generated PlantUML image {} from to {} ({}).",
                puml_image.to_string_lossy(),
                output_file.to_string_lossy(),
                e
            );
        }

        Ok(())
    }
}

impl PlantUMLBackend for PlantUMLShell {
    fn render_from_string(
        &self,
        plantuml_code: &String,
        output_file: &PathBuf,
    ) -> Result<(), Error> {
        let executor = RealCommandExecutor {};
        PlantUMLShell::render_from_string(self, plantuml_code, output_file, &executor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::join_path;
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
            generation_dir: tempdir().unwrap(),
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

    fn run_render_from_string(
        generate_error: bool,
        create_file: bool,
        code: Option<&String>,
    ) -> Result<String, Error> {
        let output_dir = tempdir().unwrap();
        //Cannot be the same path as output_dir, because otherwise we'd try to
        //copy a file onto itself
        let img_dir = tempdir().unwrap();
        let output_file = join_path(img_dir.path(), "foobar.svg");

        let shell = PlantUMLShell {
            plantuml_cmd: String::from(""),
            generation_dir: output_dir,
        };

        let executor = FakeCommandExecutor {
            error: generate_error,
            create_file: create_file,
        };

        shell.render_from_string(
            code.unwrap_or(&String::from("@startuml\nA--|>B\n@enduml")),
            &output_file,
            &executor,
        )?;

        if create_file {
            let raw_source = fs::read(&output_file)?;
            return Ok(String::from_utf8_lossy(&raw_source).into_owned());
        }

        Ok(String::from(""))
    }

    #[test]
    fn command_failure() {
        match run_render_from_string(true, false, None) {
            Ok(_file_data) => assert!(false, "Expected the command to fail"),
            Err(e) => assert!(
                e.to_string().contains("Failed to render inline diagram"),
                "Wrong error returned"
            ),
        };
    }

    #[test]
    fn no_image_file_created() {
        match run_render_from_string(false, false, None) {
            Ok(_file_data) => assert!(false, "Expected the command to fail"),
            Err(e) => assert!(
                e.to_string().contains("PlantUML did not generate an image"),
                "Wrong error returned (got {})",
                e
            ),
        };
    }

    #[test]
    fn returns_image_file_path_on_success() {
        let expected_source = String::from("My plantuml code");
        match run_render_from_string(false, true, Some(&expected_source)) {
            Ok(file_data) => {
                assert_eq!(expected_source, file_data);
            }
            Err(e) => assert!(false, "{}", e.to_string()),
        };
    }
}
