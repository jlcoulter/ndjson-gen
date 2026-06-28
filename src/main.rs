use clap::{Parser, Subcommand};
use ndjson_gen::{generate, generate_into, Size};
use std::io;
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

        /// Output file path (use --stdout to write to stdout instead)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Write to stdout instead of a file
        #[arg(long, conflicts_with = "output")]
        stdout: bool,

        /// Random seed for reproducible output
        #[arg(long)]
        seed: Option<u64>,
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
        Commands::Generate {
            size,
            output,
            stdout,
            seed,
        } => {
            let target = size.parse::<Size>()?;

            // TODO: use seed once rand::SeedableRng is wired through
            if let Some(s) = seed {
                tracing::info!(seed = s, "seed specified (not yet wired to RNG)");
            }

            if stdout {
                generate_into(target, io::stdout().lock())?;
            } else {
                let path = output
                    .ok_or_else(|| anyhow::anyhow!("--output <path> or --stdout is required"))?;
                generate(target, &path)?;
            }
        }
    }

    Ok(())
}
