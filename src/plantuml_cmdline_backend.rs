use crate::plantuml_backend::PlantUMLBackend;
use anyhow::{bail, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// A trait class for wrapping the actual rendering command.
///
/// Only here to make unit testing the renderer possible, this is cheating a
/// bit, but the other option is not testing it at all, or partially through
/// integration tests.
pub trait CommandExecutor {
    fn execute(&self, dir: &str, args: Vec<String>, input: &[u8]) -> Result<Vec<u8>>;
}

pub struct RealCommandExecutor;

impl CommandExecutor for RealCommandExecutor {
    fn execute(&self, dir: &str, args: Vec<String>, input: &[u8]) -> Result<Vec<u8>> {
        let mut command = Command::new(&args[0]);
        command.args(&args[1..]).current_dir(dir);

        log::debug!("Command: {:?}", &command);
        log::debug!("Working dir {:?}", dir);

        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = match child.stdin.take() {
            Some(val) => val,
            None => bail!("Cannot take stdin!"),
        };
        stdin.write_all(input)?;
        log::debug!("{} bytes written", input.len());
        drop(stdin); // IMPORTANT!!! If omitted plantuml will halt!

        let output = child.wait_with_output()?;
        if output.status.success() {
            log::debug!("Command success {:?}", output.status);
        } else {
            log::error!(
                "Command error {:?}: {:?}",
                output.status,
                String::from_utf8_lossy(output.stderr.as_slice())
            );
            bail!(
                "Command error {:?}",
                String::from_utf8_lossy(output.stderr.as_slice()),
            );
        }

        Ok(output.stdout)
    }
}

pub struct PlantUMLExecutableBackend {
    args: Vec<String>,
    executor: Box<dyn CommandExecutor>,
}

/// Invokes PlantUML as a shell/cmd program.
impl PlantUMLExecutableBackend {
    pub fn new(command: String, executor: Option<Box<dyn CommandExecutor>>) -> Self {
        Self {
            args: command.split_whitespace().map(|a| a.to_owned()).collect(),
            executor: match executor {
                Some(v) => v,
                None => Box::new(RealCommandExecutor),
            },
        }
    }
}

impl PlantUMLBackend for PlantUMLExecutableBackend {
    /// Generate an image file from the given plantuml code.
    fn render_from_string(
        &self,
        plantuml_code: &str,
        chapter_path: &str,
        image_format: &str,
        output_file: &Path,
    ) -> Result<()> {
        let mut args: Vec<String> = self.args.clone();
        args.push(format!("-t{}", image_format));
        args.push(String::from("-nometadata"));
        args.push(String::from("-pipe"));

        let output = self
            .executor
            .execute(chapter_path, args, plantuml_code.as_bytes())?;
        fs::write(output_file, &output)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::join_path;
    use anyhow::bail;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    const THE_ERROR: &str = "The ERROR!";
    const TEST_DIR: &str = "testdir";

    struct FakeCommandExecutor {
        expect_dir: String,
        error: bool,
        output: Box<Vec<u8>>,
    }

    impl CommandExecutor for FakeCommandExecutor {
        fn execute(&self, dir: &str, _: Vec<String>, _: &[u8]) -> Result<Vec<u8>> {
            assert_eq!(self.expect_dir.as_str(), dir);

            if self.error {
                bail!(THE_ERROR)
            }

            Ok(self.output.as_ref().clone())
        }
    }

    fn run_render_from_string(
        source: Option<&String>,
        output: Option<&String>,
        generate_error: bool,
        read_result: bool,
    ) -> Result<String> {
        let output_dir = tempdir().unwrap();
        let output_data: Vec<u8> = match output {
            Some(data) => data.clone().into_bytes(),
            None => Vec::default(),
        };
        let output_file = join_path(output_dir.path(), "foobar.svg");

        let executor: Box<dyn CommandExecutor> = Box::new(FakeCommandExecutor {
            expect_dir: String::from(TEST_DIR),
            error: generate_error,
            output: Box::new(output_data),
        });
        let backend: Box<dyn PlantUMLBackend> = Box::new(PlantUMLExecutableBackend {
            args: vec![String::from("plantuml")],
            executor,
        });

        backend.render_from_string(
            source.map_or("@startuml\nA--|>B\n@enduml", AsRef::as_ref),
            TEST_DIR,
            "svg",
            &output_file,
        )?;

        if read_result {
            let raw_source = fs::read(&output_file)?;
            return Ok(String::from_utf8_lossy(&raw_source).into_owned());
        }

        Ok(String::default())
    }

    #[test]
    fn command_failure() {
        match run_render_from_string(None, None, true, false) {
            Ok(_file_data) => panic!("Expected the command to fail"),
            Err(e) => assert_eq!(e.to_string(), THE_ERROR),
        };
    }

    #[test]
    fn returns_image_file_path_on_success() {
        let expected_output = String::from("svg code here");
        match run_render_from_string(
            Some(&String::from("My plantuml code")),
            Some(&expected_output),
            false,
            true,
        ) {
            Ok(file_data) => {
                assert_eq!(expected_output, file_data);
            }
            Err(e) => panic!("{}", e),
        };
    }
}
