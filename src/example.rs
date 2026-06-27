use anyhow::Result;

/// Greet someone by name.
pub fn greet(name: &str) -> Result<String> {
    tracing::info!(name, "greeting");
    Ok(format!("Hello, {name}!"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_default() {
        assert_eq!(greet("world").unwrap(), "Hello, world!");
    }

    #[test]
    fn greet_custom() {
        assert_eq!(greet("Rust").unwrap(), "Hello, Rust!");
    }

    #[test]
    fn greet_empty() {
        assert_eq!(greet("").unwrap(), "Hello, !");
    }
}
