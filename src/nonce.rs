/// Utility functions for the library
use uuid::Uuid;

/// Generate a cryptographically secure nonce for IPC requests
///
/// This function uses UUID v4 to generate a unique identifier,
/// which is cryptographically random and globally unique.
///
/// # Arguments
///
/// * `prefix` - A prefix to identify the type of operation (e.g., "set-activity", "clear-activity")
///
/// # Returns
///
/// A string in the format `{prefix}-{uuid}`
///
/// # Examples
///
/// ```
/// # use presenceforge::nonce::generate_nonce;
/// let nonce = generate_nonce("set-activity");
/// assert!(nonce.starts_with("set-activity-"));
/// ```
///
/// # Security
///
/// UUID v4 provides 122 bits of randomness, making collisions extremely unlikely
/// (probability of collision is approximately 1 in 2^61 after generating 1 billion UUIDs).
/// This is far superior to timestamp-based nonces which can collide during rapid operations.
pub fn generate_nonce(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_nonce_format() {
        let nonce = generate_nonce("test");
        assert!(nonce.starts_with("test-"));
        assert!(nonce.len() > 5); // prefix + dash + uuid
    }

    #[test]
    fn test_generate_nonce_uniqueness() {
        let nonce1 = generate_nonce("test");
        let nonce2 = generate_nonce("test");
        assert_ne!(nonce1, nonce2, "Nonces should be unique");
    }

    #[test]
    fn test_generate_nonce_different_prefixes() {
        let nonce1 = generate_nonce("set-activity");
        let nonce2 = generate_nonce("clear-activity");

        assert!(nonce1.starts_with("set-activity-"));
        assert!(nonce2.starts_with("clear-activity-"));
        assert_ne!(nonce1, nonce2);
    }
}
