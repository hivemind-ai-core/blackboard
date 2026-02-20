use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub where_: String,
    pub what: String,
    pub ref_: JsonValue,
}

// Custom Serialize to output "where" instead of "where_"
impl Serialize for Reference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Reference", 3)?;
        state.serialize_field("where", &self.where_)?;
        state.serialize_field("what", &self.what)?;
        state.serialize_field("ref", &self.ref_)?;
        state.end()
    }
}

// Custom Deserialize to handle "where" field
impl<'de> Deserialize<'de> for Reference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ReferenceHelper {
            #[serde(rename = "where")]
            where_: String,
            what: String,
            #[serde(rename = "ref")]
            ref_: JsonValue,
        }

        let helper = ReferenceHelper::deserialize(deserializer)?;
        Ok(Reference {
            where_: helper.where_,
            what: helper.what,
            ref_: helper.ref_,
        })
    }
}

impl Reference {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_reference_creation() {
        let r = Reference {
            where_: "tt".to_string(),
            what: "task".to_string(),
            ref_: json!(13),
        };
        assert_eq!(r.where_, "tt");
        assert_eq!(r.what, "task");
        assert_eq!(r.ref_, json!(13));
    }

    #[test]
    fn test_reference_serialization() {
        let r = Reference {
            where_: "tt".to_string(),
            what: "task".to_string(),
            ref_: json!(13),
        };
        let json_str = serde_json::to_string(&r).unwrap();
        assert!(json_str.contains("\"where\":\"tt\""));
        assert!(json_str.contains("\"what\":\"task\""));
        assert!(json_str.contains("\"ref\":13"));
    }

    #[test]
    fn test_reference_deserialization() {
        let json_str = r#"{"where":"tt","what":"task","ref":13}"#;
        let r: Reference = serde_json::from_str(json_str).unwrap();
        assert_eq!(r.where_, "tt");
        assert_eq!(r.what, "task");
        assert_eq!(r.ref_, json!(13));
    }

    #[test]
    fn test_reference_with_string_ref() {
        let r = Reference {
            where_: "github".to_string(),
            what: "issue".to_string(),
            ref_: json!("abc-123"),
        };
        assert_eq!(r.ref_, json!("abc-123"));
    }

    #[test]
    fn test_reference_clone() {
        let r1 = Reference {
            where_: "bb".to_string(),
            what: "message".to_string(),
            ref_: json!(42),
        };
        let r2 = r1.clone();
        assert_eq!(r1, r2);
    }
}
