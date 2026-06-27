use anyhow::{Context, Result};
use fake::Fake;
use rand::Rng;
use serde::Serialize;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

/// Parsed byte size with optional unit (KB, MB, GB).
#[derive(Debug, Clone, Copy)]
pub struct Size {
    bytes: u64,
}

impl FromStr for Size {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
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
            .with_context(|| format!("invalid size number: {num}"))?;
        Ok(Size {
            bytes: n * multiplier,
        })
    }
}

/// Schema for generated records — realistic fake data.
#[derive(Serialize)]
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

impl Record {
    fn generate(id: u64, rng: &mut impl Rng) -> Self {
        let first = fake::faker::name::en::FirstName().fake::<String>();
        let last = fake::faker::name::en::LastName().fake::<String>();
        let domain = fake::faker::internet::en::DomainSuffix().fake::<String>();
        let city = fake::faker::address::en::CityName().fake::<String>();
        let state = fake::faker::address::en::StateAbbr().fake::<String>();
        let zip = fake::faker::address::en::ZipCode().fake::<String>();
        let statuses = ["active", "inactive", "pending", "closed"];
        let year: u32 = rng.random_range(2020..=2026);
        let month: u32 = rng.random_range(1..=12);
        let day: u32 = rng.random_range(1..=28);
        let hour: u32 = rng.random_range(0..=23);
        let minute: u32 = rng.random_range(0..=59);
        let second: u32 = rng.random_range(0..=59);

        Record {
            id,
            name: format!("{first} {last}"),
            email: format!("{first}.{last}@example.{domain}").to_lowercase(),
            city,
            state,
            zip,
            amount: {
                let raw: f64 = rng.random_range(1.0..10000.0);
                (raw * 100.0).round() / 100.0
            },
            status: statuses[rng.random_range(0..statuses.len())].to_string(),
            timestamp: format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"),
        }
    }
}

/// Generate NDJSON data until the target file size is reached.
///
/// Writes one JSON object per line. The output file will be approximately
/// `target` bytes — it may overshoot by up to one record's worth of bytes.
pub fn generate(target: Size, output: &Path) -> Result<()> {
    let mut file = std::fs::File::create(output)
        .with_context(|| format!("cannot create {}", output.display()))?;
    let mut rng = rand::rng();
    let mut written: u64 = 0;
    let mut id: u64 = 1;

    while written < target.bytes {
        let record = Record::generate(id, &mut rng);
        let mut line = serde_json::to_string(&record).with_context(|| "serializing record")?;
        line.push('\n');
        file.write_all(line.as_bytes())
            .with_context(|| "writing record")?;
        written += line.len() as u64;
        id += 1;
    }

    file.flush()?;
    tracing::info!(
        bytes = written,
        records = id - 1,
        path = %output.display(),
        "generation complete"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_bytes() {
        let s: Size = "1024".parse().unwrap();
        assert_eq!(s.bytes, 1024);
    }

    #[test]
    fn parse_size_kb() {
        let s: Size = "5KB".parse().unwrap();
        assert_eq!(s.bytes, 5 * 1024);
    }

    #[test]
    fn parse_size_mb() {
        let s: Size = "10MB".parse().unwrap();
        assert_eq!(s.bytes, 10 * 1_048_576);
    }

    #[test]
    fn parse_size_gb() {
        let s: Size = "2GB".parse().unwrap();
        assert_eq!(s.bytes, 2 * 1_073_741_824);
    }

    #[test]
    fn parse_size_case_insensitive() {
        let s: Size = "10mb".parse().unwrap();
        assert_eq!(s.bytes, 10 * 1_048_576);
    }

    #[test]
    fn parse_size_with_spaces() {
        let s: Size = " 10 MB ".parse().unwrap();
        assert_eq!(s.bytes, 10 * 1_048_576);
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
    fn generate_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.ndjson");
        generate(Size { bytes: 512 }, &path).unwrap();
        assert!(path.exists());
        let contents = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert!(!lines.is_empty());
        // Every line should be valid JSON
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
        // Shouldn't overshoot by more than one record (~200 bytes)
        assert!(size < 1024 + 512, "file shouldn't overshoot too much");
    }
}
