use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use camino::{Utf8Path, Utf8PathBuf};

use std::fs;

static WORKSPACE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn create_workspace(name: &str) -> Utf8PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let process_id = std::process::id();
    let sequence = WORKSPACE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "phase29-generation-{name}-{timestamp}-{process_id}-{sequence}"
    ));
    fs::create_dir_all(&root).expect("workspace should be created");
    Utf8PathBuf::from_path_buf(root).expect("temp path should be UTF-8")
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
