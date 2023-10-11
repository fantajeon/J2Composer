use log::{debug, info};
use serde_json::Value;
use std::collections::HashMap;
use tera::{Error, Result, Tera, Value as TeraValue};

pub fn register_filters(tera: &mut Tera) {
    info!("register builtin-filters");
    tera.register_filter("to_object", to_object);
    tera.register_filter("from_json", from_json_filter);
    tera.register_filter("from_yaml", from_yaml_filter);
    tera.register_filter("from_toml", from_toml_filter);
}

fn to_object(value: &TeraValue, _args: &HashMap<String, TeraValue>) -> Result<TeraValue> {
    debug!("call to_object");
    match value {
        TeraValue::Array(arr) => {
            if arr.len() % 2 != 0 {
                return Err(Error::msg(format!(
                    "The array has {} elements which is odd. It should have an even number of elements for key-value pairing. Example: [key1, val1, key2, val2, ...]",
                    arr.len()
                )));
            }

            let mut map = tera::Map::new();
            for i in (0..arr.len()).step_by(2) {
                if let (TeraValue::String(key), val) = (&arr[i], &arr[i + 1]) {
                    map.insert(key.clone(), val.clone());
                } else {
                    return Err(Error::msg("Expected a string key in the array"));
                }
            }

            Ok(TeraValue::Object(map))
        }
        _ => Err(Error::msg(
            "Value must be an array to be parsed into an object",
        )),
    }
}

fn from_json_filter(value: &TeraValue, _args: &HashMap<String, TeraValue>) -> Result<TeraValue> {
    debug!("call from_json_filter");
    match value {
        TeraValue::String(s) => {
            let parsed: Value = serde_json::from_str(s)
                .map_err(|e| Error::msg(format!("Failed to parse JSON string: {}", e)))?;
            Ok(TeraValue::Object(parsed.as_object().unwrap().clone()))
        }
        _ => Err(Error::msg("Value must be a string to be parsed as JSON")),
    }
}

pub fn from_yaml_filter(value: &Value, _: &HashMap<String, Value>) -> Result<TeraValue> {
    debug!("call from_yaml_filter");
    match value {
        Value::String(s) => match serde_yaml::from_str::<serde_json::Value>(s) {
            Ok(parsed) => Ok(Value::Object(parsed.as_object().unwrap().clone())),
            Err(e) => Err(Error::msg(format!("Failed to parse YAML: {}", e))),
        },
        _ => Err(Error::msg("Value provided is not a string")),
    }
}

pub fn from_toml_filter(value: &Value, _: &HashMap<String, Value>) -> Result<TeraValue> {
    debug!("call from_toml_filter");
    match value {
        Value::String(s) => match toml::from_str::<serde_json::Value>(s) {
            Ok(parsed) => Ok(Value::Object(parsed.as_object().unwrap().clone())),
            Err(e) => Err(Error::msg(format!("Failed to parse TOML: {}", e))),
        },
        _ => Err(Error::msg("Value provided is not a string")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::Value as TeraValue;

    #[test]
    fn test_from_json_filter_valid() {
        let json_str = TeraValue::String(r#"{"key": "value"}"#.to_string());
        let result = from_json_filter(&json_str, &HashMap::new()).unwrap();
        match result {
            TeraValue::Object(map) => {
                assert_eq!(map["key"], TeraValue::String("value".to_string()))
            }
            _ => panic!("Expected an Object!"),
        }
    }

    #[test]
    fn test_from_json_filter_invalid() {
        let json_str = TeraValue::String(r#"{"key": "value""#.to_string());
        assert!(from_json_filter(&json_str, &HashMap::new()).is_err());
    }

    #[test]
    fn test_from_yaml_filter_valid() {
        let yaml_str = TeraValue::String("key: value\n".to_string());
        let result = from_yaml_filter(&yaml_str, &HashMap::new()).unwrap();
        match result {
            TeraValue::Object(map) => {
                assert_eq!(map["key"], TeraValue::String("value".to_string()))
            }
            _ => panic!("Expected an Object!"),
        }
    }

    #[test]
    fn test_from_yaml_filter_invalid() {
        let yaml_str = TeraValue::String("key: value:".to_string());
        assert!(from_yaml_filter(&yaml_str, &HashMap::new()).is_err());
    }

    #[test]
    fn test_from_toml_filter_valid() {
        let toml_str = TeraValue::String("key = \"value\"\n".to_string());
        let result = from_toml_filter(&toml_str, &HashMap::new()).unwrap();
        match result {
            TeraValue::Object(map) => {
                assert_eq!(map["key"], TeraValue::String("value".to_string()))
            }
            _ => panic!("Expected an Object!"),
        }
    }

    #[test]
    fn test_from_toml_filter_invalid() {
        let toml_str = TeraValue::String("key = value\n".to_string());
        assert!(from_toml_filter(&toml_str, &HashMap::new()).is_err());
    }

    #[test]
    fn test_register_filters() {
        // Tera 인스턴스를 생성하고 필터를 등록
        let mut tera = Tera::default();
        let context = tera::Context::new();
        register_filters(&mut tera);

        // JSON 필터 테스트
        let template = r#"{% set t = '{"key": "value"}' | from_json %}{{t.key}}"#;
        let rendered_output = tera.render_str(template, &context).unwrap();
        assert_eq!(rendered_output, "value");

        // YAML 필터 테스트
        let rendered = tera
            .render_str(
                r#"{% set t = "key: value" | from_yaml %}{{t.key}}"#,
                &context,
            )
            .unwrap();
        assert_eq!(rendered, "value", "from_yaml");

        // TOML 필터 테스트
        let rendered = tera
            .render_str(
                r#"{% set t = "key = 'value'" | from_toml %}{{t.key}}"#,
                &context,
            )
            .unwrap();
        assert_eq!(rendered, "value", "form_toml");
    }
}
