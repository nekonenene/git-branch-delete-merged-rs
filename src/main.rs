use clap::Parser;
use anyhow::{anyhow, Result};
use std::process::Command;

#[derive(Parser)]
struct Args {
    #[arg(required = true, index = 1, help = "Base branch name (e.g. main, develop)")]
    base_branch: String,
}

fn exec_command(program: &str, args: &[&str]) -> Result<String> {
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
                return Err(anyhow!("\"{} {}\" received {}\n{}", program, args_str, output.status, stderr));
            }
        }
        Err(err) => {
            return Err(anyhow!("\"{} {}\" failed:\n{}", program, args_str, err));
        }
    }
}

fn main() -> Result<()> {
    let output = Command::new("git").arg("version").output();

    if output.is_err() {
        eprintln!("Command not found: git {}", output.unwrap_err());
        std::process::exit(1);
    }

    let output = exec_command("git", &["for-each-ref", "refs/heads/", "--format", "%(refname:short)"]);

    if output.is_err() {
        eprintln!("{}", output.unwrap_err());
        std::process::exit(1);
    }

    let local_branch_names_with_newline = output.unwrap();
    let local_branch_names: Vec<&str> = local_branch_names_with_newline.split('\n').collect();

    println!("{:?}", local_branch_names);

    Ok(())
}
