use log::debug;
use std::collections::HashMap;
use std::process::Command;

pub fn execute_shell_command(
    cmd: &str,
    env: &Option<HashMap<String, String>>,
    interpreter: Option<&str>,
) -> tera::Result<String> {
    debug!("shell command: {}, env: {:?}", cmd, env);
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
    let interpreter = extract_interpreter(cmd);
    let mut command = run_with_interpreter(interpreter, cmd, env_vars)?;

    command.spawn()?.wait()?;
    Ok(command)
}

pub fn extract_interpreter(cmd: &str) -> &str {
    let lines: Vec<&str> = cmd.split('\n').collect();
    if lines[0].starts_with("#!") {
        &lines[0][2..]
    } else {
        "sh"
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
