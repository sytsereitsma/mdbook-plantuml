//! A `mdbook` preprocessor to render PlantUML diagrams into the book dir as images

#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;
extern crate glob;
extern crate mdbook;
extern crate path_absolutize;
extern crate serde;
extern crate tempfile;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod plantuml;
mod plantumlconfig;

use failure::{Error, ResultExt};
use glob::glob;
use plantuml::{PlantUML, PlantUMLRenderer};
use plantumlconfig::{PlantUMLConfig, PlantUMLSource};
use std::env;
use std::fs;
use std::path::PathBuf;

/// The main entrypoint for this crate.
pub fn render(cfg: &mdbook::Config, standalone: bool) -> Result<(), Error> {
    info!("Started rendering the plantuml images");
    let plantuml_cfg = get_plantuml_config(cfg)?;
    let build_dir = &cfg.build.build_dir;
    let plantuml = PlantUML::new(&plantuml_cfg.plantuml_cmd);
    check_config(&plantuml_cfg)?;
    for (i, src) in plantuml_cfg.sources.iter().enumerate() {
        let img_output_path = get_source_output_dir(&src.output_dir, build_dir, standalone)?;
        create_output_directory(&img_output_path)
            .and_then(|_| plantuml.render_files(&src.src, Some(img_output_path)))
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
/// When running from mdBook itself (i.e. not standalone), the working dir has
/// already changed to the book dir, so we do not need to append the relative
/// output path to it.
fn get_source_output_dir(
    output_path: &PathBuf,
    build_dir: &PathBuf,
    standalone: bool,
) -> Result<PathBuf, Error> {
    use path_absolutize::*;

    let mut output_dir;
    if standalone {
        output_dir = PathBuf::from(&build_dir);
    } else {
        output_dir = env::current_dir()?;
    }
    output_dir.push(output_path);

    // PlantUML (the version I use, 1.2019.00) internally changes the working
    // dir to the uml file dir.
    // So when specifying a relative path you end up copying the files to the
    // dir relative to the uml file.
    // Hence we need to get the absolute path of the output dir
    let abs_path = output_dir.absolutize().or_else(|e| {
        let msg = format!(
            "Failed to get absolute output directory for '{}' ({})",
            output_dir.display(),
            e
        );
        bail!(msg)
    })?;

    Ok(abs_path)
}

/// Create the output directory
fn create_output_directory(output_dir: &PathBuf) -> Result<(), Error> {
    match fs::create_dir_all(output_dir) {
        Err(e) => {
            let msg = format!(
                "Failed to create PlantUML image output directory '{}' ({}))",
                output_dir.display(),
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
