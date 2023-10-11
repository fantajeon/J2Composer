// src/function.rs
use crate::command::execute_shell_command;
use anyhow::Context as _Context;
use log::{debug, info};
use std::collections::HashMap;
use std::fs;
use tera::{Error, Result, Value as TeraValue};

fn read_file(args: &HashMap<String, TeraValue>) -> Result<TeraValue> {
    debug!("call function(__builtin): read_file: {:?}", args);
    let path = match args.get("file_path") {
        Some(val) => val
            .as_str()
            .with_context(|| "read_file file_path should be a string")
            .map_err(|e| Error::msg(e))?,
        None => return Err(Error::msg("read_file file_path is required")),
    };

    let content = fs::read_to_string(path)
        .with_context(|| format!("read_file: Error reading file: {}", path))
        .map_err(|e| Error::msg(e))?;

    Ok(TeraValue::String(content))
}

fn shell(args: &HashMap<String, TeraValue>) -> Result<TeraValue> {
    let cmd = match args.get("cmd") {
        Some(TeraValue::String(s)) => s,
        _ => return Err(tera::Error::msg("cmd must be provided and be a string")),
    };

    let interpreter = match args.get("interpreter") {
        Some(TeraValue::String(s)) => Some(s.as_str()),
        _ => Some("sh"),
    };

    let mut env: HashMap<String, String> = HashMap::new();
    for (k, v) in args.iter() {
        if k != "cmd" && k != "interpreter" {
            if let TeraValue::String(s) = v {
                env.insert(k.clone(), s.clone());
            }
        }
    }

    match execute_shell_command(cmd, &Some(env), interpreter) {
        Ok(output) => Ok(TeraValue::String(output)),
        Err(e) => Err(e),
    }
}

pub fn register_functions(tera: &mut tera::Tera) {
    info!("register builtin-functions");
    tera.register_function("read_file", read_file);
    tera.register_function("shell", shell);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tera::Value as TeraValue;

    #[test]
    fn test_shell() {
        let mut args = HashMap::new();
        args.insert(
            "cmd".to_string(),
            TeraValue::String("echo hello".to_string()),
        );

        let result = shell(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TeraValue::String("hello\n".to_string()));

        args.insert(
            "interpreter".to_string(),
            TeraValue::String("/bin/bash".to_string()),
        );

        let result = shell(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TeraValue::String("hello\n".to_string()));
    }
}
