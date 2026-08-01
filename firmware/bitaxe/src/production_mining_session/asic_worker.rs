//! Single bounded worker around the retained BM1366 production executor.

use std::sync::mpsc::{self, SyncSender, TrySendError};

use bitaxe_asic::bm1366::{
    command::VersionMask,
    production::Bm1366ProductionCommand,
    result::{Bm1366NonceResult, Bm1366ValidJobIds},
};
use bitaxe_stratum::v1::production_session::AsicPollCompletion;
use bitaxe_stratum::v1::production_session::{ProductionAsicFailure, ProductionSessionEffect};
use bitaxe_stratum::v1::production_work::PoolSessionGeneration;

use crate::asic_adapter::production::{
    apply_negotiated_version_mask, ProductionAsicExecutor, ProductionReadOutcome,
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
    sender: SyncSender<AsicWorkerCommand>,
}

impl AsicWorker {
    pub(super) fn spawn(emit: impl Fn(AsicWorkerEvent) + Send + 'static) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(COMMAND_CAPACITY);
        std::thread::Builder::new()
            .name("production-asic".to_owned())
            .stack_size(WORKER_STACK_BYTES)
            .spawn(move || {
                let mut executor = ProductionAsicExecutor::new();
                while let Ok(command) = receiver.recv() {
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
                        } => match executor.maybe_execute(command, &valid_jobs) {
                            Ok(Some(result)) => {
                                emit(AsicWorkerEvent::Result { generation, result });
                            }
                            Ok(None) => {}
                            Err(_) => emit(AsicWorkerEvent::Failed {
                                generation,
                                failure: ProductionAsicFailure::Dispatch,
                            }),
                        },
                        AsicWorkerCommand::Poll {
                            generation,
                            valid_jobs,
                            slice_ms,
                        } => match executor.try_read_production_result(&valid_jobs, slice_ms) {
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
                            Ok(ProductionReadOutcome::RegisterReadProof(_)) => {
                                emit(AsicWorkerEvent::PollCompleted {
                                    generation,
                                    completion: AsicPollCompletion::RegisterRead,
                                });
                            }
                            Err(_) => emit(AsicWorkerEvent::Failed {
                                generation,
                                failure: ProductionAsicFailure::Poll,
                            }),
                        },
                        AsicWorkerCommand::Shutdown => return,
                    }
                }
            })?;
        Ok(Self { sender })
    }

    pub(super) fn try_send(
        &self,
        command: AsicWorkerCommand,
    ) -> Result<(), TrySendError<AsicWorkerCommand>> {
        self.sender.try_send(command)
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

impl Drop for AsicWorker {
    fn drop(&mut self) {
        if self.sender.try_send(AsicWorkerCommand::Shutdown).is_err() {
            log::warn!("production_asic_worker_shutdown=degraded");
        }
    }
}
