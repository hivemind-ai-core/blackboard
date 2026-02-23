use crate::core::errors::{BBError, BBResult};
use crate::core::models::reference::Reference;
use serde_json::Value as JsonValue;

pub fn parse_ref(s: &str) -> BBResult<Reference> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return Err(BBError::InvalidRefFormat(s.to_string()));
    }

    let where_ = parts[0].trim();
    let what = parts[1].trim();
    let ref_str = parts[2].trim();

    if where_.is_empty() || what.is_empty() || ref_str.is_empty() {
        return Err(BBError::InvalidRefFormat(s.to_string()));
    }

    let ref_value = if ref_str.chars().all(|c| c.is_ascii_digit()) {
        JsonValue::Number(ref_str.parse::<i64>().unwrap().into())
    } else {
        JsonValue::String(ref_str.to_string())
    };

    Ok(Reference {
        where_: where_.to_string(),
        what: what.to_string(),
        ref_: ref_value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ref_numeric() {
        let r = parse_ref("tt:task:13").unwrap();
        assert_eq!(r.where_, "tt");
        assert_eq!(r.what, "task");
        assert_eq!(r.ref_, json!(13));
    }

    #[test]
    fn test_parse_ref_string() {
        let r = parse_ref("github:issue:abc-123").unwrap();
        assert_eq!(r.where_, "github");
        assert_eq!(r.what, "issue");
        assert_eq!(r.ref_, json!("abc-123"));
    }

    #[test]
    fn test_parse_ref_internal() {
        let r = parse_ref("bb:message:42").unwrap();
        assert_eq!(r.where_, "bb");
        assert_eq!(r.what, "message");
        assert_eq!(r.ref_, json!(42));
    }

    #[test]
    fn test_parse_ref_invalid_format() {
        assert!(parse_ref("invalid").is_err());
        assert!(parse_ref("only:two").is_err());
        assert!(parse_ref("too:many:parts:here").is_err());
    }

    #[test]
    fn test_parse_ref_empty_parts() {
        assert!(parse_ref(":what:ref").is_err());
        assert!(parse_ref("where::ref").is_err());
        assert!(parse_ref("where:what:").is_err());
    }

    #[test]
    fn test_parse_ref_path_with_colons() {
        // This tests that refs with colons in the path part are handled
        // Actually, we split by all colons, so this is 4 parts and should fail
        assert!(parse_ref("bb:artifact:src/file.rs:line10").is_err());
    }

    #[test]
    fn test_parse_ref_with_whitespace() {
        let r = parse_ref("  tt : task : 13  ").unwrap();
        assert_eq!(r.where_, "tt");
        assert_eq!(r.what, "task");
        assert_eq!(r.ref_, json!(13));
    }
}
