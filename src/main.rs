use clap::Parser;
use anyhow::Result;
use std::process::Command;

#[derive(Parser)]
struct Args {
    #[arg(required = true, index = 1, help = "Base branch name (e.g. main, develop)")]
    base_branch: String,
}

fn main() -> Result<()> {
    let output = Command::new("git").arg("version").output();

    if output.is_err() {
        eprintln!("Command not found: git");
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

    Ok(())
}
