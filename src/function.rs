use tera::{Function, Result, Value as TeraValue, Error};
use std::fs;
use std::collections::HashMap;
use anyhow::{Context as _Context};
use log::{info, debug};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BuiltinFunction {
    pub name: String,
}

impl Function for BuiltinFunction {
    fn call(&self, args: &HashMap<String, TeraValue>) -> Result<TeraValue> {
        debug!("call function(__builtin): read_file: {:?}", args);
        let path = match args.get("file_path") {
            Some(val) => val.as_str().with_context(|| format!("{}, file_path should be a string", self.name)).map_err(|e| Error::msg(e))?,
            None => return Err(Error::msg(format!("{}, file_path is required", self.name))),
        };

        let content = fs::read_to_string(path).with_context(|| format!("Error reading file: {}", path)).map_err(|e| Error::msg(e))?;

        Ok(TeraValue::String(content))
    }
}

pub fn register_functions(tera: &mut tera::Tera) {
    info!("register builtin-functions");
    tera.register_function("read_file", BuiltinFunction{name: "read_file".to_string()});
}
