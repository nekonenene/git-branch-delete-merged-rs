use ansi_term::Colour::Red;
use anyhow::Result;
use clap::Parser;
use std::process::Command;

use git_branch_delete_merged::{exec_command, pick_merged_branches, pick_squashed_branches};

#[derive(Parser)]
struct Args {
    #[arg(required = true, index = 1, help = "Base branch name (e.g. main, develop)")]
    base_branch: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let base_branch_name = &args.base_branch;

    let mut deletable_branch_names = Vec::new();

    let result = Command::new("git").arg("version").output();
    if result.is_err() {
        eprintln!("{}", Red.paint("Command not found: git"));
        std::process::exit(1);
    }

    let result = exec_command("git", &["for-each-ref", "refs/heads/", "--format", "%(refname:short)"]);
    if result.is_err() {
        eprintln!("{}", Red.paint(&result.unwrap_err().to_string()));
        std::process::exit(1);
    }

    let result = pick_merged_branches(base_branch_name);
    if result.is_err() {
        eprintln!("{}", Red.paint(&result.unwrap_err().to_string()));
        std::process::exit(1);
    }

    let mut merged_branch_names = result.unwrap();
    deletable_branch_names.append(&mut merged_branch_names);

    let result = pick_squashed_branches(base_branch_name);
    if result.is_err() {
        eprintln!("{}", Red.paint(&result.unwrap_err().to_string()));
        std::process::exit(1);
    }

    let mut squashed_branch_names = result.unwrap();
    deletable_branch_names.append(&mut squashed_branch_names);

    deletable_branch_names.sort();
    deletable_branch_names.dedup();

    println!("Deletable branches: {:?}", deletable_branch_names);

    Ok(())
}
