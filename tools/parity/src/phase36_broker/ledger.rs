use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::contract::{sha256_hex, Phase36AllowedOperation, Phase36BrokerFailure};

const ZERO_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const MAX_INTERVAL_MILLIS: u64 = 3_600_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum Phase36LedgerTransition {
    Authorized,
    Invoked,
    Completed,
    Failed { category: Phase36BrokerFailure },
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Phase36LedgerRecord {
    pub(crate) sequence: u64,
    pub(crate) effect_id: u64,
    pub(crate) operation: Phase36AllowedOperation,
    pub(crate) transition: Phase36LedgerTransition,
    pub(crate) monotonic_millis: u64,
    pub(crate) previous_digest: String,
    pub(crate) record_digest: String,
}

impl Phase36LedgerRecord {
    pub fn next(
        state: &Phase36LedgerState,
        operation: Phase36AllowedOperation,
        transition: Phase36LedgerTransition,
        monotonic_millis: u64,
    ) -> Result<Self, Phase36LedgerError> {
        state.validate_requested_transition(operation, transition, monotonic_millis)?;
        let effect_id = state
            .active
            .map_or(state.next_effect_id, |active| active.effect_id);
        let mut record = Self {
            sequence: state.next_sequence,
            effect_id,
            operation,
            transition,
            monotonic_millis,
            previous_digest: state.previous_digest.clone(),
            record_digest: String::new(),
        };
        record.record_digest = record.computed_digest()?;
        Ok(record)
    }

    fn computed_digest(&self) -> Result<String, Phase36LedgerError> {
        let bytes = serde_json::to_vec(&(
            "phase36-broker-ledger-record-v1",
            self.sequence,
            self.effect_id,
            self.operation,
            self.transition,
            self.monotonic_millis,
            &self.previous_digest,
        ))
        .map_err(|_| Phase36LedgerError::Encoding)?;
        Ok(sha256_hex(&bytes))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationStage {
    Authorized,
    Invoked,
    Terminal {
        maybe_failure: Option<Phase36BrokerFailure>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActiveOperation {
    effect_id: u64,
    operation: Phase36AllowedOperation,
    stage: OperationStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Normal,
    RecoveryRequired,
    CleanupRequired,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase36LedgerState {
    interval_start_millis: u64,
    last_monotonic_millis: u64,
    next_sequence: u64,
    next_effect_id: u64,
    previous_digest: String,
    active: Option<ActiveOperation>,
    success_index: usize,
    flow: Flow,
    effect_count: u8,
    first_failure: Option<Phase36BrokerFailure>,
    secondary_failure: Option<Phase36BrokerFailure>,
}

impl Phase36LedgerState {
    pub fn start(interval_start_millis: u64) -> Result<Self, Phase36LedgerError> {
        if interval_start_millis == 0 {
            return Err(Phase36LedgerError::InvalidInterval);
        }
        Ok(Self {
            interval_start_millis,
            last_monotonic_millis: interval_start_millis,
            next_sequence: 1,
            next_effect_id: 1,
            previous_digest: ZERO_DIGEST.to_owned(),
            active: None,
            success_index: 0,
            flow: Flow::Normal,
            effect_count: 0,
            first_failure: None,
            secondary_failure: None,
        })
    }

    pub fn apply(&mut self, record: &Phase36LedgerRecord) -> Result<(), Phase36LedgerError> {
        if self.flow == Flow::Complete {
            return Err(Phase36LedgerError::PostClose);
        }
        if record.sequence < self.next_sequence {
            return Err(Phase36LedgerError::Duplicate);
        }
        if record.sequence != self.next_sequence {
            return Err(Phase36LedgerError::OutOfOrder);
        }
        if record.previous_digest != self.previous_digest {
            return Err(Phase36LedgerError::HashChain);
        }
        if record.record_digest != record.computed_digest()? {
            return Err(Phase36LedgerError::HashChain);
        }
        self.validate_requested_transition(
            record.operation,
            record.transition,
            record.monotonic_millis,
        )?;

        self.apply_transition(record)?;
        self.last_monotonic_millis = record.monotonic_millis;
        self.previous_digest.clone_from(&record.record_digest);
        self.next_sequence += 1;
        Ok(())
    }

    pub fn seal(
        self,
        interval_end_millis: u64,
    ) -> Result<Phase36EffectInterval, Phase36LedgerError> {
        if self.active.is_some() {
            return Err(Phase36LedgerError::Unclosed);
        }
        if self.flow != Flow::Complete {
            return Err(Phase36LedgerError::Incomplete);
        }
        if interval_end_millis <= self.last_monotonic_millis
            || interval_end_millis - self.interval_start_millis > MAX_INTERVAL_MILLIS
        {
            return Err(Phase36LedgerError::InvalidInterval);
        }
        let seal_bytes = serde_json::to_vec(&(
            "phase36-broker-ledger-seal-v1",
            self.interval_start_millis,
            interval_end_millis,
            &self.previous_digest,
            self.effect_count,
            self.first_failure,
            self.secondary_failure,
        ))
        .map_err(|_| Phase36LedgerError::Encoding)?;
        Ok(Phase36EffectInterval {
            start_millis: self.interval_start_millis,
            end_millis: interval_end_millis,
            effect_count: self.effect_count,
            ledger_digest: sha256_hex(&seal_bytes),
            first_failure: self.first_failure,
            secondary_failure: self.secondary_failure,
        })
    }

    fn validate_requested_transition(
        &self,
        operation: Phase36AllowedOperation,
        transition: Phase36LedgerTransition,
        monotonic_millis: u64,
    ) -> Result<(), Phase36LedgerError> {
        if monotonic_millis <= self.last_monotonic_millis {
            return Err(Phase36LedgerError::OutOfOrder);
        }
        let Some(active) = self.active else {
            if transition != Phase36LedgerTransition::Authorized
                || self.expected_operation()? != operation
            {
                return Err(Phase36LedgerError::OutOfOrder);
            }
            return Ok(());
        };
        if active.operation != operation {
            return Err(Phase36LedgerError::OutOfOrder);
        }
        match (active.stage, transition) {
            (OperationStage::Authorized, Phase36LedgerTransition::Invoked)
            | (OperationStage::Invoked, Phase36LedgerTransition::Completed)
            | (OperationStage::Terminal { .. }, Phase36LedgerTransition::Closed) => Ok(()),
            (OperationStage::Invoked, Phase36LedgerTransition::Failed { category })
                if category.valid_for(operation) =>
            {
                Ok(())
            }
            _ => Err(Phase36LedgerError::OutOfOrder),
        }
    }

    fn expected_operation(&self) -> Result<Phase36AllowedOperation, Phase36LedgerError> {
        match self.flow {
            Flow::Normal => Phase36AllowedOperation::SUCCESS_ORDER
                .get(self.success_index)
                .copied()
                .ok_or(Phase36LedgerError::Incomplete),
            Flow::RecoveryRequired => Ok(Phase36AllowedOperation::TypedRecovery),
            Flow::CleanupRequired => Ok(Phase36AllowedOperation::Cleanup),
            Flow::Complete => Err(Phase36LedgerError::PostClose),
        }
    }

    fn apply_transition(&mut self, record: &Phase36LedgerRecord) -> Result<(), Phase36LedgerError> {
        match record.transition {
            Phase36LedgerTransition::Authorized => {
                self.active = Some(ActiveOperation {
                    effect_id: record.effect_id,
                    operation: record.operation,
                    stage: OperationStage::Authorized,
                });
                self.effect_count = self
                    .effect_count
                    .checked_add(1)
                    .ok_or(Phase36LedgerError::Incomplete)?;
                self.next_effect_id += 1;
            }
            Phase36LedgerTransition::Invoked => {
                self.active_mut()?.stage = OperationStage::Invoked;
            }
            Phase36LedgerTransition::Completed => {
                self.active_mut()?.stage = OperationStage::Terminal {
                    maybe_failure: None,
                };
            }
            Phase36LedgerTransition::Failed { category } => {
                self.record_failure(category);
                self.active_mut()?.stage = OperationStage::Terminal {
                    maybe_failure: Some(category),
                };
            }
            Phase36LedgerTransition::Closed => self.close_active()?,
        }
        Ok(())
    }

    fn active_mut(&mut self) -> Result<&mut ActiveOperation, Phase36LedgerError> {
        self.active.as_mut().ok_or(Phase36LedgerError::OutOfOrder)
    }

    fn close_active(&mut self) -> Result<(), Phase36LedgerError> {
        let active = self.active.take().ok_or(Phase36LedgerError::OutOfOrder)?;
        let OperationStage::Terminal { maybe_failure } = active.stage else {
            return Err(Phase36LedgerError::Unclosed);
        };
        if active.operation == Phase36AllowedOperation::Cleanup {
            self.flow = Flow::Complete;
            return Ok(());
        }
        if maybe_failure.is_some() {
            self.flow = if active.operation == Phase36AllowedOperation::TypedRecovery {
                Flow::CleanupRequired
            } else {
                Flow::RecoveryRequired
            };
            return Ok(());
        }
        if active.operation == Phase36AllowedOperation::TypedRecovery {
            self.flow = Flow::CleanupRequired;
            return Ok(());
        }
        self.success_index += 1;
        Ok(())
    }

    fn record_failure(&mut self, failure: Phase36BrokerFailure) {
        if self.first_failure.is_none() {
            self.first_failure = Some(failure);
            return;
        }
        if self.secondary_failure.is_none() {
            self.secondary_failure = Some(failure);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Phase36EffectInterval {
    start_millis: u64,
    end_millis: u64,
    effect_count: u8,
    ledger_digest: String,
    first_failure: Option<Phase36BrokerFailure>,
    secondary_failure: Option<Phase36BrokerFailure>,
}

impl Phase36EffectInterval {
    #[must_use]
    pub const fn start_millis(&self) -> u64 {
        self.start_millis
    }

    #[must_use]
    pub const fn end_millis(&self) -> u64 {
        self.end_millis
    }

    #[must_use]
    pub const fn effect_count(&self) -> u8 {
        self.effect_count
    }

    #[must_use]
    pub fn ledger_digest(&self) -> &str {
        &self.ledger_digest
    }

    #[must_use]
    pub const fn first_failure(&self) -> Option<Phase36BrokerFailure> {
        self.first_failure
    }

    #[must_use]
    pub const fn secondary_failure(&self) -> Option<Phase36BrokerFailure> {
        self.secondary_failure
    }

    pub fn record_after_close(&self) -> Result<(), Phase36LedgerError> {
        Err(Phase36LedgerError::PostClose)
    }
}

#[derive(Debug)]
pub struct PrivateAppendOnlyLedger {
    file: File,
}

impl PrivateAppendOnlyLedger {
    pub fn create(path: &Utf8Path) -> Result<Self, Phase36LedgerError> {
        let parent = path.parent().ok_or(Phase36LedgerError::PrivateRoot)?;
        let parent_metadata =
            fs::symlink_metadata(parent).map_err(|_| Phase36LedgerError::PrivateRoot)?;
        if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
            return Err(Phase36LedgerError::PrivateRoot);
        }
        if parent_metadata.permissions().mode() & 0o777 != 0o700 {
            return Err(Phase36LedgerError::Permissions);
        }
        let file = OpenOptions::new()
            .append(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_APPEND | libc::O_CLOEXEC)
            .open(path)
            .map_err(|_| Phase36LedgerError::Storage)?;
        let mode = file
            .metadata()
            .map_err(|_| Phase36LedgerError::Storage)?
            .permissions()
            .mode()
            & 0o777;
        if mode != 0o600 {
            return Err(Phase36LedgerError::Permissions);
        }
        Ok(Self { file })
    }

    pub fn append(&mut self, record: &Phase36LedgerRecord) -> Result<(), Phase36LedgerError> {
        let mut bytes = serde_json::to_vec(record).map_err(|_| Phase36LedgerError::Encoding)?;
        bytes.push(b'\n');
        self.file
            .write_all(&bytes)
            .map_err(|_| Phase36LedgerError::Storage)?;
        self.file
            .sync_data()
            .map_err(|_| Phase36LedgerError::Storage)
    }

    pub fn seal(&mut self) -> Result<(), Phase36LedgerError> {
        self.file
            .sync_all()
            .map_err(|_| Phase36LedgerError::Storage)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Phase36LedgerError {
    #[error("phase36_ledger_duplicate")]
    Duplicate,
    #[error("phase36_ledger_out_of_order")]
    OutOfOrder,
    #[error("phase36_ledger_hash_chain_invalid")]
    HashChain,
    #[error("phase36_ledger_unclosed")]
    Unclosed,
    #[error("phase36_ledger_incomplete")]
    Incomplete,
    #[error("phase36_ledger_post_close")]
    PostClose,
    #[error("phase36_ledger_interval_invalid")]
    InvalidInterval,
    #[error("phase36_ledger_encoding_failed")]
    Encoding,
    #[error("phase36_ledger_private_root_invalid")]
    PrivateRoot,
    #[error("phase36_ledger_permissions_invalid")]
    Permissions,
    #[error("phase36_ledger_storage_failed")]
    Storage,
}
