# Rust CLI Template

A GitHub template for Rust CLI tools — clap arguments, anyhow errors, structured logging, tests, CI, and Docker included.

## Features

- **CLI framework** with [clap](https://clap.rs/) (derive API)
- **Error handling** with [anyhow](https://docs.rs/anyhow) + custom error types with [thiserror](https://docs.rs/thiserror)
- **Structured logging** with [tracing](https://docs.rs/tracing) + [tracing-subscriber](https://docs.rs/tracing-subscriber)
- **Tests** — unit tests in `src/`, integration tests in `tests/`
- **CI** via GitHub Actions (test, clippy, fmt, build on push/PR)
- **Docker** multi-stage build (scratch image, ~3MB static binary)
- **Makefile** for common tasks

## Usage

1. Click **"Use this template"** on GitHub to create a new repo
2. Run the setup script:
   ```sh
   ./setup.sh mytool
   ```
3. Add your modules in `src/`
4. Delete the example module when you have your own

## Project Structure

```
.
├── src/
│   ├── main.rs            # Entry point, clap CLI, dispatch
│   ├── lib.rs             # Re-exports for integration tests
│   └── example.rs         # Example module (delete me)
├── tests/
│   └── integration_test.rs # Integration tests
├── .github/
│   └── workflows/
│       └── ci.yml          # Test + clippy + fmt
├── Cargo.toml
├── Dockerfile
├── Makefile
├── setup.sh
└── README.md
```

## Quick Start

```sh
# Run locally
make run

# Run tests
make test

# Build release binary
make build

# Build Docker image
make docker

# Lint
make lint
```

## Container Images

CI builds and pushes a container image to GHCR on every push to any branch.

```sh
# Pull the latest image
docker pull ghcr.io/<owner>/rust-cli-template:latest

# Pull a specific commit
docker pull ghcr.io/<owner>/rust-cli-template:<sha>

# Run
docker run ghcr.io/<owner>/rust-cli-template:latest
```

Replace `<owner>` with your GitHub username or org. Images are tagged with both `latest` and the commit SHA.

## Adding a New Module

```rust
// src/mymodule.rs
use anyhow::Result;

pub fn do_thing(input: &str) -> Result<String> {
    tracing::info!(input, "processing");
    Ok(format!("processed: {input}"))
}
```

Then add `mod mymodule;` to `src/lib.rs` and wire it into a clap subcommand in `src/main.rs`.

## License

MIT