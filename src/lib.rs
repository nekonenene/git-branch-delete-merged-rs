use anyhow::{anyhow, Result};
use std::process::Command;

pub fn exec_command(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program).args(args).output();
    let args_str = args.join(" ");

    match output {
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
