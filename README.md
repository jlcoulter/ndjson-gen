# ndjson-gen

Generate NDJSON files of a specified size with realistic fake data.

Creates newline-delimited JSON files where each line is a valid JSON object with randomized records — useful for benchmarks, test fixtures, and data pipeline development.

## Usage

```sh
# Generate a 10MB file
ndjson-gen generate 10MB --output data.ndjson

# Generate a 1GB file
ndjson-gen generate 1GB --output big.ndjson

# Generate a 512KB file
ndjson-gen generate 512KB --output small.ndjson

# Specify size in raw bytes
ndjson-gen generate 1048576 --output exact.ndjson

# Write to stdout (pipe-friendly)
ndjson-gen generate 1MB --stdout | head -n 5

# Verbose logging
ndjson-gen generate 10MB --output data.ndjson -v

# Seed for reproducible output
ndjson-gen generate 10MB --output data.ndjson --seed 42
```

### Size units

| Unit | Meaning |
|------|---------|
| `B`  | Bytes |
| `KB` | Kilobytes (1024 bytes) |
| `MB` | Megabytes (1024² bytes) |
| `GB` | Gigabytes (1024³ bytes) |

No unit is interpreted as raw bytes. Case-insensitive.

## Output format

Each line is a JSON object:

```json
{"id":1,"name":"Alice Smith","email":"alice.smith@example.com","city":"Springfield","state":"IL","zip":"62704","amount":423.50,"status":"active","timestamp":"2024-03-15T14:22:08Z"}
```

The file size will meet or slightly exceed the target (by up to one record).

## Library

This crate can also be used as a library:

```rust
use ndjson_gen::{generate, generate_into, Size};
use std::str::FromStr;

let target = Size::from_str("10MB").unwrap();

// Write to a file
generate(target, std::path::Path::new("output.ndjson")).unwrap();

// Write to any Write sink
let mut buf: Vec<u8> = Vec::new();
generate_into(target, &mut buf).unwrap();
```

## Install

```sh
cargo build --release
# Binary at target/release/ndjson-gen
```

Or pull the Docker image:

```sh
docker pull ghcr.io/jlcoulter/ndjson-gen:latest
```

## Test

```sh
make test
make lint
```

## Docker

```sh
make docker
docker run --rm -v $(pwd)/data:/data ndjson-gen generate 10MB --output /data/out.ndjson
```

Multi-arch images (amd64 + arm64) are built and pushed to GHCR on every push to `main`.

## License

MIT