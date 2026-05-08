mod cmd_callers;
mod cmd_def;
mod cmd_impact;
mod cmd_impl;
mod cmd_index;
mod cmd_refs;
mod output;

use std::process;

use clap::{Parser, Subcommand};

use who_core::error::ExitCode;

#[derive(Parser)]
#[command(
    name = "who",
    version,
    about = "Semantic code intelligence for humans and AI agents"
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

    /// Output as NDJSON
    #[arg(long, global = true)]
    ndjson: bool,

    /// Output in quickfix format
    #[arg(long, value_name = "FORMAT", global = true)]
    format: Option<String>,

    /// Show explain/why information
    #[arg(long, global = true)]
    why: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Find callers of a function at a source location
    Call {
        /// Target location
        target: String,
    },
    /// Resolve a call site to its definition
    Def {
        /// Target location
        target: String,
    },
    /// Find all references to a symbol
    Refs {
        /// Target location
        target: String,
    },
    /// Find implementations of a trait/interface method
    Impl {
        /// Target location
        target: String,
    },
    /// Find likely impact of changing a function
    Impact {
        /// Target location
        target: String,
        /// Depth of caller chain to traverse
        #[arg(long, default_value = "2")]
        depth: u32,
        /// Include tests in output
        #[arg(long)]
        tests: bool,
    },
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

    let output_opts = output::OutputOpts {
        json: cli.json,
        ndjson: cli.ndjson,
        format: cli.format.clone(),
        why: cli.why,
    };

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
        Some(Commands::Call { target }) => cmd_callers::run(&target, &output_opts),
        Some(Commands::Def { target }) => cmd_def::run(&target, &output_opts),
        Some(Commands::Refs { target }) => cmd_refs::run(&target, &output_opts),
        Some(Commands::Impl { target }) => cmd_impl::run(&target, &output_opts),
        Some(Commands::Impact {
            target,
            depth,
            tests,
        }) => cmd_impact::run(&target, depth, tests, &output_opts),
        None => {
            if let Some(target) = cli.target {
                cmd_callers::run(&target, &output_opts)
            } else {
                eprintln!("Usage: who <target> or whocall <target> or whoimpl <target>");
                eprintln!("Run 'who --help' for more information.");
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
