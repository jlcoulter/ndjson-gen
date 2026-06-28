//! ndjson-gen — Generate NDJSON files with realistic fake data.
//!
//! This crate provides a CLI tool and library for generating newline-delimited
//! JSON files of a specified size. Each line is a valid JSON object with randomized
//! fields (name, email, city, amount, etc.).
//!
//! # Library usage
//!
//! ```
//! use ndjson_gen::{generate, generate_into, Size};
//! use std::str::FromStr;
//!
//! let target = Size::from_str("10MB").unwrap();
//!
//! // Write to a file
//! generate(target, std::path::Path::new("output.ndjson")).unwrap();
//!
//! // Write to any Write sink
//! let mut buf: Vec<u8> = Vec::new();
//! generate_into(target, &mut buf).unwrap();
//! ```

pub mod generate;

pub use generate::{generate, generate_into, ParseSizeError, Record, Size};
