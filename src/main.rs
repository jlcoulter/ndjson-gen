mod example;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

/// A Rust CLI tool template — replace this description.
#[derive(Parser)]
#[command(name = "rust-cli-template", version, about)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the example command
    Example {
        /// Name to greet
        #[arg(default_value = "world")]
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level)),
        )
        .with_writer(std::io::stderr)
        .init();

    match cli.command {
        Commands::Example { name } => {
            let result = example::greet(&name)?;
            println!("{result}");
        }
    }

    Ok(())
}
