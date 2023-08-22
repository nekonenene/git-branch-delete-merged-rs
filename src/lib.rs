use ansi_term::Colour::Yellow;
use anyhow::{anyhow, Result};
use std::process::Command;

/// Exec command, and returns stdout string
///
/// # Arguments
/// * `program` - Command name
/// * `args` - Arguments
pub fn exec_command(program: &str, args: &[&str]) -> Result<String> {
    let result = Command::new(program).args(args).output();
    let args_str = args.join(" ");

    match result {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)?;
                let trimmed_stdout = stdout.trim_end_matches('\n').to_string();
                return Ok(trimmed_stdout);
            } else {
                let stderr = String::from_utf8(output.stderr)?;
                return Err(anyhow!("\"{} {}\" received {}\n\n{}", program, args_str, output.status, stderr));
            }
        }
        Err(err) => {
            return Err(anyhow!("\"{} {}\" failed:\n{}", program, args_str, err));
        }
    }
}

/// Returns branch names which has merged (Not include squashed)
///
/// # Arguments
/// * `base_branch_name` - Base branch (e.g. main, develop)
pub fn pick_merged_branches(base_branch_name: &str) -> Result<Vec<String>> {
    let result = exec_command("git", &["branch", "--merged", base_branch_name, "--format", "%(refname:short)"]);
    if result.is_err() {
        return Err(result.unwrap_err());
    }

    let merged_branch_names_with_newline = result.unwrap();
    let mut merged_branch_names: Vec<String> = merged_branch_names_with_newline.split('\n').map(str::to_string).collect();
    merged_branch_names.retain(|branch_name| branch_name != base_branch_name);

    Ok(merged_branch_names)
}

/// Returns branch names which has squashed and merged
///
/// # Arguments
/// * `base_branch_name` - Base branch (e.g. main, develop)
pub fn pick_squashed_branches(base_branch_name: &str) -> Result<Vec<String>> {
    let mut squashed_branch_names = Vec::new();

    let result = exec_command("git", &["for-each-ref", "refs/heads/", "--format", "%(refname:short)"]);
    if result.is_err() {
        return Err(result.unwrap_err());
    }

    let local_branch_names_with_newline = result.unwrap();
    let local_branch_names: Vec<&str> = local_branch_names_with_newline.split('\n').collect();

    // Add squashed branche names into squashed_branch_names
    for local_branch_name in local_branch_names.into_iter() {
        if local_branch_name == base_branch_name {
            continue;
        }

        let result = is_squashed_branch(base_branch_name, local_branch_name);
        if result.is_err() {
            return Err(result.unwrap_err());
        }

        let is_squashed = result.unwrap();
        if is_squashed {
            squashed_branch_names.push(local_branch_name.to_string());
        }
    }

    Ok(squashed_branch_names)
}

/// Returns whether target branch has squashed and merged
///
/// # Arguments
/// * `base_branch_name` - Base branch (e.g. main, develop)
/// * `target_branch_name` - Branch to be checked
fn is_squashed_branch(base_branch_name: &str, target_branch_name: &str) -> Result<bool> {
    let result = exec_command("git", &["merge-base", base_branch_name, target_branch_name]);
    if result.is_err() {
        return Err(result.unwrap_err());
    }

    let ancestor_commit_obj_hash = result.unwrap();

    let result = exec_command("git", &["rev-parse", &format!("{}^{{tree}}", target_branch_name)]);
    if result.is_err() {
        return Err(result.unwrap_err());
    }

    let root_tree_obj_hash = result.unwrap();

    let result = exec_command("git", &["commit-tree", &root_tree_obj_hash, "-p", &ancestor_commit_obj_hash, "-m", "Temporary commit"]);
    if result.is_err() {
        return Err(result.unwrap_err());
    }

    let tmp_commit_obj_hash = result.unwrap();

    let result = exec_command("git", &["cherry", base_branch_name, &tmp_commit_obj_hash]);
    if result.is_err() {
        return Err(result.unwrap_err());
    }

    let cherry_result = result.unwrap();

    if cherry_result.starts_with("- ") {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_branches_with_prompt(base_branch_name: &str, deletable_branch_names: &Vec<String>) -> Result<()> {
    let result = exec_command("git", &["rev-parse", "--abbrev-ref", "HEAD"]);
    if result.is_err() {
        return Err(result.unwrap_err());
    }

    let current_branch_name = result.unwrap();

    for target_branch_name in deletable_branch_names.into_iter() {
        let target_branch_name = target_branch_name.to_string();
        if target_branch_name == base_branch_name {
            continue;
        }

        if target_branch_name == current_branch_name {
            println!("{}", Yellow.paint(format!("Skipped '{}' branch because it is current branch", target_branch_name)));
            continue;
        }

        println!("Start deleting branch: {}", target_branch_name);
    }

    Ok(())
}
