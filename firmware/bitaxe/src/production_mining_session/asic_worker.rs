//! Single bounded worker around the retained BM1366 production executor.

use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::time::Instant;

use bitaxe_asic::bm1366::{
    command::VersionMask,
    production::Bm1366ProductionCommand,
    result::{Bm1366NonceResult, Bm1366RegisterRead, Bm1366ValidJobIds},
};
use bitaxe_stratum::v1::production_session::AsicPollCompletion;
use bitaxe_stratum::v1::production_session::{ProductionAsicFailure, ProductionSessionEffect};
use bitaxe_stratum::v1::production_work::PoolSessionGeneration;

use super::revocation::{self, WorkPermit, WorkerGeneration};
use crate::asic_adapter::production::{
    apply_negotiated_version_mask, request_hashrate_monitor_register_reads_tx,
    ProductionAsicExecutor, ProductionReadOutcome,
};

const COMMAND_CAPACITY: usize = 8;
const WORKER_STACK_BYTES: usize = 12 * 1024;

pub(super) enum AsicWorkerCommand {
    ApplyVersionMask {
        generation: PoolSessionGeneration,
        mask: VersionMask,
    },
    Dispatch {
        generation: PoolSessionGeneration,
        valid_jobs: Bm1366ValidJobIds,
        command: Bm1366ProductionCommand,
    },
    Poll {
        generation: PoolSessionGeneration,
        valid_jobs: Bm1366ValidJobIds,
        slice_ms: u32,
    },
    ReadHashrateRegisters {
        generation: PoolSessionGeneration,
    },
    Shutdown,
}

impl core::fmt::Debug for AsicWorkerCommand {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ApplyVersionMask { generation, .. } => formatter
                .debug_struct("AsicWorkerCommand::ApplyVersionMask")
                .field("generation", generation)
                .field("mask", &"redacted")
                .finish(),
            Self::Dispatch { generation, .. } => formatter
                .debug_struct("AsicWorkerCommand::Dispatch")
                .field("generation", generation)
                .field("payload", &"redacted")
                .finish(),
            Self::Poll {
                generation,
                slice_ms,
                ..
            } => formatter
                .debug_struct("AsicWorkerCommand::Poll")
                .field("generation", generation)
                .field("valid_jobs", &"redacted")
                .field("slice_ms", slice_ms)
                .finish(),
            Self::ReadHashrateRegisters { generation } => formatter
                .debug_struct("AsicWorkerCommand::ReadHashrateRegisters")
                .field("generation", generation)
                .finish(),
            Self::Shutdown => formatter.write_str("AsicWorkerCommand::Shutdown"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AsicWorkerEvent {
    Result {
        generation: PoolSessionGeneration,
        result: Bm1366NonceResult,
    },
    PollTimedOut {
        generation: PoolSessionGeneration,
    },
    PollCompleted {
        generation: PoolSessionGeneration,
        completion: AsicPollCompletion,
    },
    RegisterRead {
        generation: PoolSessionGeneration,
        read: Bm1366RegisterRead,
        observed_at_us: u64,
    },
    Failed {
        generation: PoolSessionGeneration,
        failure: ProductionAsicFailure,
    },
}

impl core::fmt::Debug for AsicWorkerEvent {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Result { generation, .. } => formatter
                .debug_struct("AsicWorkerEvent::Result")
                .field("generation", generation)
                .field("result", &"redacted")
                .finish(),
            Self::PollTimedOut { generation } => formatter
                .debug_struct("AsicWorkerEvent::PollTimedOut")
                .field("generation", generation)
                .finish(),
            Self::PollCompleted {
                generation,
                completion,
            } => formatter
                .debug_struct("AsicWorkerEvent::PollCompleted")
                .field("generation", generation)
                .field("completion", completion)
                .finish(),
            Self::RegisterRead { generation, .. } => formatter
                .debug_struct("AsicWorkerEvent::RegisterRead")
                .field("generation", generation)
                .field("read", &"redacted")
                .finish(),
            Self::Failed {
                generation,
                failure,
            } => formatter
                .debug_struct("AsicWorkerEvent::Failed")
                .field("generation", generation)
                .field("failure", failure)
                .finish(),
        }
    }
}

pub(super) struct AsicWorker {
    sender: SyncSender<(AsicWorkerCommand, WorkPermit)>,
}

impl AsicWorker {
    pub(super) fn spawn(emit: impl Fn(AsicWorkerEvent) + Send + 'static) -> std::io::Result<Self> {
        let (sender, receiver) =
            mpsc::sync_channel::<(AsicWorkerCommand, WorkPermit)>(COMMAND_CAPACITY);
        std::thread::Builder::new()
            .name("production-asic".to_owned())
            .stack_size(WORKER_STACK_BYTES)
            .spawn(move || {
                let started_at = Instant::now();
                let mut executor = ProductionAsicExecutor::new();
                while let Ok((command, permit)) = receiver.recv() {
                    if matches!(command, AsicWorkerCommand::Shutdown) {
                        return;
                    }
                    if !revocation::permits_work(permit) {
                        continue;
                    }
                    if !crate::asic_adapter::production::set_production_work_permit(permit) {
                        continue;
                    }
                    match command {
                        AsicWorkerCommand::ApplyVersionMask { generation, mask } => {
                            if !apply_negotiated_version_mask(mask) {
                                emit(AsicWorkerEvent::Failed {
                                    generation,
                                    failure: ProductionAsicFailure::VersionMask,
                                });
                            }
                        }
                        AsicWorkerCommand::Dispatch {
                            generation,
                            valid_jobs,
                            command,
                        } => match executor.maybe_execute_guarded(command, &valid_jobs, permit) {
                            Ok(Some(result)) => {
                                revocation::note_dispatch(
                                    permit.maybe_generation(),
                                    crate::runtime_uptime::millis(),
                                );
                                emit(AsicWorkerEvent::Result { generation, result });
                            }
                            Ok(None) => revocation::note_dispatch(
                                permit.maybe_generation(),
                                crate::runtime_uptime::millis(),
                            ),
                            Err(_) => emit(AsicWorkerEvent::Failed {
                                generation,
                                failure: ProductionAsicFailure::Dispatch,
                            }),
                        },
                        AsicWorkerCommand::Poll {
                            generation,
                            valid_jobs,
                            slice_ms,
                        } => match executor
                            .try_read_production_result(&valid_jobs, slice_ms.min(50))
                        {
                            Ok(ProductionReadOutcome::JobNonce(result)) => {
                                emit(AsicWorkerEvent::Result { generation, result });
                            }
                            Ok(ProductionReadOutcome::Pending) => {
                                emit(AsicWorkerEvent::PollTimedOut { generation });
                            }
                            Ok(ProductionReadOutcome::Discarded(reason)) => {
                                emit(AsicWorkerEvent::PollCompleted {
                                    generation,
                                    completion: AsicPollCompletion::Discarded(reason),
                                });
                            }
                            Ok(ProductionReadOutcome::RegisterReadProof(read)) => {
                                emit(AsicWorkerEvent::RegisterRead {
                                    generation,
                                    read,
                                    observed_at_us: elapsed_micros(started_at),
                                });
                            }
                            Err(_) => emit(AsicWorkerEvent::Failed {
                                generation,
                                failure: ProductionAsicFailure::Poll,
                            }),
                        },
                        AsicWorkerCommand::ReadHashrateRegisters { .. } => {
                            if !request_hashrate_monitor_register_reads_tx() {
                                log::warn!("hashrate_monitor_read=unavailable");
                            }
                        }
                        AsicWorkerCommand::Shutdown => return,
                    }
                }
            })?;
        Ok(Self { sender })
    }

    pub(super) fn try_send(
        &self,
        command: AsicWorkerCommand,
        maybe_generation: Option<WorkerGeneration>,
    ) -> Result<(), TrySendError<AsicWorkerCommand>> {
        self.sender
            .try_send((command, revocation::stamp(maybe_generation)))
            .map_err(|error| match error {
                TrySendError::Full((command, _)) => TrySendError::Full(command),
                TrySendError::Disconnected((command, _)) => TrySendError::Disconnected(command),
            })
    }

    pub(super) fn command_from_effect(
        effect: ProductionSessionEffect,
    ) -> Result<AsicWorkerCommand, ProductionSessionEffect> {
        match effect {
            ProductionSessionEffect::ApplyVersionMask { generation, mask } => {
                Ok(AsicWorkerCommand::ApplyVersionMask { generation, mask })
            }
            ProductionSessionEffect::DispatchAsic {
                generation,
                valid_jobs,
                command,
            } => Ok(AsicWorkerCommand::Dispatch {
                generation,
                valid_jobs,
                command,
            }),
            ProductionSessionEffect::PollAsic {
                generation,
                valid_jobs,
                slice_ms,
            } => Ok(AsicWorkerCommand::Poll {
                generation,
                valid_jobs,
                slice_ms,
            }),
            other => Err(other),
        }
    }
}

fn elapsed_micros(started_at: Instant) -> u64 {
    u64::try_from(started_at.elapsed().as_micros()).unwrap_or(u64::MAX)
}

impl Drop for AsicWorker {
    fn drop(&mut self) {
        if self
            .sender
            .try_send((AsicWorkerCommand::Shutdown, revocation::stamp(None)))
            .is_err()
        {
            log::warn!("production_asic_worker_shutdown=degraded");
        }
    }
}
