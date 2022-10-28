use std::path::{Path, PathBuf};
use crate::plantuml_backend::PlantUMLBackend;
use anyhow::{bail, format_err, Context, Result};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

trait PlantUMLRunner {
    fn run(&self, plantuml_cmd: &str, plantuml_src: &str, format: &str) -> Result<Vec<u8>>;
}

struct PipedPlantUMLRunner;
impl PlantUMLRunner for PipedPlantUMLRunner {
    fn run(&self, plantuml_cmd: &str, plantuml_src: &str, format: &str) -> Result<Vec<u8>> {
        let child = Command::new(plantuml_cmd)
            // There cannot be a space between -t and format! Otherwise PlantUML generates a PNG image
            .arg(format!("-t{}", format))
            .arg("-nometadata")
            .arg("-pipe")
            .arg("-pipeNoStderr")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        if let Err(e) = child {
            return Err(e)
                .with_context(|| format!("Failed to start PlantUML command '{}' ", plantuml_cmd));
        }

        // Pipe the plantuml source
        let mut child = child.unwrap();
        let stdin_result = child
            .stdin
            .take()
            .unwrap() // We can simply unwrap, because we know stdin is piped
            .write_all(plantuml_src.as_bytes())
            .and_then(|stdin| {
                drop(stdin);
                Ok(())
            });
        if let Err(e) = stdin_result {
            return Err(e).with_context(|| "Failed to pipe PlantUML code");
        }

        // And wait for the result
        match child.wait_with_output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(output.stdout)
                } else {
                    Err(format_err!(
                        "Failed to render image in piped mode (return value {})",
                        output.status
                    ))
                }
            }
            Err(e) => Err(e).with_context(|| "Failed to get generated piped PlantUML image"),
        }
    }
}

struct FilePlantUMLRunner;

impl  FilePlantUMLRunner {
    fn find_generated_file(generation_dir: &Path, src_file_name: &str) -> Result<PathBuf> {
        // PlantUML creates an output file based on the format, it is not always the same as `format` though (e.g. braille outputs a file
        // with extension `.braille.png`)
        // Just see which other file is in the directory next to our source file. That's the generated one...
        let entries = fs::read_dir(generation_dir)?;

        // Now find the generated file
        for entry in entries {
            if let Ok(path) = entry {
                if path.file_name() != src_file_name {
                    return Ok(path);
                }
            }
        }

        bail!("Failed to find generated PlantUML image.");
    }    
}

/// Traditional file based renderer. Simply writes a file with the PlantUML source to disk and reads back the output file
impl PlantUMLRunner for FilePlantUMLRunner {

    fn run(&self, plantuml_cmd: &str, plantuml_src: &str, format: &str) -> Result<Vec<u8>> {
        // Generate the file in a tmpdir
        let generation_dir = tempdir().with_context("Failed to create PlantUML tempdir")?;

        // Write the PlantUML source file
        const SRC_FILE_NAME: &str = "src.puml";
        let src_file = generation_dir.path().join(SRC_FILE_NAME);
        fs::write(&src_file, plantuml_src).with_context("Failed to write PlantUML source file")?;

        // Call PlantUML
        Command::new(plantuml_cmd)
            // There cannot be a space between -t and format! Otherwise PlantUML generates a PNG image
            .arg(format!("-t{}", format))
            .arg("-nometadata")
            .arg(&src_file.to_str().unwrap())
            .output()
            .with_context(|| "Failed to render image")?;

        let generated_file = FilePlantUMLRunner::find_generated_file(&generation_dir.path(), SRC_FILE_NAME)?;         
        return fs::read(generated_file).with_context(|| "Failed to read rendered image");
    }
}

pub struct PlantUMLShell {
    plantuml_cmd: String,
}

/// Invokes PlantUML as a shell/cmd program.
impl PlantUMLShell {
    pub fn new(plantuml_cmd: String) -> Self {
        Self { plantuml_cmd }
    }
}

impl PlantUMLBackend for PlantUMLShell {
    fn render_from_string(
        &self,
        plantuml_code: &str,
        image_format: &str,
    ) -> Result<Vec<u8>> {
        //let runner = PipedPlantUMLRunner {};
        let runner = FilePlantUMLRunner {};
        return runner.run(&self.plantuml_cmd, plantuml_code, image_format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::util::join_path;
    use anyhow::bail;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    struct FakeCommandExecutor {
        error: bool,
        create_file: bool,
    }

    impl CommandExecutor for FakeCommandExecutor {
        fn execute(&self, args: &[String]) -> Result<()> {
            if self.error {
                bail!("Whoops")
            } else {
                // Last argument is file name
                if self.create_file {
                    let mut filename = PathBuf::from(args.last().unwrap());
                    let source = fs::read(&filename)?;

                    // Simply copy the contents of source to the output file
                    filename.set_extension("svg");
                    fs::write(filename.as_path(), &source)?;
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
                .get_cmd_arguments(&file, "some_supported_extension")
                .unwrap()
        );
    }

    fn run_render_from_string(
        generate_error: bool,
        create_file: bool,
        code: Option<&String>,
    ) -> Result<String> {
        let output_dir = tempdir().unwrap();
        // Cannot be the same path as output_dir, because otherwise we'd try to
        // copy a file onto itself
        let img_dir = tempdir().unwrap();
        let output_file = join_path(img_dir.path(), "foobar.svg");

        let shell = PlantUMLShell {
            plantuml_cmd: String::default(),
            generation_dir: output_dir,
        };

        let executor = FakeCommandExecutor {
            error: generate_error,
            create_file,
        };

        shell.render_from_string(
            code.map_or("@startuml\nA--|>B\n@enduml", AsRef::as_ref),
            "svg",
            &output_file,
            &executor,
        )?;

        if create_file {
            let raw_source = fs::read(&output_file)?;
            return Ok(String::from_utf8_lossy(&raw_source).into_owned());
        }

        Ok(String::default())
    }

    #[test]
    fn command_failure() {
        match run_render_from_string(true, false, None) {
            Ok(_file_data) => panic!("Expected the command to fail"),
            Err(e) => assert!(
                e.to_string().contains("Failed to render inline diagram"),
                "Wrong error returned"
            ),
        };
    }

    #[test]
    fn no_image_file_created() {
        match run_render_from_string(false, false, None) {
            Ok(_file_data) => panic!("Expected the command to fail"),
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
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn get_filenames_returns_input_and_output_filename() {
        macro_rules! get_names {
            ($generation_dir:expr, $puml_name:expr, $img_name:expr) => {{
                let mut puml_image = $generation_dir.path().to_path_buf();
                let mut output_img = puml_image.clone();

                puml_image.push(PathBuf::from($puml_name));
                output_img.push(PathBuf::from($img_name));
                (puml_image, output_img)
            }};
        }

        let shell = PlantUMLShell {
            plantuml_cmd: String::default(),
            generation_dir: tempdir().unwrap(),
        };

        assert_eq!(
            get_names!(shell.generation_dir, "foo.puml", "foo.png"),
            shell.get_filenames(Path::new("foo.png"))
        );

        assert_eq!(
            get_names!(shell.generation_dir, "foo.puml", "foo.braille.png"),
            shell.get_filenames(Path::new("foo.braille.png"))
        );
    }
}
