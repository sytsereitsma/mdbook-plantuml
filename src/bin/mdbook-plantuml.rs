extern crate env_logger;
extern crate failure;
extern crate mdbook;
extern crate mdbook_plantuml;
extern crate serde_json;
extern crate structopt;
#[macro_use]
extern crate log;

use failure::{Error, ResultExt, SyncFailure};
use mdbook::renderer::RenderContext;
use mdbook::MDBook;
use std::io;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

fn main() {
    env_logger::init();
    let args = Args::from_args();

    if let Err(e) = run(&args) {
        eprintln!("Failed to generate PlantUML images ({})", e);
        process::exit(1);
    }
}

fn run(args: &Args) -> Result<(), Error> {
    // get a `RenderContext`, either from stdin (because we're used as a plugin)
    // or by instrumenting MDBook directly (in standalone mode).
    let ctx: RenderContext = if args.standalone {
        //TODO: check if root exists
        //TODO: change cwd to root
        info!("Rendering book at '{}'", args.root.display());
        let md = MDBook::load(&args.root).map_err(SyncFailure::new)?;
        let destination = md.build_dir_for("epub");
        RenderContext::new(md.root, md.book, md.config, destination)
    } else {
        serde_json::from_reader(io::stdin()).context("Unable to parse RenderContext")?
    };

    mdbook_plantuml::render(&ctx.config, args.standalone)?;

    Ok(())
}

#[derive(Debug, Clone, StructOpt)]
struct Args {
    #[structopt(
        short = "s",
        long = "standalone",
        help = "Run standalone (i.e. not as a mdbook plugin)"
    )]
    standalone: bool,
    #[structopt(
        help = "The path to the book to render.",
        parse(from_os_str),
        default_value = "."
    )]
    root: PathBuf,
}
