use anyhow::{Context, Result};
use fake::Fake;
use openapiv3::{OpenAPI, RefOr, Schema, SchemaKind, Type};
use rand::Rng;
use serde::Serialize;
use serde_json::{Map, Value};
use std::fs;
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

/// Generate NDJSON from an OpenAPI schema under `components/schemas`.
pub fn generate_from_openapi(target: Size, output: &Path, spec: &Path, schema_name: &str) -> Result<()> {
    let api = parse_openapi(spec)?;
    let schema = root_schema(&api, schema_name)?;

    let mut file = std::fs::File::create(output)
        .with_context(|| format!("cannot create {}", output.display()))?;
    let mut rng = rand::rng();
    let mut written: u64 = 0;
    let mut records: u64 = 0;

    while written < target.bytes {
        let value = generate_value_from_schema(&api, schema, &mut rng, 0)?;
        let mut line = serde_json::to_string(&value).with_context(|| "serializing OpenAPI record")?;
        line.push('\n');
        file.write_all(line.as_bytes())
            .with_context(|| "writing OpenAPI record")?;
        written += line.len() as u64;
        records += 1;
    }

    file.flush()?;
    tracing::info!(
        bytes = written,
        records,
        schema = schema_name,
        spec = %spec.display(),
        path = %output.display(),
        "OpenAPI generation complete"
    );

    Ok(())
}

fn parse_openapi(spec: &Path) -> Result<OpenAPI> {
    let text = fs::read_to_string(spec)
        .with_context(|| format!("cannot read spec {}", spec.display()))?;

    let parsed_json = serde_json::from_str::<OpenAPI>(&text);
    if let Ok(api) = parsed_json {
        return Ok(api);
    }

    serde_yaml::from_str::<OpenAPI>(&text)
        .with_context(|| format!("spec {} is not valid OpenAPI JSON or YAML", spec.display()))
}

fn root_schema<'a>(api: &'a OpenAPI, schema_name: &str) -> Result<&'a Schema> {
    let components = api
        .components
        .as_ref()
        .context("spec does not include components")?;

    let schema_ref = components
        .schemas
        .get(schema_name)
        .with_context(|| format!("schema '{}' was not found in components/schemas", schema_name))?;

    resolve_schema_ref(api, schema_ref)
}

fn resolve_schema_ref<'a>(api: &'a OpenAPI, schema_ref: &'a RefOr<Schema>) -> Result<&'a Schema> {
    match schema_ref {
        RefOr::Item(schema) => Ok(schema),
        RefOr::Reference { reference } => resolve_schema_path(api, reference),
    }
}

fn resolve_boxed_schema_ref<'a>(api: &'a OpenAPI, schema_ref: &'a RefOr<Box<Schema>>) -> Result<&'a Schema> {
    match schema_ref {
        RefOr::Item(schema) => Ok(schema.as_ref()),
        RefOr::Reference { reference } => resolve_schema_path(api, reference),
    }
}

fn resolve_schema_path<'a>(api: &'a OpenAPI, reference: &str) -> Result<&'a Schema> {
    let name = reference
        .strip_prefix("#/components/schemas/")
        .with_context(|| format!("unsupported schema reference: {reference}"))?;

    let components = api
        .components
        .as_ref()
        .context("spec does not include components")?;
    let schema_ref = components
        .schemas
        .get(name)
        .with_context(|| format!("referenced schema '{}' was not found", name))?;

    resolve_schema_ref(api, schema_ref)
}

fn generate_value_from_schema(
    api: &OpenAPI,
    schema: &Schema,
    rng: &mut impl Rng,
    depth: usize,
) -> Result<Value> {
    if depth > 10 {
        return Ok(Value::Null);
    }

    let mut value = match &schema.schema_kind {
        SchemaKind::Type(Type::String(string_type)) => {
            let variants: Vec<&String> = string_type
                .enumeration
                .iter()
                .filter_map(|v| v.as_ref())
                .collect();
            if !variants.is_empty() {
                Value::String(variants[rng.random_range(0..variants.len())].clone())
            } else {
                let first = fake::faker::name::en::FirstName().fake::<String>();
                let last = fake::faker::name::en::LastName().fake::<String>();
                let domain = fake::faker::internet::en::DomainSuffix().fake::<String>();
                match format!("{:?}", string_type.format).as_str() {
                    "Email" => Value::String(format!("{first}.{last}@example.{domain}").to_lowercase()),
                    "Date" => {
                        let year: u32 = rng.random_range(2020..=2026);
                        let month: u32 = rng.random_range(1..=12);
                        let day: u32 = rng.random_range(1..=28);
                        Value::String(format!("{year:04}-{month:02}-{day:02}"))
                    }
                    "DateTime" => Value::String(random_timestamp(rng)),
                    _ => Value::String(fake::faker::lorem::en::Sentence(3..6).fake::<String>()),
                }
            }
        }
        SchemaKind::Type(Type::Number(number_type)) => {
            let min = number_type.minimum.unwrap_or(0.0);
            let mut max = number_type.maximum.unwrap_or(10_000.0);
            if max <= min {
                max = min + 1.0;
            }
            Value::from(rng.random_range(min..=max))
        }
        SchemaKind::Type(Type::Integer(integer_type)) => {
            let min = integer_type.minimum.unwrap_or(0);
            let mut max = integer_type.maximum.unwrap_or(10_000);
            if max <= min {
                max = min + 1;
            }
            Value::from(rng.random_range(min..=max))
        }
        SchemaKind::Type(Type::Boolean {}) => Value::Bool(rng.random()),
        SchemaKind::Type(Type::Array(array_type)) => {
            let min_items = array_type.min_items.unwrap_or(1).max(1);
            let max_items = array_type.max_items.unwrap_or((min_items + 2).min(5)).max(min_items);
            let len = rng.random_range(min_items..=max_items);
            let mut values = Vec::with_capacity(len);
            if let Some(item_ref) = &array_type.items {
                let item_schema = resolve_boxed_schema_ref(api, item_ref)?;
                for _ in 0..len {
                    values.push(generate_value_from_schema(api, item_schema, rng, depth + 1)?);
                }
            }
            Value::Array(values)
        }
        SchemaKind::Type(Type::Object(object_type)) => {
            let mut map = Map::new();
            for (name, prop_ref) in &object_type.properties {
                let prop_schema = resolve_boxed_schema_ref(api, prop_ref)?;
                map.insert(
                    name.clone(),
                    generate_value_from_schema(api, prop_schema, rng, depth + 1)?,
                );
            }
            Value::Object(map)
        }
        SchemaKind::OneOf { one_of } => {
            if one_of.is_empty() {
                Value::Null
            } else {
                let selected = &one_of[rng.random_range(0..one_of.len())];
                let selected_schema = resolve_schema_ref(api, selected)?;
                generate_value_from_schema(api, selected_schema, rng, depth + 1)?
            }
        }
        SchemaKind::AnyOf { any_of } => {
            if any_of.is_empty() {
                Value::Null
            } else {
                let selected = &any_of[rng.random_range(0..any_of.len())];
                let selected_schema = resolve_schema_ref(api, selected)?;
                generate_value_from_schema(api, selected_schema, rng, depth + 1)?
            }
        }
        SchemaKind::AllOf { all_of } => {
            let mut merged = Map::new();
            for schema_ref in all_of {
                let child = resolve_schema_ref(api, schema_ref)?;
                let value = generate_value_from_schema(api, child, rng, depth + 1)?;
                if let Value::Object(obj) = value {
                    for (k, v) in obj {
                        merged.insert(k, v);
                    }
                }
            }
            Value::Object(merged)
        }
        _ => Value::Null,
    };

    if value.is_null() {
        if let Some(default) = &schema.schema_data.default {
            value = default.clone();
        } else if let Some(example) = &schema.schema_data.example {
            value = example.clone();
        }
    }

    if schema.schema_data.nullable && rng.random_ratio(1, 20) {
        Ok(Value::Null)
    } else {
        Ok(value)
    }
}

fn random_timestamp(rng: &mut impl Rng) -> String {
    let year: u32 = rng.random_range(2020..=2026);
    let month: u32 = rng.random_range(1..=12);
    let day: u32 = rng.random_range(1..=28);
    let hour: u32 = rng.random_range(0..=23);
    let minute: u32 = rng.random_range(0..=59);
    let second: u32 = rng.random_range(0..=59);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
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
