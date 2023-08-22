use ansi_term::Colour::{Yellow, Green};
use anyhow::{anyhow, Result};
use std::io::Write; // for stdout().flush()
use std::process::Command;

/// Exec command, and returns stdout string
///
/// # Arguments
/// * `program` - Command name
/// * `args` - Arguments
pub fn exec_command(program: &str, args: &[&str]) -> Result<String> {
    let args_str = args.join(" ");

    let result = Command::new(program).args(args).output();

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

/// Exec command in a child process
///
/// # Arguments
/// * `program` - Command name
/// * `args` - Arguments
fn spawn_command(program: &str, args: &[&str]) -> Result<()> {
    let args_str = args.join(" ");

    let result = Command::new(program).args(args).spawn();
    if result.is_err() {
        return Err(anyhow!(result.unwrap_err()));
    }

    let mut child = result.unwrap();
    let result = child.wait();
    if result.is_err() {
        return Err(anyhow!(result.unwrap_err()));
    }

    let exit_status = result.unwrap();
    if !exit_status.success() {
        println!("{}", Yellow.paint(format!("[WARN] \"{} {}\" received {}", program, args_str, exit_status)));
    }

    Ok(())
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

pub fn delete_branches_with_prompt(base_branch_name: &str, deletable_branch_names: &Vec<String>, yes_flag: bool) -> Result<()> {
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

        let result = delete_branch_prompt(&target_branch_name, yes_flag);
        if result.is_err() {
            return Err(result.unwrap_err());
        }
    }

    Ok(())
}

/// Returns whether the target branch deleted
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
                let result = stdin.read_line(&mut user_input);
                if result.is_err() {
                    return Err(anyhow!(result.unwrap_err()));
                }

                user_input.trim_end_matches('\n').to_string()
            };

        match input.as_str() {
            "y" | "yes" => {
                let result = exec_command("git", &["rev-parse", target_branch_name]);
                if result.is_err() {
                    return Err(result.unwrap_err());
                }

                let latest_commit_id = result.unwrap();

                let result = exec_command("git", &["branch", "-D", target_branch_name]);
                if result.is_err() {
                    return Err(result.unwrap_err());
                }

                println!("{}", Green.paint(format!("Deleted '{}' branch", target_branch_name)));
                println!("You can recreate this branch with `git branch {} {}`", target_branch_name, latest_commit_id);

                return Ok(true);
            }
            "l" | "log" => {
                let result = spawn_command("git", &["log", target_branch_name]);
                if result.is_err() {
                    return Err(result.unwrap_err());
                }
            }
            "d" | "diff" => {
                let result = spawn_command("git", &["show", target_branch_name, "-v"]);
                if result.is_err() {
                    return Err(result.unwrap_err());
                }
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
            _ => {
                println!("Skipped");
                loop_end_flag = true;
            }
        }
    }

    Ok(false)
}
