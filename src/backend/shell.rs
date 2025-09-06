use crate::backend::Backend;
use anyhow::{Context, Result, bail, format_err};

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::tempdir;

/// Split a shell command into its parts, e.g. "python D:\\foo" will become ["Python", "D:/Foo"]
pub fn split_shell_command(cmd: &str) -> Result<Vec<String>> {
    let preprocessed: String = {
        // Windows paths are converted to forward slash paths (shell_words and shlex both assume
        // posix paths and treat the backslashes as escape characters), which would make C:\foo\bar
        // become C:foobar
        if cfg!(target_family = "windows") {
            cmd.replace('\\', "/")
        } else {
            String::from(cmd)
        }
    };

    let cmd_parts =
        shlex::split(preprocessed.as_str()).ok_or_else(|| format_err!("Invalid command"))?;
    Ok(cmd_parts)
}

fn create_command(plantuml_cmd: &str) -> Result<Command> {
    let cmd_parts = split_shell_command(plantuml_cmd)?;

    let mut command = Command::new(&cmd_parts[0]);
    command.args(&cmd_parts[1..]);

    Ok(command)
}

struct PipedRunner;
impl PipedRunner {
    fn run(plantuml_cmd: &str, plantuml_src: &str, format: &str) -> Result<Vec<u8>> {
        let mut child = create_command(plantuml_cmd)?
            // There cannot be a space between -t and format! Otherwise PlantUML generates a PNG image
            .arg(format!("-t{format}"))
            .arg("-nometadata")
            .arg("-pipe")
            .arg("-pipeNoStderr")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to start PlantUML command '{plantuml_cmd}' "))?;

        // Pipe the plantuml source
        child
            .stdin
            .take()
            .unwrap() // We can simply unwrap, because we know stdin is piped
            .write_all(plantuml_src.as_bytes())
            .with_context(|| "Failed to pipe PlantUML code")?;

        // And wait for the result
        let output = child
            .wait_with_output()
            .with_context(|| "Failed to get generated piped PlantUML image")?;

        // If the command was successful and there is no stderr output, otherwise stderr contains the error message
        if output.status.success() && output.stderr.is_empty() {
            Ok(output.stdout)
        } else {
            log::error!("Failed to render image in piped mode ({})", output.status);
            Err(format_err!(
                "Failed to render image in piped mode ({})\n  stdout: '{}'\n  stderr: '{}'",
                output.status,
                String::from_utf8(output.stdout).unwrap_or_default(),
                String::from_utf8(output.stderr).unwrap_or_default(),
            ))
        }
    }
}

/// Traditional file based renderer. Simply writes a file with the PlantUML source to disk and reads back the output file
struct FileRunner;
impl FileRunner {
    fn find_generated_file(generation_dir: &Path, src_file_name: &str) -> Result<PathBuf> {
        // PlantUML creates an output file based on the format, it is not always the same as `format` though (e.g. braille outputs a file
        // with extension `.braille.png`)
        // Just see which other file is in the directory next to our source file. That's the generated one...
        let entries = fs::read_dir(generation_dir)?;

        // Now find the generated file
        for path in entries.flatten() {
            if path.file_name() != src_file_name {
                return Ok(path.path());
            }
        }

        bail!("Failed to find generated PlantUML image.");
    }

    fn run(plantuml_cmd: &str, plantuml_src: &str, format: &str) -> Result<Vec<u8>> {
        // Generate the file in a tmpdir
        let generation_dir = tempdir().with_context(|| "Failed to create PlantUML tempdir")?;

        // Write the PlantUML source file
        const SRC_FILE_NAME: &str = "src.puml";
        let src_file = generation_dir.path().join(SRC_FILE_NAME);
        fs::write(&src_file, plantuml_src)
            .with_context(|| "Failed to write PlantUML source file")?;

        // Call PlantUML
        create_command(plantuml_cmd)?
            // There cannot be a space between -t and format! Otherwise PlantUML generates a PNG image
            .arg(format!("-t{format}"))
            .arg("-nometadata")
            .arg(src_file.to_str().unwrap())
            .output()
            .with_context(|| "Failed to render image")?;

        let generated_file = Self::find_generated_file(generation_dir.path(), SRC_FILE_NAME)?;
        fs::read(generated_file).with_context(|| "Failed to read rendered image")
    }
}

pub struct PlantUMLShell {
    plantuml_cmd: String,
    piped: bool,
}

/// Invokes PlantUML as a shell/cmd program.
impl PlantUMLShell {
    pub fn new(plantuml_cmd: String, piped: bool) -> Self {
        log::info!(
            "Selected PlantUML shell '{}' (piped={})",
            &plantuml_cmd,
            piped
        );
        Self {
            plantuml_cmd,
            piped,
        }
    }
}

impl Backend for PlantUMLShell {
    fn render_from_string(&self, plantuml_code: &str, image_format: &str) -> Result<Vec<u8>> {
        if self.piped {
            PipedRunner::run(&self.plantuml_cmd, plantuml_code, image_format)
        } else {
            FileRunner::run(&self.plantuml_cmd, plantuml_code, image_format)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_generated_file() {
        let generation_dir = tempdir().unwrap();

        let found_file = FileRunner::find_generated_file(generation_dir.path(), "somefile.txt");
        assert!(found_file.is_err());
    }

    #[test]
    fn test_split_shell_command() {
        assert!(split_shell_command("").unwrap().is_empty());

        // String with multiple arguments
        assert_eq!(
            vec![
                String::from("python"),
                String::from("foo"),
                String::from("bar")
            ],
            split_shell_command("python foo bar").unwrap()
        );

        // Unclosed quoted string
        assert!(split_shell_command("python \"/foo").is_err());

        if cfg!(target_family = "windows") {
            // On windows backslashes are converted to forward slashes paths
            assert_eq!(
                vec![String::from("python"), String::from("D:/foo/bar")],
                split_shell_command("python D:\\foo\\bar").unwrap()
            );

            // String with escaped space (escaping with backslashes is not a thing on windows)
            assert_eq!(
                vec![
                    String::from("python"),
                    String::from("foo/"),
                    String::from("bar")
                ],
                split_shell_command("python foo\\ bar").unwrap()
            );
        }

        // And on non windows platforms they are treated as posix paths, meaning backslashes are treated as escape characters
        if !cfg!(target_family = "windows") {
            assert_eq!(
                vec![String::from("python"), String::from("D:foobar")],
                split_shell_command("python D:\\foo\\bar").unwrap()
            );

            // String with escaped spaces
            assert_eq!(
                vec![String::from("python"), String::from("foo bar")],
                split_shell_command("python foo\\ bar").unwrap()
            );
        }
    }
}
