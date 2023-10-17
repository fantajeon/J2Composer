use crate::ast::{Executable, Param};
use crate::command::{execute_shell_command, replace_placeholder, tera_to_str};
use std::collections::HashMap;

pub struct ShellCommand {
    pub script: String,
    pub params: Option<Vec<Param>>,
    pub env: Option<HashMap<String, String>>,
}

pub struct ShellFunction {
    pub command: ShellCommand,
}

pub struct ShellFilter {
    pub command: ShellCommand,
}

fn prepare_command(
    script: &str,
    params: &Option<Vec<Param>>,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<String> {
    let mut cmd = script.clone().to_string();
    if let Some(parameters) = params {
        for param in parameters {
            replace_placeholder(&mut cmd, param, args)?;
        }
    }
    Ok(cmd)
}

impl Executable for ShellFunction {
    fn execute(
        &self,
        args: &HashMap<String, tera::Value>,
        _value: Option<&tera::Value>,
    ) -> tera::Result<tera::Value> {
        let cmd = prepare_command(&self.command.script, &self.command.params, args)?;
        println!("==> ShellFunction: {}", cmd);
        Ok(tera::Value::String(execute_shell_command(
            &cmd,
            &self.command.env,
            None,
        )?))
    }
}

pub fn prepare_command_filter(
    script: &str,
    params: &Option<Vec<Param>>,
    value: &tera::Value,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<String> {
    let mut cmd = script.clone().to_string();
    if let Some(parameters) = params {
        for param in parameters {
            println!("prepare_command_filter: check={}", param.name);
            // input은 value로 직접 처리되므로 이를 건너뛰기
            if param.name != "input" {
                replace_placeholder(&mut cmd, param, args)?;
            }
        }
    }

    cmd = cmd.replace("$(input)", &format!("\"{}\"", tera_to_str(value)?));
    Ok(cmd)
}

impl Executable for ShellFilter {
    fn execute(
        &self,
        args: &HashMap<String, tera::Value>,
        value: Option<&tera::Value>,
    ) -> tera::Result<tera::Value> {
        let cmd = prepare_command_filter(
            &self.command.script,
            &self.command.params,
            value.unwrap(),
            args,
        )?;
        println!("===> ShellFilter: {}", cmd);
        Ok(tera::Value::String(execute_shell_command(
            &cmd,
            &self.command.env,
            None,
        )?))
    }
}
