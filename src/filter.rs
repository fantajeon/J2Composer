use std::collections::HashMap;
use log::{debug, info};
use serde_json::Value;
use tera::{Error, Result, Tera, Value as TeraValue};

pub fn register_filters(tera: &mut Tera) {
    info!("register builtin-filters");
    tera.register_filter("from_json", from_json_filter);
    tera.register_filter("from_yaml", from_yaml_filter);
    tera.register_filter("from_toml", from_toml_filter);
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
