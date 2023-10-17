use crate::ast::Param;
use log::debug;
use std::collections::HashMap;
use std::process::Command;

pub fn tera_to_str(value: &tera::Value) -> tera::Result<String> {
    match value {
        tera::Value::Null => Ok("null".to_string()),
        tera::Value::Bool(b) => Ok(b.to_string()),
        tera::Value::Number(n) => Ok(n.to_string()),
        tera::Value::String(s) => Ok(s.clone()),
        tera::Value::Array(arr) => Ok(serde_json::to_string(arr)?),
        tera::Value::Object(obj) => Ok(serde_json::to_string(obj)?),
    }
}

#[allow(dead_code)]
pub fn replace_placeholder(
    cmd: &mut String,
    param: &Param,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<()> {
    if !args.contains_key(&param.name) && param.default.is_none() {
        return Err(tera::Error::msg(format!(
            "Parameter '{}' not provided and no default value is set.",
            param.name
        )));
    }
    let placeholder = format!("$({})", param.name);
    let value = args.get(&param.name).unwrap();

    let value_str = tera_to_str(value)?;
    debug!("placeholder: {} => {}", placeholder, value_str);
    *cmd = cmd.replace(&placeholder, &value_str);
    Ok(())
}

pub fn execute_shell_command(
    cmd: &str,
    env: &Option<HashMap<String, String>>,
    interpreter: Option<&str>,
) -> tera::Result<String> {
    debug!(
        "interpreter: {:?}, shell command: {}, env: {:?}",
        interpreter, cmd, env
    );
    let mut shell_cmd = if let Some(interpreter) = interpreter {
        run_with_interpreter(interpreter, cmd, env.as_ref())
    } else {
        run_with_shebang(cmd, env.as_ref())
    }
    .map_err(|e| tera::Error::msg(format!("Failed to execute command '{}': {}", cmd, e)))?;

    let output = shell_cmd.output();

    match &output {
        Ok(o) if o.status.success() => {
            let output_str = String::from_utf8_lossy(&o.stdout).into_owned();
            debug!("shell command: {} => output_str: {}", cmd, output_str);
            Ok(output_str)
        }
        Ok(o) => Err(tera::Error::msg(format!(
            "Failed to execute command '{}': {}",
            cmd,
            String::from_utf8_lossy(&o.stderr)
        ))),
        Err(e) => Err(tera::Error::msg(format!(
            "Failed to execute command '{}': {}",
            cmd, e
        ))),
    }
}

fn run_with_shebang(
    cmd: &str,
    env_vars: Option<&HashMap<String, String>>,
) -> Result<Command, std::io::Error> {
    let (interpreter, actual_cmd) = extract_interpreter(cmd);
    let mut command = run_with_interpreter(&interpreter, &actual_cmd, env_vars)?;
    debug!("run with shebang: {}, cmd={}", interpreter, actual_cmd);

    command.spawn()?.wait()?;
    Ok(command)
}

pub fn extract_interpreter(cmd: &str) -> (String, String) {
    let lines: Vec<&str> = cmd.split('\n').collect();
    if lines[0].starts_with("#!") {
        (lines[0][2..].to_string(), lines[1..].join("\n"))
    } else {
        ("sh".to_string(), cmd.to_string())
    }
}

pub fn run_with_interpreter(
    interpreter: &str,
    cmd: &str,
    env_vars: Option<&HashMap<String, String>>,
) -> Result<Command, std::io::Error> {
    let mut command = Command::new(interpreter);
    command.arg("-c").arg(cmd);

    if let Some(envs) = env_vars {
        for (key, value) in envs.iter() {
            command.env(key, value);
        }
    }
    Ok(command)
}
