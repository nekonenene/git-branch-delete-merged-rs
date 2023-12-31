use ansi_term::Colour::{Yellow, Green};
use anyhow::Result;
use std::io::Write;

use crate::command::{exec_command, spawn_command};

/// Return branch names which has merged (Not include squashed)
///
/// # Arguments
/// * `base_branch_name` - Base branch (e.g. main, develop)
pub fn pick_merged_branches(base_branch_name: &str) -> Result<Vec<String>> {
    let merged_branch_names_with_newline =
        exec_command("git", &["branch", "--merged", base_branch_name, "--format", "%(refname:short)"])?;
    let mut merged_branch_names: Vec<String> = merged_branch_names_with_newline.split('\n').map(str::to_string).collect();
    merged_branch_names.retain(|branch_name| branch_name != base_branch_name);

    Ok(merged_branch_names)
}

/// Return branch names which has squashed and merged
///
/// # Arguments
/// * `base_branch_name` - Base branch (e.g. main, develop)
pub fn pick_squashed_branches(base_branch_name: &str) -> Result<Vec<String>> {
    let mut squashed_branch_names = Vec::new();

    let local_branch_names_with_newline =
        exec_command("git", &["for-each-ref", "refs/heads/", "--format", "%(refname:short)"])?;
    let local_branch_names: Vec<&str> = local_branch_names_with_newline.split('\n').collect();

    // Add squashed branche names into squashed_branch_names
    for local_branch_name in local_branch_names.into_iter() {
        if local_branch_name == base_branch_name {
            continue;
        }

        let is_squashed = is_squashed_branch(base_branch_name, local_branch_name)?;

        if is_squashed {
            squashed_branch_names.push(local_branch_name.to_string());
        }
    }

    Ok(squashed_branch_names)
}

/// Return whether target branch has squashed and merged
///
/// # Arguments
/// * `base_branch_name` - Base branch (e.g. main, develop)
/// * `target_branch_name` - Branch to be checked
fn is_squashed_branch(base_branch_name: &str, target_branch_name: &str) -> Result<bool> {
    let ancestor_commit_obj_hash =
        exec_command("git", &["merge-base", base_branch_name, target_branch_name])?;
    let root_tree_obj_hash =
        exec_command("git", &["rev-parse", &format!("{}^{{tree}}", target_branch_name)])?;
    let tmp_commit_obj_hash =
        exec_command("git", &["commit-tree", &root_tree_obj_hash, "-p", &ancestor_commit_obj_hash, "-m", "Temporary commit"])?;
    let cherry_result =
        exec_command("git", &["cherry", base_branch_name, &tmp_commit_obj_hash])?;

    if cherry_result.starts_with("- ") {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Delete each branch after the confirmation
///
/// # Arguments
/// * `base_branch_name` - Base branch (e.g. main, develop)
/// * `deletable_branch_names` - List of branches to be deleted
/// * `yes_flag` - If true, delete all branches without confirmation
pub fn delete_branches_with_prompt(base_branch_name: &str, deletable_branch_names: &Vec<String>, yes_flag: bool) -> Result<()> {
    let current_branch_name = exec_command("git", &["rev-parse", "--abbrev-ref", "HEAD"])?;

    for target_branch_name in deletable_branch_names.into_iter() {
        let target_branch_name = target_branch_name.to_string();
        if target_branch_name == base_branch_name {
            continue;
        }

        if target_branch_name == current_branch_name {
            println!("{}", Yellow.paint(format!("Skipped '{}' branch because it is current branch", target_branch_name)));
            continue;
        }

        delete_branch_prompt(&target_branch_name, yes_flag)?;
    }

    Ok(())
}

/// Show prompt to confirm deletion, and return whether the target branch deleted
///
/// # Arguments
/// * `target_branch_name` - Branch name to be deleted
/// * `yes_flag` - If true, delete the branch without confirmation
fn delete_branch_prompt(target_branch_name: &str, yes_flag: bool) -> Result<bool> {
    let mut loop_end_flag = false;

    while !loop_end_flag {
        let input =
            if yes_flag {
                String::from("yes")
            } else {
                print!("\nAre you sure to delete {} branch? [y|n|l|d|q|help]: ", Yellow.paint(format!("'{}'", target_branch_name)));
                std::io::stdout().flush().unwrap();

                let mut user_input = String::new();
                let stdin = std::io::stdin();
                stdin.read_line(&mut user_input)?;

                user_input.trim_end_matches('\n').to_string()
            };

        match input.as_str() {
            "y" | "yes" => {
                let latest_commit_id = exec_command("git", &["rev-parse", target_branch_name])?;
                exec_command("git", &["branch", "-D", target_branch_name])?;

                println!("{}", Green.paint(format!("Deleted '{}' branch", target_branch_name)));
                println!("You can recreate this branch with `git branch {} {}`", target_branch_name, latest_commit_id);

                return Ok(true);
            }
            "n" | "no" => {
                println!("Skipped");
                loop_end_flag = true;
            }
            "l" | "log" => {
                spawn_command("git", &["log", target_branch_name, "-100"])?; // Show only 100 logs to avoid broken pipe error
            }
            "d" | "diff" => {
                spawn_command("git", &["show", target_branch_name, "-v"])?;
            }
            "q" | "quit" => {
                println!("{}", Yellow.paint("Suspends processing"));
                std::process::exit(0);
            }
            "h" | "help" => {
                println!("\n\
                    y: Yes, delete the branch\n\
                    n: No, skip deleting\n\
                    l: Show git logs of the branch\n\
                    d: Show the latest commit of the branch and its diff\n\
                    q: Quit immediately\n\
                    h: Display this help\
                ");
            }
            _ => {}
        }
    }

    Ok(false)
}
