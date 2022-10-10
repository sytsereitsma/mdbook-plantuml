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
        if let Err(e) = setup_logging(args.log) {
            eprintln!("{}", e);
            process::exit(2);
        }

        log::debug!(
            "============================== Starting preprocessor ============================"
        );
        if let Err(e) = handle_preprocessing(&preprocessor) {
            log::error!("{}", e);
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
    // Signal whether the renderer is supported by exiting with 1 or 0.
    if pre.supports_renderer(renderer) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn setup_logging(log_to_file: bool) -> Result<(), Box<dyn Error>> {
    use log::LevelFilter;
    use log4rs::append::console::{ConsoleAppender, Target};
    use log4rs::append::file::FileAppender;
    use log4rs::filter::threshold::ThresholdFilter;

    use log4rs::config::{Appender, Config, Root};
    use log4rs::encode::pattern::PatternEncoder;

    // Whatever you do, DO NOT, log to stdout. Stdout is only for communication with mdbook
    let log_std_err = ConsoleAppender::builder().target(Target::Stderr).build();
    let mut config_builder = Config::builder().appender(
        Appender::builder()
            .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
            .build("logstderr", Box::new(log_std_err)),
    );

    if log_to_file {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build("output.log")?;
        config_builder =
            config_builder.appender(Appender::builder().build("logfile", Box::new(logfile)));
    }

    let config = config_builder.build(
        Root::builder()
            .appender("logfile")
            .appender("logstderr")
            .build(LevelFilter::Debug),
    )?;
    log4rs::init_config(config)?;

    Ok(())
}
