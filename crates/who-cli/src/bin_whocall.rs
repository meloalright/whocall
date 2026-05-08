mod cmd_callers;
mod cmd_index;
mod output;

use std::process;

use clap::{Parser, Subcommand};

use who_core::error::ExitCode;

#[derive(Parser)]
#[command(
    name = "whocall",
    version,
    about = "Semantic code intelligence — find callers of a symbol"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Target (file:line, file:line:col, file#symbol, qualified::symbol, or plain name)
    #[arg(global = false)]
    target: Option<String>,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the local index
    Index {
        /// Path to index (defaults to current directory)
        path: Option<String>,
        /// Language filter
        #[arg(long)]
        lang: Option<String>,
        /// Watch for changes
        #[arg(long)]
        watch: bool,
        /// Clean and rebuild index
        #[arg(long)]
        clean: bool,
        /// Don't respect .gitignore
        #[arg(long)]
        no_gitignore: bool,
        /// Include patterns
        #[arg(long)]
        include: Option<String>,
        /// Exclude patterns
        #[arg(long)]
        exclude: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let output_opts = output::OutputOpts { json: cli.json };

    let result = match cli.command {
        Some(Commands::Index {
            path,
            lang,
            clean,
            no_gitignore,
            include,
            exclude,
            ..
        }) => cmd_index::run(cmd_index::IndexOpts {
            path: path.unwrap_or_else(|| ".".to_string()),
            lang,
            clean,
            no_gitignore,
            include,
            exclude,
        }),
        None => {
            if let Some(target) = cli.target {
                cmd_callers::run(&target, &output_opts)
            } else {
                eprintln!("Usage: whocall <target> or whocall index <path>");
                eprintln!("Run 'whocall --help' for more information.");
                process::exit(ExitCode::ParseError.code());
            }
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e:#}");
        let code = e
            .downcast_ref::<who_core::error::WhoError>()
            .map(|e| e.exit_code().code())
            .unwrap_or(ExitCode::InternalError.code());
        process::exit(code);
    }
}
