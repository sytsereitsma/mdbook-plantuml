use clap::{Parser, Subcommand};
use mdbook::errors::Error as MDBookError;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_plantuml::PlantUMLPreprocessor;
use std::error::Error;
use std::io;
use std::process;

#[derive(Parser)]
#[clap(version, author, about)]
pub struct Args {
    /// Log to './output.log'
    ///
    /// (may help troubleshooting rendering issues).
    #[clap(short, long)]
    log: bool,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Check whether a renderer is supported by this preprocessor
    Supports { renderer: String },
}

fn main() {
    let args = Args::parse();

    let preprocessor = PlantUMLPreprocessor;
    if let Some(Command::Supports { renderer }) = args.command {
        handle_supports(&preprocessor, &renderer);
    } else {
        if args.log {
            if let Err(e) = setup_logging() {
                eprintln!("{}", e);
                process::exit(2);
            }
        }
        if let Err(e) = handle_preprocessing(&preprocessor) {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), MDBookError> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        // We should probably use the `semver` crate to check compatibility
        // here...
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, but we're being \
             called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }
    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, renderer: &str) -> ! {
    // Signal whether the renderer is supported by exiting with 1 or 0.
    if pre.supports_renderer(renderer) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn setup_logging() -> Result<(), Box<dyn Error>> {
    use log::LevelFilter;
    use log4rs::append::file::FileAppender;
    use log4rs::config::{Appender, Config, Root};
    use log4rs::encode::pattern::PatternEncoder;

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug),
        )?;
    log4rs::init_config(config)?;

    log::info!("--- Started preprocessor ---");

    Ok(())
}
