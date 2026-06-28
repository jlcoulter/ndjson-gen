//! NDJSON file generation with realistic fake data.
//!
//! This module provides the core generation logic: parsing size specifications,
//! creating fake records, and writing them as newline-delimited JSON.

use fake::Fake;
use rand::Rng;
use serde::Serialize;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

/// Error type for size string parsing.
#[derive(Debug, Error)]
pub enum ParseSizeError {
    /// The numeric portion could not be parsed.
    #[error("invalid size number: {0}")]
    InvalidNumber(String),
    /// The size value is zero, which would produce an empty file.
    #[error("size must be greater than zero")]
    ZeroSize,
}

/// Parsed byte size with optional unit (KB, MB, GB).
///
/// Supports strings like `10MB`, `1GB`, `512KB`, `1024B`, or raw bytes (`1048576`).
/// Case-insensitive, whitespace-tolerant.
///
/// # Examples
///
/// ```
/// use ndjson_gen::Size;
/// use std::str::FromStr;
///
/// let s = Size::from_str("10MB").unwrap();
/// assert_eq!(s.bytes(), 10 * 1_048_576);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Size {
    bytes: u64,
}

impl Size {
    /// Returns the target size in bytes.
    pub fn bytes(self) -> u64 {
        self.bytes
    }
}

impl FromStr for Size {
    type Err = ParseSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_uppercase();
        let (num, multiplier) = if s.ends_with("GB") {
            (&s[..s.len() - 2], 1_073_741_824u64)
        } else if s.ends_with("MB") {
            (&s[..s.len() - 2], 1_048_576u64)
        } else if s.ends_with("KB") {
            (&s[..s.len() - 2], 1_024u64)
        } else if s.ends_with("B") {
            (&s[..s.len() - 1], 1u64)
        } else {
            (s.as_str(), 1u64)
        };
        let n: u64 = num
            .trim()
            .parse()
            .map_err(|_| ParseSizeError::InvalidNumber(num.trim().to_string()))?;
        if n == 0 {
            return Err(ParseSizeError::ZeroSize);
        }
        Ok(Size {
            bytes: n * multiplier,
        })
    }
}

/// Schema for generated records — realistic fake data.
///
/// Each record contains an ID, name, email, city, state, zip, amount, status,
/// and timestamp. The `generate` method produces a randomized instance, and the
/// struct implements [`serde::Serialize`] so it can be serialized as a JSON object.
#[derive(Debug, Serialize)]
pub struct Record {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub amount: f64,
    pub status: String,
    pub timestamp: String,
}

/// Days in each month (non-leap year). Used to generate valid dates.
const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

impl Record {
    /// Generate a single fake record with the given ID and RNG.
    pub fn generate(id: u64, rng: &mut impl Rng) -> Self {
        let first = fake::faker::name::en::FirstName().fake::<String>();
        let last = fake::faker::name::en::LastName().fake::<String>();
        let domain = fake::faker::internet::en::DomainSuffix().fake::<String>();
        let city = fake::faker::address::en::CityName().fake::<String>();
        let state = fake::faker::address::en::StateAbbr().fake::<String>();
        let zip = fake::faker::address::en::ZipCode().fake::<String>();
        let statuses = ["active", "inactive", "pending", "closed"];

        let year: u32 = rng.random_range(2020..=2026);
        let month: u32 = rng.random_range(1..=12);
        let day: u32 = rng.random_range(1..=DAYS_IN_MONTH[(month - 1) as usize]);
        let hour: u32 = rng.random_range(0..=23);
        let minute: u32 = rng.random_range(0..=59);
        let second: u32 = rng.random_range(0..=59);

        // Sanitize name components for email — remove non-alphanumeric chars
        let first_clean: String = first
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        let last_clean: String = last.chars().filter(|c| c.is_ascii_alphanumeric()).collect();

        Record {
            id,
            name: format!("{first} {last}"),
            email: format!(
                "{}.{last_clean}@example.{domain}",
                first_clean.to_lowercase(),
                last_clean = last_clean.to_lowercase()
            ),
            amount: {
                let raw: f64 = rng.random_range(1.0..10000.0);
                (raw * 100.0).round() / 100.0
            },
            status: statuses[rng.random_range(0..statuses.len())].to_string(),
            timestamp: format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"),
            city,
            state,
            zip,
        }
    }
}

/// Generate NDJSON records into any [`Write`] sink until `target` bytes are reached.
///
/// Writes one JSON object per line. The output will be approximately `target` bytes
/// — it may overshoot by up to one record's worth of bytes.
///
/// # Errors
///
/// Returns an error if writing to `writer` fails.
pub fn generate_into(target: Size, writer: impl Write) -> Result<(), std::io::Error> {
    let mut writer = BufWriter::new(writer);
    let mut rng = rand::rng();
    let mut written: u64 = 0;
    let mut id: u64 = 1;

    while written < target.bytes {
        let record = Record::generate(id, &mut rng);
        let mut line = serde_json::to_string(&record).map_err(std::io::Error::other)?;
        line.push('\n');
        writer.write_all(line.as_bytes())?;
        written += line.len() as u64;
        id += 1;
    }

    writer.flush()?;
    tracing::info!(bytes = written, records = id - 1, "generation complete");
    Ok(())
}

/// Generate NDJSON data into a file until the target file size is reached.
///
/// Convenience wrapper around [`generate_into`] that creates and writes to a file.
/// The output file will meet or slightly exceed the target size (by up to one record).
pub fn generate(target: Size, output: &Path) -> Result<(), std::io::Error> {
    let file = std::fs::File::create(output)?;
    generate_into(target, file)?;
    tracing::info!(path = %output.display(), bytes = target.bytes(), "file written");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_bytes() {
        let s: Result<Size, ParseSizeError> = "1024".parse();
        assert_eq!(s.unwrap().bytes(), 1024);
    }

    #[test]
    fn parse_size_kb() {
        let s: Size = "5KB".parse().unwrap();
        assert_eq!(s.bytes(), 5 * 1024);
    }

    #[test]
    fn parse_size_mb() {
        let s: Size = "10MB".parse().unwrap();
        assert_eq!(s.bytes(), 10 * 1_048_576);
    }

    #[test]
    fn parse_size_gb() {
        let s: Size = "2GB".parse().unwrap();
        assert_eq!(s.bytes(), 2 * 1_073_741_824);
    }

    #[test]
    fn parse_size_case_insensitive() {
        let s: Size = "10mb".parse().unwrap();
        assert_eq!(s.bytes(), 10 * 1_048_576);
    }

    #[test]
    fn parse_size_with_spaces() {
        let s: Size = " 10 MB ".parse().unwrap();
        assert_eq!(s.bytes(), 10 * 1_048_576);
    }

    #[test]
    fn parse_size_zero_errors() {
        let err = "0".parse::<Size>().unwrap_err();
        assert!(matches!(err, ParseSizeError::ZeroSize));
    }

    #[test]
    fn parse_size_invalid_number() {
        let err = "abc".parse::<Size>().unwrap_err();
        assert!(matches!(err, ParseSizeError::InvalidNumber(_)));
    }

    #[test]
    fn record_generates_valid_json() {
        let mut rng = rand::rng();
        let rec = Record::generate(1, &mut rng);
        let json = serde_json::to_string(&rec).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
        assert_eq!(parsed["id"], 1);
    }

    #[test]
    fn record_email_is_lowercase_and_clean() {
        let mut rng = rand::rng();
        let rec = Record::generate(1, &mut rng);
        // Email should be all lowercase and contain no special chars before @
        let local_part = rec.email.split('@').next().unwrap();
        assert_eq!(local_part, local_part.to_lowercase());
        assert!(local_part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.'));
    }

    #[test]
    fn generate_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.ndjson");
        generate(Size { bytes: 512 }, &path).unwrap();
        assert!(path.exists());
        let contents = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert!(!lines.is_empty());
        for line in &lines {
            assert!(serde_json::from_str::<serde_json::Value>(line).is_ok());
        }
    }

    #[test]
    fn generate_respects_size_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.ndjson");
        let target = Size { bytes: 1024 };
        generate(target, &path).unwrap();
        let size = std::fs::metadata(&path).unwrap().len();
        assert!(size >= 1024, "file should be at least target size");
        assert!(size < 1024 + 512, "file shouldn't overshoot too much");
    }

    #[test]
    fn generate_into_works_with_vec() {
        let mut buf: Vec<u8> = Vec::new();
        generate_into(Size { bytes: 512 }, &mut buf).unwrap();
        assert!(buf.len() >= 512);
        let s = String::from_utf8(buf).unwrap();
        for line in s.lines() {
            assert!(serde_json::from_str::<serde_json::Value>(line).is_ok());
        }
    }
}
