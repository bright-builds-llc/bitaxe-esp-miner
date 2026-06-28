use std::fs;

use anyhow::{bail, Context, Result};
use camino::Utf8Path;

#[derive(Debug, Clone, Eq, PartialEq)]
struct PartitionRow {
    name: String,
    partition_type: String,
    subtype: String,
    offset: String,
    size: String,
}

pub(crate) fn validate_ultra205_partition_contract(path: &Utf8Path) -> Result<()> {
    let contents = fs::read_to_string(path.as_std_path())
        .with_context(|| format!("failed to read partition table {path}"))?;
    let partitions = parse_partition_table(&contents)?;

    require_partition(&partitions, "nvs", "data", "nvs", "0x9000", "0x6000")?;
    require_partition(&partitions, "phy_init", "data", "phy", "0xf000", "0x1000")?;
    require_partition(&partitions, "factory", "app", "factory", "0x10000", "4M")?;
    require_partition(&partitions, "www", "data", "spiffs", "0x410000", "3M")?;
    require_partition(&partitions, "ota_0", "app", "ota_0", "0x710000", "4M")?;
    require_partition(&partitions, "ota_1", "app", "ota_1", "0xb10000", "4M")?;
    require_partition(&partitions, "otadata", "data", "ota", "0xf10000", "8k")?;
    require_partition(&partitions, "coredump", "data", "coredump", "", "64K")
}

fn parse_partition_table(contents: &str) -> Result<Vec<PartitionRow>> {
    let mut partitions = Vec::new();

    for (line_index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let columns: Vec<&str> = line.split(',').map(str::trim).collect();
        if columns.len() < 5 {
            bail!(
                "partition table line {} has {} columns, expected at least 5",
                line_index + 1,
                columns.len()
            );
        }

        partitions.push(PartitionRow {
            name: columns[0].to_owned(),
            partition_type: columns[1].to_owned(),
            subtype: columns[2].to_owned(),
            offset: columns[3].to_owned(),
            size: columns[4].to_owned(),
        });
    }

    Ok(partitions)
}

fn require_partition(
    partitions: &[PartitionRow],
    name: &str,
    partition_type: &str,
    subtype: &str,
    offset: &str,
    size: &str,
) -> Result<()> {
    let Some(partition) = partitions.iter().find(|partition| partition.name == name) else {
        bail!("missing required Ultra 205 partition {name}");
    };

    if partition.partition_type != partition_type
        || partition.subtype != subtype
        || partition.offset != offset
        || partition.size != size
    {
        bail!(
            "partition {name} drifted: expected type={partition_type} subtype={subtype} offset={offset} size={size}, found type={} subtype={} offset={} size={}",
            partition.partition_type,
            partition.subtype,
            partition.offset,
            partition.size
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::{Utf8Path, Utf8PathBuf};
    use std::env;
    use tempfile::{tempdir, TempDir};

    #[test]
    fn partition_contract_accepts_checked_in_ultra205_table() {
        // Arrange
        let path = checked_in_partition_table();

        // Act
        let result = validate_ultra205_partition_contract(&path);

        // Assert
        assert!(result.is_ok(), "{result:#?}");
    }

    #[test]
    fn partition_contract_rejects_www_offset_drift() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let path = write_partition_table(
            &dir,
            "nvs,data,nvs,0x9000,0x6000\n\
             phy_init,data,phy,0xf000,0x1000\n\
             factory,app,factory,0x10000,4M\n\
             www,data,spiffs,0x420000,3M\n\
             ota_0,app,ota_0,0x710000,4M\n\
             ota_1,app,ota_1,0xb10000,4M\n\
             otadata,data,ota,0xf10000,8k\n\
             coredump,data,coredump,,64K\n",
        );

        // Act
        let result = validate_ultra205_partition_contract(&path);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("www"));
        assert!(error.contains("0x410000"));
    }

    #[test]
    fn partition_contract_rejects_missing_otadata() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let path = write_partition_table(
            &dir,
            "nvs,data,nvs,0x9000,0x6000\n\
             phy_init,data,phy,0xf000,0x1000\n\
             factory,app,factory,0x10000,4M\n\
             www,data,spiffs,0x410000,3M\n\
             ota_0,app,ota_0,0x710000,4M\n\
             ota_1,app,ota_1,0xb10000,4M\n\
             coredump,data,coredump,,64K\n",
        );

        // Act
        let result = validate_ultra205_partition_contract(&path);

        // Assert
        assert!(format!("{result:#?}").contains("otadata"));
    }

    fn checked_in_partition_table() -> Utf8PathBuf {
        let relative_path = "firmware/bitaxe/partitions-ultra205.csv";
        let cargo_path = repo_root().join(relative_path);
        if cargo_path.is_file() {
            return cargo_path;
        }

        for env_name in ["TEST_SRCDIR", "RUNFILES_DIR"] {
            let Ok(runfiles_dir) = env::var(env_name) else {
                continue;
            };

            let base = Utf8PathBuf::from(runfiles_dir);
            if let Ok(workspace) = env::var("TEST_WORKSPACE") {
                let candidate = base.join(workspace).join(relative_path);
                if candidate.is_file() {
                    return candidate;
                }
            }

            let main_candidate = base.join("_main").join(relative_path);
            if main_candidate.is_file() {
                return main_candidate;
            }
        }

        cargo_path
    }

    fn repo_root() -> Utf8PathBuf {
        Utf8Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
    }

    fn write_partition_table(dir: &TempDir, contents: &str) -> Utf8PathBuf {
        let path =
            Utf8PathBuf::from_path_buf(dir.path().join("partitions-ultra205.csv")).expect("utf8");
        std::fs::write(path.as_std_path(), contents).expect("write partition table");
        path
    }
}
