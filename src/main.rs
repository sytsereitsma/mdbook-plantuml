use anyhow::Result;
use clap::{Parser, Subcommand};
use mdbook_plantuml::plantuml_config;
use mdbook_preprocessor::Preprocessor;
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

    let preprocessor = mdbook_plantuml::Preprocessor;
    if let Some(Command::Supports { renderer }) = args.command {
        handle_supports(&preprocessor, &renderer);
    } else if let Err(e) = handle_preprocessing(&preprocessor, args.log) {
        panic!("{}", e);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor, log_to_file: bool) -> Result<()> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    let config = plantuml_config(&ctx);
    setup_logging(log_to_file, config.verbose)?;

    log::debug!(
        "============================== Starting preprocessor ============================"
    );
    log::info!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    if ctx.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
        // We should probably use the `semver` crate to check compatibility
        // here...
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, but we're being \
             called from version {}",
            pre.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    // Preprocess the book
    let processed_book = pre.run(&ctx, book)?;

    // And let mdbook know the result
    serde_json::to_writer(io::stdout(), &processed_book)?;

    // Save the output to file too (uncomment when debugging)
    // use std::fs::File;
    // match File::create("mdbook-plantuml_back-to-mdbook.json") {
    //     Err(why) => eprintln!("couldn't open mdbook-plantuml_back-to-mdbook.json: {}", why),
    //     Ok(file) => serde_json::to_writer_pretty(file, &processed_book)?,
    // };

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, renderer: &str) -> ! {
    // Signal whether the renderer is supported by exiting with 1 (not supported) or 0 (supported).
    match pre.supports_renderer(renderer) {
        Err(e) => {
            eprintln!("Error checking renderer support: {}", e);
            process::exit(1);
        }
        Ok(has_support) => {
            process::exit(if has_support { 0 } else { 1 });
        }
    }
}

fn setup_logging(log_to_file: bool, verbose: bool) -> Result<()> {
    use log::LevelFilter;
    use log4rs::append::console::{ConsoleAppender, Target};
    use log4rs::append::file::FileAppender;
    use log4rs::filter::threshold::ThresholdFilter;

    use log4rs::config::{Appender, Config, Root};
    use log4rs::encode::pattern::PatternEncoder;

    // Whatever you do, DO NOT, log to stdout. Stdout is only for communication with mdbook
    let log_std_err = ConsoleAppender::builder().target(Target::Stderr).build();
    let mut config_builder = Config::builder().appender({
        let log_level = if verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        };

        Appender::builder()
            .filter(Box::new(ThresholdFilter::new(log_level)))
            .build("logstderr", Box::new(log_std_err))
    });

    if log_to_file {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build("output.log")?;
        config_builder =
            config_builder.appender(Appender::builder().build("logfile", Box::new(logfile)));
    }

    let mut root_builder = Root::builder();
    root_builder = root_builder.appender("logstderr");
    if log_to_file {
        root_builder = root_builder.appender("logfile");
    }

    let config = config_builder.build(root_builder.build(LevelFilter::Debug))?;
    log4rs::init_config(config)?;

    Ok(())
}
