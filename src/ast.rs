use serde::Deserialize;
use std::collections::HashMap;
use tera;

pub trait Executable: Sync + Send {
    fn execute(
        &self,
        args: &HashMap<String, tera::Value>,
        value: Option<&tera::Value>,
    ) -> tera::Result<tera::Value>;
}

#[derive(Debug, Deserialize, Clone)]
pub struct Param {
    pub name: String,
    pub description: Option<String>,
    pub default: Option<String>,
}
