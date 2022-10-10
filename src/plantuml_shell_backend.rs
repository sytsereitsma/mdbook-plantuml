use crate::plantuml_backend::PlantUMLBackend;
use anyhow::{bail, format_err, Result};
use log;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::tempdir;

trait PlantUMLRunner {
    fn run(&self, plantuml_cmd: &str, plantuml_src: &str, format: &str) -> Result<Vec<u8>>;
}

struct PipedPlantUMLRunner;
impl PlantUMLRunner for PipedPlantUMLRunner {
    fn run(&self, plantuml_cmd: &str, plantuml_src: &str, format: &str) -> Result<Vec<u8>> {
        let mut child = Command::new(plantuml_cmd)
            // There cannot be a space between -t and format! Otherwise PlantUML generates a PNG image
            .arg(format!("-t{}", format))
            .arg("-nometadata")
            .arg("-pipe")
            .arg("-pipeNoStderr")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn().expect("Failed to start PlantUML");

        // Pipe the plantuml source
        let mut stdin = child
            .stdin
            .take()
            .expect("Failed to open stdin for piped rendering");
        stdin
            .write_all(plantuml_src.as_bytes())
            .expect("Failed to write to stdin for piped rendering");
        drop(stdin); // Drop stdin, because otherwise child will hang in wait_with_output, because it cannot close stdin

        // And wait for the result
        let output = child
            .wait_with_output()
            .expect("Failed to get generated piped PlantUML image");

        if output.status.success() {
            // Stdout contains the image data
            Ok(output.stdout)
        } else {
            Err(format_err!(
                "Failed to render image in piped mode (return value {})",
                output.status
            ))
        }
    }
}

struct FilePlantUMLRunner;
/// Traditional file based renderer. Simply writes a file with the PlantUML source to disk and reads back the output file
impl PlantUMLRunner for FilePlantUMLRunner {
    fn run(&self, plantuml_cmd: &str, plantuml_src: &str, format: &str) -> Result<Vec<u8>> {
        // Generate the file in a tmpdir
        let generation_dir = tempdir().expect("Failed to create PlantUML temp dir");

        const SRC_FILE_NAME: &str = "src.puml";
        let src_file = generation_dir.path().join(SRC_FILE_NAME);

        fs::write(&src_file, plantuml_src).expect("Failed to write PlantUML source file");

        Command::new(plantuml_cmd)
            // There cannot be a space between -t and format! Otherwise PlantUML generates a PNG image
            .arg(format!("-t{}", format))
            .arg("-nometadata")
            .arg(&src_file.to_str().unwrap())
            .output()
            .expect("Failed to render image");

        // PlantUML creates an output file based on the format, it is not always the same as `format` though (e.g. braille outputs a file
        // with extension .braille.png)
        // Just see which other file is in the directory next to our source file. That's the generated one...
        let entries = fs::read_dir(generation_dir.path()).unwrap();
        for entry in entries {
            if let Ok(path) = entry {
                if path.file_name() != SRC_FILE_NAME {
                    let data =
                        fs::read(path.path()).expect("Failed to read generated PlantUML image.");
                    return Ok(data);
                }
            }
        }

        bail!("Failed to find generated PlantUML image.");
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

    /// Generate an image file from the given plantuml code.
    fn render_from_string(&self, plantuml_code: &str, image_format: &str) -> Result<Vec<u8>> {
        //let runner = PipedPlantUMLRunner {};
        let runner = FilePlantUMLRunner {};
        return runner.run(&self.plantuml_cmd, plantuml_code, image_format);
    }
}

impl PlantUMLBackend for PlantUMLShell {
    fn render_from_string(
        &self,
        plantuml_code: &str,
        image_format: &str,
        _output_file: &Path,
    ) -> Result<Vec<u8>> {
        Self::render_from_string(self, plantuml_code, image_format)
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
