use crate::plantuml_backend::PlantUMLBackend;
use anyhow::{bail, format_err, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::tempdir;
use shlex::Shlex;

fn create_command(plantuml_cmd: &str) -> Command {
    // No need to check had_errors for lex, that was already done by the backend factory
    let mut lex = Shlex::new(plantuml_cmd);
    let cmd_parts = lex.by_ref().collect::<Vec<_>>();

    let mut command = Command::new(&cmd_parts[0]);
    command.args(&cmd_parts[1..]);

    command
}

struct PipedPlantUMLRunner;
impl PipedPlantUMLRunner {
    fn run(plantuml_cmd: &str, plantuml_src: &str, format: &str) -> Result<Vec<u8>> {

        let mut child = create_command(plantuml_cmd)
            // There cannot be a space between -t and format! Otherwise PlantUML generates a PNG image
            .arg(format!("-t{}", format))
            .arg("-nometadata")
            .arg("-pipe")
            .arg("-pipeNoStderr")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to start PlantUML command '{}' ", plantuml_cmd))?;

        // Pipe the plantuml source
        child
            .stdin
            .take()
            .unwrap() // We can simply unwrap, because we know stdin is piped
            .write_all(plantuml_src.as_bytes())
            .and_then(|stdin| {
                drop(stdin);
                Ok(())
            })
            .with_context(|| "Failed to pipe PlantUML code")?;

        // And wait for the result
        let output = child
            .wait_with_output()
            .with_context(|| "Failed to get generated piped PlantUML image")?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(format_err!(
                "Failed to render image in piped mode ({})\n  stdout: '{}'\n  stderr: '{}'",
                output.status,
                String::from_utf8(output.stdout).unwrap_or(String::from("")),
                String::from_utf8(output.stderr).unwrap_or(String::from(""))
            ))
        }
    }
}

/// Traditional file based renderer. Simply writes a file with the PlantUML source to disk and reads back the output file
struct FilePlantUMLRunner;
impl FilePlantUMLRunner {
    fn find_generated_file(generation_dir: &Path, src_file_name: &str) -> Result<PathBuf> {
        // PlantUML creates an output file based on the format, it is not always the same as `format` though (e.g. braille outputs a file
        // with extension `.braille.png`)
        // Just see which other file is in the directory next to our source file. That's the generated one...
        let entries = fs::read_dir(generation_dir)?;

        // Now find the generated file
        for entry in entries {
            if let Ok(path) = entry {
                if path.file_name() != src_file_name {
                    return Ok(path.path());
                }
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
        create_command(plantuml_cmd)
            // There cannot be a space between -t and format! Otherwise PlantUML generates a PNG image
            .arg(format!("-t{}", format))
            .arg("-nometadata")
            .arg(&src_file.to_str().unwrap())
            .output()
            .with_context(|| "Failed to render image")?;

        let generated_file =
            FilePlantUMLRunner::find_generated_file(&generation_dir.path(), SRC_FILE_NAME)?;
        return fs::read(generated_file).with_context(|| "Failed to read rendered image");
    }
}

pub struct PlantUMLShell {
    plantuml_cmd: String,
    piped: bool,
}

/// Invokes PlantUML as a shell/cmd program.
impl PlantUMLShell {
    pub fn new(plantuml_cmd: String, piped: bool) -> Self {
        log::info!("Selected PlantUML shell {} (piped={})", &plantuml_cmd, piped);
        Self {
            plantuml_cmd,
            piped,
        }
    }
}

impl PlantUMLBackend for PlantUMLShell {
    fn render_from_string(&self, plantuml_code: &str, image_format: &str) -> Result<Vec<u8>> {
        if self.piped {
            PipedPlantUMLRunner::run(&self.plantuml_cmd, plantuml_code, image_format)
        } else {
            FilePlantUMLRunner::run(&self.plantuml_cmd, plantuml_code, image_format)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_generated_file() {
        let generation_dir = tempdir().unwrap();

        let found_file = FilePlantUMLRunner::find_generated_file(&generation_dir.path(), "somefile.txt");
        assert!(found_file.is_err());
    }
}
