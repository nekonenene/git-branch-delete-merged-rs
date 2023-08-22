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
                return Ok(stdout);
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

    let output = Command::new("git")
        .args(["for-each-ref", "refs/heads/", "--format", "%(refname:short)"])
        .output();

    match output {
        Ok(output) => {
            println!("status: {}", output.status);
            let stdout = String::from_utf8(output.stdout)?;
            println!("stdout: {}", stdout);
            let stderr = String::from_utf8(output.stderr)?;
            println!("stderr: {}", stderr);
        }
        Err(err) => {
            eprintln!("git command failed: {}", err);
            std::process::exit(1);
        }
    }

    let output = exec_command("git", &["afor-each-ref", "refs/heads/", "--format", "%(refname:short)"]);

    if output.is_err() {
        eprintln!("{}", output.unwrap_err());
        std::process::exit(1);
    } else {
        println!("{}", output.unwrap());
    }

    Ok(())
}
