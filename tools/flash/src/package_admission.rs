use anyhow::{bail, Context, Result};
use esp_idf_part::{AppType, Error as PartitionError, Partition, PartitionTable, SubType, Type};

use crate::esp32s3_image::{self, ExpectedApplication};

const PARTITION_TABLE_OFFSET: usize = 0x8000;
const PARTITION_TABLE_LEN: usize = 0x1000;
const FACTORY_PARTITION_OFFSET: u32 = 0x10000;
const FACTORY_PARTITION_SIZE: u32 = 0x400000;

pub(crate) struct ExpectedApplicationIdentity<'a> {
    pub(crate) build_label: &'a str,
    pub(crate) source_commit: &'a str,
    pub(crate) app_elf_sha256: &'a [u8],
}

pub(crate) fn validate_factory_ota_identity(
    factory_bytes: &[u8],
    ota_bytes: &[u8],
    expected: ExpectedApplicationIdentity<'_>,
) -> Result<()> {
    let ota_validation = esp32s3_image::validate(
        ota_bytes,
        ExpectedApplication {
            build_label: expected.build_label,
            source_commit: expected.source_commit,
            app_elf_sha256: expected.app_elf_sha256,
        },
    )?;

    let factory_partition = parse_factory_partition(factory_bytes)?;
    validate_factory_layout(&factory_partition)?;
    let factory_offset = usize::try_from(factory_partition.offset())
        .context("identity_admission=blocked reason=factory_partition_range_overflow")?;
    let partition_size = usize::try_from(factory_partition.size())
        .context("identity_admission=blocked reason=factory_partition_range_overflow")?;
    if ota_bytes.len() > partition_size {
        bail!("identity_admission=blocked reason=factory_partition_ota_oversized");
    }
    let factory_ota_end = factory_offset
        .checked_add(ota_bytes.len())
        .context("identity_admission=blocked reason=factory_partition_range_overflow")?;
    let Some(factory_ota) = factory_bytes.get(factory_offset..factory_ota_end) else {
        bail!("identity_admission=blocked reason=factory_image_undersized");
    };
    let factory_validation = esp32s3_image::validate(
        factory_ota,
        ExpectedApplication {
            build_label: expected.build_label,
            source_commit: expected.source_commit,
            app_elf_sha256: expected.app_elf_sha256,
        },
    )?;
    if factory_ota != ota_bytes || factory_validation != ota_validation {
        bail!("identity_admission=blocked reason=factory_ota_image_mismatch");
    }

    Ok(())
}

fn parse_factory_partition(factory_bytes: &[u8]) -> Result<Partition> {
    let table_end = PARTITION_TABLE_OFFSET
        .checked_add(PARTITION_TABLE_LEN)
        .context("identity_admission=blocked reason=factory_partition_table_range_overflow")?;
    let Some(table_bytes) = factory_bytes.get(PARTITION_TABLE_OFFSET..table_end) else {
        bail!("identity_admission=blocked reason=factory_partition_table_truncated");
    };
    let table = match PartitionTable::try_from_bytes(table_bytes) {
        Ok(table) => table,
        Err(PartitionError::MultipleFactoryPartitions) => {
            bail!("identity_admission=blocked reason=factory_partition_duplicate");
        }
        Err(_) => bail!("identity_admission=blocked reason=factory_partition_table_invalid"),
    };
    let matches = table
        .partitions()
        .iter()
        .filter(|partition| {
            partition.ty() == Type::App && partition.subtype() == SubType::App(AppType::Factory)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => bail!("identity_admission=blocked reason=factory_partition_missing"),
        [partition] => Ok((*partition).clone()),
        _ => bail!("identity_admission=blocked reason=factory_partition_duplicate"),
    }
}

fn validate_factory_layout(partition: &Partition) -> Result<()> {
    if partition.offset() != FACTORY_PARTITION_OFFSET || partition.size() != FACTORY_PARTITION_SIZE
    {
        bail!("identity_admission=blocked reason=factory_partition_layout_mismatch");
    }

    Ok(())
}

#[cfg(test)]
mod tests;
