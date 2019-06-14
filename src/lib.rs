//! A `mdbook` preprocessor to render PlantUML diagrams into the book dir as images

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate glob;
extern crate mdbook;
extern crate serde;
use std::fs;
use std::path::PathBuf;

use glob::glob;
use std::process::Command;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod plantumlconfig;
use failure::{Error, ResultExt};
use plantumlconfig::{PlantUMLConfig, PlantUMLSource};

/// The main entrypoint for this crate.
pub fn render(cfg: &mdbook::Config) -> Result<(), Error> {
    info!("Started rendering the plantuml images");
    let plantuml_cfg = get_plantuml_config(cfg)?;
    let build_dir = &cfg.build.build_dir;

    check_config(&plantuml_cfg)?;
    for (i, src) in plantuml_cfg.sources.iter().enumerate() {
        let img_output_path = get_source_output_dir(&src.output_dir, build_dir)?;
        create_output_directory(&img_output_path)
            .and_then(|_| render_source(&src, &plantuml_cfg.plantuml_cmd, &img_output_path))
            .or_else(|e| {
                let msg = format!(
                    "Failed to generate UML diagrams for source entry {} ({})",
                    i, e
                );
                bail!(msg)
            })?;
    }

    Ok(())
}

/// Return the absolute path of the output path.
/// The output path is assumed to be relative to the book build dir.
/// If the output path is already an absolute path this path is used.
fn get_source_output_dir(output_path: &PathBuf, build_dir: &PathBuf) -> Result<PathBuf, Error> {
    let mut output_dir = PathBuf::from(&build_dir);
    output_dir.push(output_path);

    // PlantUML (the version I use, 1.2019.00) internally changes the working
    // dir to the uml file dir.
    // So when specifying a relative path you end up copying the files to the
    // dir relative to the uml file.
    // Hence we get the absolute path of the output dir
    let abs_path = fs::canonicalize(output_dir.as_path())?;

    Ok(abs_path)
}

/// Create the output directory
fn create_output_directory(output_dir: &PathBuf) -> Result<(), Error> {
    match fs::create_dir_all(output_dir) {
        Err(e) => {
            let msg = format!(
                "Failed to create PlantUML image output directory '{}' ({}))",
                output_dir.to_string_lossy(),
                e
            );
            bail!(msg);
        }
        Ok(()) => (),
    };

    Ok(())
}

/// Get the PlantUMLConfig from the book config
fn get_plantuml_config(cfg: &mdbook::Config) -> Result<PlantUMLConfig, Error> {
    match cfg.get("output.plantuml") {
        Some(raw) => raw
            .clone()
            .try_into()
            .context("Unable to deserialize the `output.plantuml` table.")
            .map_err(Error::from),
        None => Ok(PlantUMLConfig::default()),
    }
}

/// Check the config for errors
fn check_config(cfg: &PlantUMLConfig) -> Result<(), Error> {
    if cfg.sources.is_empty() {
        bail!("Nothing to do, please specify on or more source sections.");
    }

    for entry in &cfg.sources {
        check_config_source_entry(&entry)?;
    }

    Ok(())
}

/// Check a config source entry for errors
fn check_config_source_entry(entry: &PlantUMLSource) -> Result<(), Error> {
    if entry.src.is_empty() {
        bail!(
            "Invalid source definition, src needs to contain at least one file, glob or directory."
        );
    }

    for src in &entry.src {
        match glob(src.as_str()) {
            Err(e) => {
                let msg = format!("Invalid source file {} glob pattern ({}).", src, e);
                bail!(msg);
            }
            Ok(paths) => {
                if paths.count() == 0 {
                    warn!("No source files found for pattern {}", src);
                }
            }
        };
    }

    Ok(())
}

/// Get the command line for rendering the given source entry
fn get_cmd_arguments(
    src: &PlantUMLSource,
    plantuml_cmd: &String,
    output_dir: &PathBuf,
) -> Result<Vec<String>, Error> {
    let mut args: Vec<String> = Vec::new();
    args.push(plantuml_cmd.clone());
    args.push(String::from("-o"));
    let output_dir = output_dir.to_str().unwrap();
    args.push(String::from(output_dir));

    for s in src.src.clone() {
        args.push(s);
    }

    Ok(args)
}

/// Render the given source entry using PlantUML
fn render_source(
    src: &PlantUMLSource,
    plantuml_cmd: &String,
    build_dir: &PathBuf,
) -> Result<(), Error> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C");
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd
    };

    let args = get_cmd_arguments(src, plantuml_cmd, build_dir)?;
    debug!("Executing '{}'", args.join(" "));

    match cmd.args(args).output() {
        Err(e) => {
            let msg = format!("Failed to generate PlantUML diagrams ({})", e);
            bail!(msg);
        }
        Ok(output) => {
            debug!("Successfully generated PlantUML diagrams.");
            debug!("stdout: {}", String::from_utf8(output.stdout).unwrap());
            debug!("stderr: {}", String::from_utf8(output.stderr).unwrap());
        }
    }

    Ok(())
}

#[cfg(test)]
mod check_config_tests {
    use super::*;

    #[test]
    fn fails_when_no_sources() {
        let no_source = PlantUMLConfig::default();
        match check_config(&no_source) {
            Err(_) => assert!(true),
            Ok(_) => assert!(false, "Should fail when no source files are found"),
        }
    }

    #[test]
    fn fails_for_empty_source_definition() {
        let mut empty_source = PlantUMLConfig::default();
        empty_source.sources.push(PlantUMLSource::default());

        match check_config(&empty_source) {
            Err(_) => assert!(true),
            Ok(_) => assert!(false, "Should fail when empty source definitions are found"),
        }
    }

    #[test]
    fn fails_when_source_has_invalid_glob_pattern() {
        let source_definition = PlantUMLSource {
            src: vec![String::from("[*")],
            output_dir: PathBuf::from("."),
        };
        let mut cfg = PlantUMLConfig::default();
        cfg.sources.push(source_definition);

        match check_config(&cfg) {
            Err(_) => assert!(true),
            Ok(_) => assert!(
                false,
                "Should fail when invalid source glob pattern is found"
            ),
        }
    }
}
