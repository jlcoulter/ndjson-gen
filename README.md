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

# Verbose logging
ndjson-gen generate 10MB --output data.ndjson -v

# Generate from OpenAPI schema
ndjson-gen generate-openapi 10MB --spec openapi.yaml --schema Order --output orders.ndjson
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

OpenAPI mode generates each record from the selected schema in `components/schemas`, filling fields with randomized values while honoring common schema constraints like enums, arrays, numbers, and nested objects.

The file size will meet or slightly exceed the target (by up to one record).

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