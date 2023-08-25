mod branches;
mod command;

use ansi_term::Colour::{Red, Yellow, Green};
use anyhow::Result;
use clap::Parser;
use spinners::{Spinner, Spinners};
use std::process::Command;

use crate::branches::{pick_merged_branches, pick_squashed_branches, delete_branches_with_prompt};
use crate::command::exec_command;

#[derive(Parser)]
#[command(author = "nekonenene", version, about)]
struct Args {
    #[arg(required = true, index = 1, help = "Base branch name (e.g. main, develop)")]
    base_branch: String,
    #[arg(required = false, long = "yes", short = 'y', action = clap::ArgAction::SetTrue, help = "Delete all merged branches without confirmations")]
    yes_flag: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let base_branch_name = args.base_branch.as_str();
    let yes_flag = args.yes_flag;

    let mut deletable_branch_names = Vec::new();

    let result = Command::new("git").arg("version").output();
    if let Err(_) = result {
        eprintln!("{}", Red.paint("Command not found: git"));
        std::process::exit(1);
    }

    let result = exec_command("git", &["rev-parse", "--verify", base_branch_name]);
    if let Err(_) = result {
        eprintln!("{}", Red.paint(format!("Base branch not found: {}", base_branch_name)));
        std::process::exit(1);
    }

    let mut sp = Spinner::new(Spinners::Dots2, "Searching merged branches...".into());

    let result = pick_merged_branches(base_branch_name);
    if let Err(err) = result {
        eprintln!("{}", Red.paint(err.to_string()));
        std::process::exit(1);
    }

    let mut merged_branch_names = result.unwrap();
    deletable_branch_names.append(&mut merged_branch_names);

    let result = pick_squashed_branches(base_branch_name);
    if let Err(err) = result {
        eprintln!("{}", Red.paint(err.to_string()));
        std::process::exit(1);
    }

    let mut squashed_branch_names = result.unwrap();
    deletable_branch_names.append(&mut squashed_branch_names);

    deletable_branch_names.sort();
    deletable_branch_names.dedup();

    sp.stop_with_newline();

    if deletable_branch_names.len() == 0 {
        println!("{}", Yellow.paint(format!("There is no branch which has merged into {}", base_branch_name)));
        std::process::exit(0);
    } else {
        println!("Found {} merged branches: [{}]", Green.paint(format!("{}", deletable_branch_names.len())), deletable_branch_names.join(" "));
    }

    let result = delete_branches_with_prompt(base_branch_name, &deletable_branch_names, yes_flag);
    if let Err(err) = result {
        eprintln!("{}", Red.paint(err.to_string()));
        std::process::exit(1);
    }

    Ok(())
}
