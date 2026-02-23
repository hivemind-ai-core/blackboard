use crate::core::errors::{BBError, BBResult};
use chrono::Duration;

pub fn parse_duration(s: &str) -> BBResult<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Err(BBError::InvalidInput("empty duration".to_string()));
    }

    let num_part = &s[..s.len() - 1];
    let unit = s.chars().last().unwrap();

    let num: u64 = num_part
        .parse()
        .map_err(|_| BBError::InvalidInput(format!("invalid duration number: {num_part}")))?;

    let seconds: i64 = match unit {
        's' => num as i64,
        'm' => num as i64 * 60,
        'h' => num as i64 * 3600,
        'd' => num as i64 * 86400,
        'w' => num as i64 * 604800,
        _ => {
            return Err(BBError::InvalidInput(format!(
                "invalid duration unit: {unit} (expected s, m, h, d, w)"
            )));
        }
    };

    Ok(Duration::seconds(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(parse_duration("30s").unwrap().num_seconds(), 30);
        assert_eq!(parse_duration("1s").unwrap().num_seconds(), 1);
    }

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration("10m").unwrap().num_seconds(), 600);
        assert_eq!(parse_duration("1m").unwrap().num_seconds(), 60);
    }

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(parse_duration("2h").unwrap().num_seconds(), 7200);
        assert_eq!(parse_duration("1h").unwrap().num_seconds(), 3600);
    }

    #[test]
    fn test_parse_duration_days() {
        assert_eq!(parse_duration("1d").unwrap().num_seconds(), 86400);
        assert_eq!(parse_duration("2d").unwrap().num_seconds(), 172800);
    }

    #[test]
    fn test_parse_duration_weeks() {
        assert_eq!(parse_duration("1w").unwrap().num_seconds(), 604800);
        assert_eq!(parse_duration("2w").unwrap().num_seconds(), 1209600);
    }

    #[test]
    fn test_parse_duration_whitespace() {
        assert_eq!(parse_duration("  10m  ").unwrap().num_seconds(), 600);
    }

    #[test]
    fn test_parse_duration_invalid_unit() {
        assert!(parse_duration("10x").is_err());
        assert!(parse_duration("10y").is_err());
    }

    #[test]
    fn test_parse_duration_invalid_number() {
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("xm").is_err());
    }

    #[test]
    fn test_parse_duration_empty() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn test_parse_duration_no_unit() {
        assert!(parse_duration("10").is_err());
    }

    #[test]
    fn test_parse_duration_only_unit() {
        assert!(parse_duration("m").is_err());
    }
}
