use anyhow::{anyhow, Result};
use std::process::Command;

pub fn exec_command(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program).args(args).output();
    let args_str = args.join(" ");

    match output {
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

/// Returns whether target branch has squashed and merged
///
/// # Arguments
///
/// * `base_branch_name` - Base branch (e.g. main, develop)
/// * `target_branch_name` - Branch to be checked
pub fn is_squashed_branch(base_branch_name: &str, target_branch_name: &str) -> Result<bool> {
    let output = exec_command("git", &["merge-base", base_branch_name, target_branch_name]);
    if output.is_err() {
        return Err(output.unwrap_err());
    }

    let ancestor_commit_obj_hash = output.unwrap();

    let output = exec_command("git", &["rev-parse", &format!("{}^{{tree}}", target_branch_name)]);
    if output.is_err() {
        return Err(output.unwrap_err());
    }

    let root_tree_obj_hash = output.unwrap();

    let output = exec_command("git", &["commit-tree", &root_tree_obj_hash, "-p", &ancestor_commit_obj_hash, "-m", "Temporary commit"]);
    if output.is_err() {
        return Err(output.unwrap_err());
    }

    let tmp_commit_obj_hash = output.unwrap();

    let output = exec_command("git", &["cherry", base_branch_name, &tmp_commit_obj_hash]);
    if output.is_err() {
        return Err(output.unwrap_err());
    }

    let cherry_result = output.unwrap();

    if cherry_result.starts_with("- ") {
        Ok(true)
    } else {
        Ok(false)
    }
}
