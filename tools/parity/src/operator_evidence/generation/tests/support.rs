use std::time::{SystemTime, UNIX_EPOCH};

use super::super::*;
use crate::operator_evidence::ShareOutcome;

pub(super) fn create_workspace(name: &str) -> Utf8PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("phase29-generation-{name}-{unique}"));
    fs::create_dir_all(&root).expect("workspace should be created");
    Utf8PathBuf::from_path_buf(root).expect("temp path should be UTF-8")
}

pub(super) fn write_phase27_source(root: &Utf8Path, outcome: ShareOutcome) {
    fs::create_dir_all(root.as_std_path()).expect("source root should be created");
    let (slot_status, maybe_support) = match outcome {
        ShareOutcome::Accepted | ShareOutcome::Rejected => (
            "passed",
            "asic_bridge_status: result_correlated\nsafe_stop_status: complete\n",
        ),
        ShareOutcome::LiveSubmitResponseObserved => {
            unreachable!("Phase 28 source fixtures do not use Phase 25 outcomes")
        }
        ShareOutcome::BlockedSafePrerequisite => (
            "blocked",
            "asic_bridge_status: blocked\nsafe_stop_status: complete\n",
        ),
    };
    let common = format!(
        "board: 205\nredaction_status: passed\nraw_artifacts_committed: no\nraw_pool_values_committed: no\nshare_outcome: {}\n{maybe_support}",
        outcome.as_str()
    );
    for file in [
        "summary.md",
        "share-outcome.md",
        "redaction-review.md",
        "conclusion.md",
    ] {
        fs::write(root.join(file).as_std_path(), &common).expect("source file should write");
    }
    fs::write(
        root.join("share-outcome.md").as_std_path(),
        format!("slot: share-outcome\nslot_status: {slot_status}\n{common}"),
    )
    .expect("share source should write");
}

pub(super) fn snapshot(root: &Utf8Path) -> String {
    let mut entries = fs::read_dir(root.as_std_path())
        .expect("root should read")
        .map(|entry| entry.expect("entry should read"))
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    let mut output = String::new();
    for entry in entries {
        if !entry.path().is_file() {
            continue;
        }
        output.push_str(&entry.file_name().to_string_lossy());
        output.push('\n');
        output.push_str(&fs::read_to_string(entry.path()).expect("file should read"));
    }
    output
}

pub(super) fn find_staging_root(workspace: &Utf8Path) -> Option<Utf8PathBuf> {
    fs::read_dir(workspace.as_std_path())
        .expect("workspace should read")
        .filter_map(Result::ok)
        .find_map(|entry| {
            let name = entry.file_name();
            name.to_string_lossy()
                .starts_with(".destination.staging-")
                .then(|| Utf8PathBuf::from_path_buf(entry.path()).expect("path should be UTF-8"))
        })
}
