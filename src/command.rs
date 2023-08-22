use ansi_term::Colour::Yellow;
use anyhow::{anyhow, Result};
use std::process::Command;

/// Exec command, and return stdout string
///
/// # Arguments
/// * `program` - Command name
/// * `args` - Arguments
pub fn exec_command(program: &str, args: &[&str]) -> Result<String> {
    let args_str = args.join(" ");

    let result = Command::new(program).args(args).output();

    match result {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)?;
                let trimmed_stdout = stdout.trim_end_matches('\n').to_string();
                return Ok(trimmed_stdout);
            } else {
                let stderr = String::from_utf8(output.stderr)?;
                return Err(anyhow!("\"{} {}\" received {}\n\n{}", program, args_str, output.status, stderr));
            }
        }
        Err(err) => {
            return Err(anyhow!("\"{} {}\" failed:\n{}", program, args_str, err));
        }
    }
}

/// Exec command in a child process
///
/// # Arguments
/// * `program` - Command name
/// * `args` - Arguments
pub fn spawn_command(program: &str, args: &[&str]) -> Result<()> {
    let args_str = args.join(" ");

    let result = Command::new(program).args(args).spawn();
    if result.is_err() {
        return Err(anyhow!(result.unwrap_err()));
    }

    let mut child = result.unwrap();
    let result = child.wait();
    if result.is_err() {
        return Err(anyhow!(result.unwrap_err()));
    }

    let exit_status = result.unwrap();
    if !exit_status.success() {
        println!("{}", Yellow.paint(format!("[WARN] \"{} {}\" received {}", program, args_str, exit_status)));
    }

    Ok(())
}
