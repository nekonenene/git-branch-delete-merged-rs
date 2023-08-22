use ansi_term::Colour::Red;
use anyhow::Result;
use clap::Parser;
use std::process::Command;

use git_branch_delete_merged::exec_command;

#[derive(Parser)]
struct Args {
    #[arg(required = true, index = 1, help = "Base branch name (e.g. main, develop)")]
    base_branch: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let base_branch_name = args.base_branch;
    println!("Base branch name: {}", base_branch_name);

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

    for local_branch_name in local_branch_names.iter() {
        println!("Branch name: {}", local_branch_name);

        let output = exec_command("git", &["rev-parse", local_branch_name]);
        if output.is_err() {
            eprintln!("{}", Red.paint(&output.unwrap_err().to_string()));
            std::process::exit(1);
        }

        let latest_commit_id = output.unwrap();
        println!("{}", latest_commit_id);
    }

    Ok(())
}
