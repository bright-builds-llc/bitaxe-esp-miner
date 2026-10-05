use crate::*;

const TASK: &str = "task-str005-start-panic-diagnosis";
const ENABLE: &str =
    "Development core-dump acquisition: enabled (recovery evidence prerequisite satisfied).";

#[cfg(test)]
pub(super) fn admit_task(tasks: &str) -> Result<()> {
    admit_mode(tasks, false)
}

pub(super) fn admit_mode(tasks: &str, clear: bool) -> Result<()> {
    let owners = [
        (
            TASK,
            if clear {
                "Development core-dump clearing: enabled (private archive verified)."
            } else {
                ENABLE
            },
        ),
        (
            "task-str005-v2-accepted-share-probe",
            if clear {
                "Renew-image core-dump clearing: enabled (private archive verified)."
            } else {
                "Renew-image core-dump acquisition: enabled (fresh recovery required)."
            },
        ),
        (
            "task-str005-heap-loss-diagnosis",
            if clear {
                "Heap-loss core-dump clearing: enabled (private archive verified)."
            } else {
                "Heap-loss core-dump acquisition: enabled (fresh recovery required)."
            },
        ),
    ];
    let mut active = false;
    let mut maybe_selected = None;
    let mut counts = [0; 3];
    let mut enabled = [false; 3];
    for line in tasks.lines() {
        if line.starts_with("## ") {
            active = line.trim() == "## Active";
            maybe_selected = None;
        }
        if let Some(heading) = line.strip_prefix("### ") {
            maybe_selected = owners
                .iter()
                .position(|(task, _)| heading.split([' ', '|']).next() == Some(*task));
            if let Some(index) = maybe_selected {
                counts[index] += 1;
            }
        }
        if let Some(index) = maybe_selected {
            if active && line.trim() == owners[index].1 {
                enabled[index] = true;
            }
        }
    }
    if enabled.iter().filter(|&&value| value).count() != 1
        || enabled
            .iter()
            .enumerate()
            .any(|(index, &value)| value && counts[index] != 1)
    {
        bail!("core_dump=blocked reason=active_contract_disabled_or_ambiguous");
    }
    Ok(())
}

pub(super) fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

pub(super) fn create_root(root: &Utf8Path) -> Result<()> {
    if root
        .components()
        .any(|component| matches!(component, camino::Utf8Component::ParentDir))
    {
        bail!("core_dump=blocked reason=private_path");
    }
    if fs::symlink_metadata(root).is_ok() {
        bail!("core_dump=blocked reason=private_root_exists");
    }
    let parent = root.parent().context("private_parent_missing")?;
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        bail!("private_parent_invalid");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o700 {
        bail!("private_parent_mode");
    }
    for ancestor in parent.ancestors() {
        if fs::symlink_metadata(ancestor)?.file_type().is_symlink() {
            bail!("private_ancestor_symlink");
        }
    }
    fs::create_dir(root)?;
    set_private_directory_mode(root)
}

pub(super) fn dump_partition(bytes: &[u8]) -> Result<(u32, u32)> {
    if bytes.len() != 4096 {
        bail!("partition_table_length");
    }
    let table = esp_idf_part::PartitionTable::try_from_bytes(bytes)
        .map_err(|_| anyhow::anyhow!("partition_table_integrity"))?;
    table
        .validate()
        .map_err(|_| anyhow::anyhow!("partition_table_layout"))?;
    let mut ranges = Vec::new();
    let mut selected = Vec::new();
    let mut checksum = false;
    for (index, entry) in bytes.chunks_exact(32).enumerate() {
        if entry[..2] == [0xeb, 0xeb] {
            if bytes[(index + 1) * 32..].iter().any(|byte| *byte != 0xff) {
                bail!("partition_table_trailing_records");
            }
            checksum = true;
            break;
        }
        if entry[..2] != [0xaa, 0x50] {
            bail!("partition_table_entry");
        }
        let offset = u32::from_le_bytes(entry[4..8].try_into()?);
        let size = u32::from_le_bytes(entry[8..12].try_into()?);
        let end = offset
            .checked_add(size)
            .context("partition_range_overflow")?;
        if size == 0 || offset < 0x9000 || end > 0x1000000 {
            bail!("partition_range");
        }
        if ranges
            .iter()
            .any(|&(start, stop)| offset < stop && start < end)
        {
            bail!("partition_overlap");
        }
        ranges.push((offset, end));
        if entry[2..4] == [1, 3] {
            if entry[28..32] != [0; 4] {
                bail!("dump_partition_flags");
            }
            selected.push((offset, size));
        }
    }
    if !checksum || selected.len() != 1 {
        bail!("dump_partition_ambiguous");
    }
    let (offset, size) = selected[0];
    if offset != 0xf12000 || !matches!(size, 0x10000 | 0xee000) {
        bail!("dump_partition_layout");
    }
    Ok((offset, size))
}

// Build identity deliberately excludes documentation; the effect contract must also be clean.
pub(super) fn require_clean_contract(workspace: &Utf8Path) -> Result<()> {
    let output = Command::new("git")
        .current_dir(workspace)
        .args(["status", "--porcelain", "--untracked-files=normal"])
        .output()?;
    if !output.status.success() || !output.stdout.is_empty() {
        bail!("core_dump=blocked reason=contract_not_clean");
    }
    Ok(())
}
