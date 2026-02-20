use crate::core::errors::BBResult;
use crate::util::duration::parse_duration as util_parse_duration;
use chrono::Duration;

pub fn validate_duration(s: &str) -> BBResult<Duration> {
    util_parse_duration(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_duration_valid() {
        assert_eq!(validate_duration("10m").unwrap().num_seconds(), 600);
        assert_eq!(validate_duration("2h").unwrap().num_seconds(), 7200);
        assert_eq!(validate_duration("1d").unwrap().num_seconds(), 86400);
    }

    #[test]
    fn test_validate_duration_invalid() {
        assert!(validate_duration("invalid").is_err());
        assert!(validate_duration("10x").is_err());
        assert!(validate_duration("").is_err());
    }
}
