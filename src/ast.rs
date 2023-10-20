use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tera;

pub trait Executable: Sync + Send {
    fn execute(
        &self,
        args: &HashMap<String, tera::Value>,
        value: Option<&tera::Value>,
    ) -> tera::Result<tera::Value>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wasm {
    pub path: String,
    pub import: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Param {
    pub name: String,
    pub description: Option<String>,
    pub default: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Option<Vec<Param>>,
    pub env: Option<HashMap<String, String>>,
    pub description: Option<String>,
    pub wasm: Option<Wasm>,
    pub script: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FilterDeclaration {
    pub name: String,
    pub params: Option<Vec<Param>>,
    pub env: Option<HashMap<String, String>>,
    pub description: Option<String>,
    pub wasm: Option<Wasm>,
    pub script: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmDeclartion {
    pub wasm: Wasm,
    pub params: Option<Vec<Param>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmFunction {
    pub decl: WasmDeclartion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmFilter {
    pub decl: WasmDeclartion,
}

pub struct ExecutableFunction {
    pub executor: Box<dyn Executable>,
    pub name: String,
}
