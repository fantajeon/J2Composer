extern crate jintemplify_plugin;
mod ast;
mod command;
pub mod function;
pub mod plugin;
pub mod wasm_plugin;
pub use ast::{ExecutableFunction, FilterDeclaration, FunctionDeclaration};
mod render;
mod shell_plugin;
