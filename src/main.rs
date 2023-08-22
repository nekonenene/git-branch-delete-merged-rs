use ansi_term::Colour::Red;
use anyhow::Result;
use clap::Parser;
use std::process::Command;

use git_branch_delete_merged::{exec_command, is_squashed_branch};

#[derive(Parser)]
struct Args {
    #[arg(required = true, index = 1, help = "Base branch name (e.g. main, develop)")]
    base_branch: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let base_branch_name = args.base_branch;

    let mut deletable_branch_names = Vec::new();

    let output = Command::new("git").arg("version").output();
    if output.is_err() {
        eprintln!("{}", Red.paint("Command not found: git"));
        std::process::exit(1);
    }

    let output = exec_command("git", &["for-each-ref", "refs/heads/", "--format", "%(refname:short)"]);
    if output.is_err() {
        eprintln!("{}", Red.paint(&output.unwrap_err().to_string()));
        std::process::exit(1);
    }

    let local_branch_names_with_newline = output.unwrap();
    let local_branch_names: Vec<&str> = local_branch_names_with_newline.split('\n').collect();

    println!("{:?}", local_branch_names);

    // Add squashed branche names into deletable_branch_names
    for local_branch_name in local_branch_names.iter() {
        println!("Branch name: {}", local_branch_name);

        if local_branch_name.eq(&base_branch_name) {
            continue;
        }

        let result = is_squashed_branch(&base_branch_name, &local_branch_name);
        if result.is_err() {
            eprintln!("{}", Red.paint(&result.unwrap_err().to_string()));
            std::process::exit(1);
        }

        let is_squashed = result.unwrap();
        if is_squashed {
            deletable_branch_names.push(local_branch_name);
        }
    }

    println!("{:?}", deletable_branch_names);

    Ok(())
}
