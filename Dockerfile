# Build stage
FROM rust:1.87-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Final stage
FROM scratch
COPY --from=builder /app/target/release/rust-cli-template /rust-cli-template
ENTRYPOINT ["/rust-cli-template"]