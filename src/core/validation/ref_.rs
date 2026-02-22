//! Reference validation wrapper
//! 
//! Provides a consistent interface for ref validation with BBResult error handling.

use crate::core::errors::BBResult;
use crate::core::models::reference::Reference;
use crate::util::ref_::parse_ref;

/// Validates a reference string and returns a Reference object.
/// 
/// This is a wrapper around `util::ref_::parse_ref` that provides
/// consistent BBResult error handling for the validation module.
/// 
/// # Arguments
/// * `s` - The reference string in format "where:what:ref"
/// 
/// # Returns
/// * `Ok(Reference)` - The parsed reference
/// * `Err(BBError::InvalidRefFormat)` - If the format is invalid
/// 
/// # Examples
/// ```
/// use bb::core::validation::ref_::validate_ref;
/// 
/// let reference = validate_ref("tt:task:13").unwrap();
/// assert_eq!(reference.where_, "tt");
/// assert_eq!(reference.what, "task");
/// ```
#[allow(dead_code)]
pub fn validate_ref(s: &str) -> BBResult<Reference> {
    parse_ref(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_ref_numeric() {
        let r = validate_ref("tt:task:13").unwrap();
        assert_eq!(r.where_, "tt");
        assert_eq!(r.what, "task");
        assert_eq!(r.ref_, json!(13));
    }

    #[test]
    fn test_validate_ref_string() {
        let r = validate_ref("github:issue:abc-123").unwrap();
        assert_eq!(r.where_, "github");
        assert_eq!(r.what, "issue");
        assert_eq!(r.ref_, json!("abc-123"));
    }

    #[test]
    fn test_validate_ref_invalid_format() {
        assert!(validate_ref("invalid").is_err());
        assert!(validate_ref("only:two").is_err());
        assert!(validate_ref("too:many:parts:here").is_err());
    }

    #[test]
    fn test_validate_ref_empty_parts() {
        assert!(validate_ref(":what:ref").is_err());
        assert!(validate_ref("where::ref").is_err());
        assert!(validate_ref("where:what:").is_err());
    }

    #[test]
    fn test_validate_ref_error_type() {
        let result = validate_ref("invalid");
        assert!(result.is_err());
        // Verify it returns InvalidRefFormat error
        match result {
            Err(crate::core::errors::BBError::InvalidRefFormat(_)) => {},
            _ => panic!("Expected InvalidRefFormat error"),
        }
    }
}
