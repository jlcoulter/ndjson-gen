mod generate;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

/// Generate NDJSON files of a specified size with realistic fake data.
#[derive(Parser)]
#[command(name = "ndjson-gen", version, about)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an NDJSON file of the given size
    Generate {
        /// Target file size (e.g. 10MB, 1GB, 512KB, or raw bytes)
        size: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Generate NDJSON from an OpenAPI schema
    GenerateOpenapi {
        /// Target file size (e.g. 10MB, 1GB, 512KB, or raw bytes)
        size: String,

        /// OpenAPI spec file path (.yaml, .yml, or .json)
        #[arg(short, long)]
        spec: PathBuf,

        /// Name of schema under components/schemas
        #[arg(short, long)]
        schema: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
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
        Commands::Generate { size, output } => {
            let target = size.parse::<generate::Size>()?;
            generate::generate(target, &output)?;
        }
        Commands::GenerateOpenapi {
            size,
            spec,
            schema,
            output,
        } => {
            let target = size.parse::<generate::Size>()?;
            generate::generate_from_openapi(target, &output, &spec, &schema)?;
        }
    }

    Ok(())
}
