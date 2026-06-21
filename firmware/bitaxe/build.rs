use std::process::Command;

fn main() {
    embuild::espidf::sysenv::output();

    let Some(commit) = maybe_git_commit() else {
        return;
    };

    println!("cargo:rustc-env=BITAXE_FIRMWARE_COMMIT={commit}");
}

fn maybe_git_commit() -> Option<String> {
    let maybe_output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;

    if !maybe_output.status.success() {
        return None;
    }

    let commit = String::from_utf8(maybe_output.stdout).ok()?;
    let commit = commit.trim();
    if commit.is_empty() {
        return None;
    }

    Some(commit.to_owned())
}
