use clap::Parser;
use std::process::Command;

#[derive(Parser)]
struct Args {
    #[arg(required = true, index = 1, help = "Base branch name (e.g. main, develop)")]
    base_branch: String,
}

fn main() {
    println!("Hello, world!");

    let output = Command::new("git")
        .arg("-v")
        .output()
        .unwrap();

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    println!("{}", stdout_str);
}
