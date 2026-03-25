//! Git stash operations.

use std::path::Path;

use super::command::{git_command_stdout, run_git_simple};

/// A single stash entry from `git stash list`.
#[derive(Debug, Clone)]
pub struct StashEntry {
    /// Stash index (0 = most recent)
    pub index: usize,
    /// Branch where stash was created
    pub branch: String,
    /// Human-readable message (the commit summary after the branch)
    pub message: String,
    /// Full ref string, e.g. `stash@{0}` — use this for git commands
    pub ref_str: String,
}

/// List all stash entries for the given repository.
///
/// Returns entries ordered by index (0 = most recent).
pub fn stash_list(repo: &Path) -> Vec<StashEntry> {
    let output = match git_command_stdout(repo, &["stash", "list"]) {
        Some(s) if !s.trim().is_empty() => s,
        _ => return Vec::new(),
    };

    output.lines().filter_map(parse_stash_line).collect()
}

/// Parse a single line from `git stash list`.
///
/// Format: `stash@{N}: WIP on branch: hash message`
///         `stash@{N}: On branch: custom message`
fn parse_stash_line(line: &str) -> Option<StashEntry> {
    // Extract ref_str (the part before ": ")
    let colon_pos = line.find(": ")?;
    let ref_str = line[..colon_pos].to_string();

    // Extract index from "stash@{N}"
    let index: usize = ref_str
        .strip_prefix("stash@{")?
        .strip_suffix('}')?
        .parse()
        .ok()?;

    // Rest after ": "
    let rest = &line[colon_pos + 2..];

    // Try to extract branch from "WIP on branch: ..." or "On branch: ..."
    let (branch, message) = if let Some(after_wip) = rest.strip_prefix("WIP on ") {
        if let Some(colon2) = after_wip.find(": ") {
            let branch = after_wip[..colon2].to_string();
            let msg = after_wip[colon2 + 2..].to_string();
            (branch, msg)
        } else {
            (after_wip.to_string(), String::new())
        }
    } else if let Some(after_on) = rest.strip_prefix("On ") {
        if let Some(colon2) = after_on.find(": ") {
            let branch = after_on[..colon2].to_string();
            let msg = after_on[colon2 + 2..].to_string();
            (branch, msg)
        } else {
            (after_on.to_string(), String::new())
        }
    } else {
        (String::new(), rest.to_string())
    };

    Some(StashEntry {
        index,
        branch,
        message,
        ref_str,
    })
}

/// Create a new stash with an optional message.
pub fn stash_push(repo: &Path, message: &str) -> Result<(), String> {
    if message.is_empty() {
        run_git_simple(repo, &["stash", "push"], "Failed to create stash")
    } else {
        run_git_simple(
            repo,
            &["stash", "push", "-m", message],
            "Failed to create stash",
        )
    }
}

/// Pop (apply + drop) the stash at the given index.
pub fn stash_pop(repo: &Path, index: usize) -> Result<(), String> {
    let ref_str = format!("stash@{{{}}}", index);
    run_git_simple(repo, &["stash", "pop", &ref_str], "Failed to pop stash")
}

/// Apply the stash at the given index (keep it in the stash list).
pub fn stash_apply(repo: &Path, index: usize) -> Result<(), String> {
    let ref_str = format!("stash@{{{}}}", index);
    run_git_simple(repo, &["stash", "apply", &ref_str], "Failed to apply stash")
}

/// Drop (delete) the stash at the given index.
pub fn stash_drop(repo: &Path, index: usize) -> Result<(), String> {
    let ref_str = format!("stash@{{{}}}", index);
    run_git_simple(repo, &["stash", "drop", &ref_str], "Failed to drop stash")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wip_on() {
        let line = "stash@{0}: WIP on main: abc1234 fix something";
        let entry = parse_stash_line(line).unwrap();
        assert_eq!(entry.index, 0);
        assert_eq!(entry.ref_str, "stash@{0}");
        assert_eq!(entry.branch, "main");
        assert_eq!(entry.message, "abc1234 fix something");
    }

    #[test]
    fn test_parse_on() {
        let line = "stash@{1}: On feature/foo: my custom message";
        let entry = parse_stash_line(line).unwrap();
        assert_eq!(entry.index, 1);
        assert_eq!(entry.branch, "feature/foo");
        assert_eq!(entry.message, "my custom message");
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_stash_line("not a stash line").is_none());
    }
}
