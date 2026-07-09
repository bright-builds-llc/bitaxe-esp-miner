use std::env;
use std::process::Command;

fn main() {
    embuild::espidf::sysenv::output();
    println!("cargo:rerun-if-env-changed=BITAXE_SOURCE_COMMIT");
    println!("cargo:rerun-if-env-changed=BITAXE_MINING_EVIDENCE_MODE");
    println!("cargo:rerun-if-env-changed=BITAXE_HARDWARE_EVIDENCE_ACK");
    println!("cargo:rerun-if-env-changed=BITAXE_WORK_RESULT_INVESTIGATION");
    println!("cargo:rerun-if-env-changed=BITAXE_CHIP_DETECT_INVESTIGATION");
    emit_git_rerun_hints();

    let Some(commit) = maybe_git_commit() else {
        return;
    };

    println!("cargo:rustc-env=BITAXE_FIRMWARE_COMMIT={commit}");
}

fn emit_git_rerun_hints() {
    if let Some(head_path) = git_path("HEAD") {
        println!("cargo:rerun-if-changed={head_path}");
    }

    let Some(ref_name) = git_stdout(["symbolic-ref", "-q", "HEAD"]) else {
        return;
    };
    let Some(ref_path) = git_path(ref_name.trim()) else {
        return;
    };

    println!("cargo:rerun-if-changed={ref_path}");
}

fn maybe_git_commit() -> Option<String> {
    if let Some(commit) = maybe_env_commit() {
        return Some(commit);
    }

    git_stdout(["rev-parse", "--short=12", "HEAD"])
}

fn maybe_env_commit() -> Option<String> {
    let commit = env::var("BITAXE_SOURCE_COMMIT").ok()?;
    let commit = commit.trim();
    if commit.is_empty() {
        return None;
    }

    Some(commit.to_owned())
}

fn git_path(path: &str) -> Option<String> {
    git_stdout(["rev-parse", "--git-path", path])
}

fn git_stdout<const N: usize>(args: [&str; N]) -> Option<String> {
    let maybe_output = Command::new("git").args(args).output().ok()?;

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
