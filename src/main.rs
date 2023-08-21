use clap::Parser;
use anyhow::{Result, Context};
use std::process::Command;

#[derive(Parser)]
struct Args {
    #[arg(required = true, index = 1, help = "Base branch name (e.g. main, develop)")]
    base_branch: String,
}

fn main() -> Result<()> {
    println!("Hello, world!");

    let output = Command::new("git").arg("-v").output().with_context(|| "Command not found: git");

    match output {
        Ok(output) => {
            println!("status: {}", output.status);
            let stdout = String::from_utf8(output.stdout)?;
            println!("stdout: {}", stdout);
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }

    Ok(())
}
