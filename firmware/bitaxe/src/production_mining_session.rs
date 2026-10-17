//! Thin ESP owner and qualified live-I/O adapter for the Production Mining Session.
pub(crate) mod admission_diagnostics;
mod asic_worker;
mod best_difficulty;
mod bwg;
mod campaign_status;
pub(crate) mod cooling;
pub(crate) mod cooling_core;
mod hashrate;
mod inbox;
mod live_hardware;
pub(crate) mod mining_progress;
mod notifications;
mod owner_loop;
mod owner_progress;
pub(crate) mod owner_resources;
mod pending_observation;
mod qualification;
mod readiness;
mod readiness_trace;
pub(crate) mod revocation;
mod scoreboard;
pub(crate) mod shutdown_budget;
mod status_publication;
mod transport;
pub(crate) use transport::borrow::NoiseBorrowHandle;
pub(crate) mod watchdog;
use self::asic_worker::{AsicWorker, AsicWorkerCommand, AsicWorkerEvent};
use self::campaign_status::publication::CampaignStatusPublicationSchedule;
use self::campaign_status::CampaignStatusTracker;
use self::hashrate::ProductionHashrateMonitor;
use self::owner_loop::{run_owner, safe_stop_subphase};
use self::readiness_trace::ReadinessTransitionTracker;
use self::transport::{PoolTransportCommand, PoolTransportEvent, PoolTransportWorkers};
use crate::mining_actuation::SafeShutdownStep;
use bitaxe_core::runtime_health::TaskWatchdogOwnerSubphase;
use bitaxe_core::runtime_orchestration::{PeriodicDeadline, PRODUCTION_REREAD_CADENCE_MS};
use bitaxe_runtime::queue::{self, Receiver, SyncSender};
use bitaxe_safety::observation::MonotonicMillis;
use bitaxe_stratum::v1::production_session::{
    AsicPollCompletion, ProductionAsicFailure, ProductionMiningSession, ProductionPool,
    ProductionReadiness, ProductionSessionEffect, ProductionSessionEvent,
    ProductionSessionSnapshot, ProductionSessionWakeup, ProductionTransportFailure,
};
use bitaxe_stratum::v1::production_work::ProductionNonceObservation;
pub(crate) use bwg::{
    cooling as bwg_cooling, renew as bwg_renew, safe_stop as bwg_safe_stop, start as bwg_start,
};
pub use notifications::notify;
pub(crate) use qualification::status_evidence;
use std::sync::mpsc::{self, TrySendError};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
const OWNER_STACK_BYTES: usize = 24 * 1024;
const NOTIFICATION_CAPACITY: usize = 16;
static NOTIFICATIONS: OnceLock<SyncSender<OwnerInboxMessage>> = OnceLock::new();
/// Who drives the fan controller: the campaign path, or one prepared Gate lease
/// (ADR-0037 decision 6). Cleared before every safe-stop, cooling and activation.
static FAN_CONTROLLER_OWNER: Mutex<Option<revocation::FanControllerOwner>> = Mutex::new(None);
pub(crate) fn fan_controller_owner() -> Option<revocation::FanControllerOwner> {
    *FAN_CONTROLLER_OWNER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
pub(super) fn set_fan_controller_owner(maybe_owner: Option<revocation::FanControllerOwner>) {
    *FAN_CONTROLLER_OWNER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = maybe_owner;
}
/// Ends idle fan control before preparation or the cooling review drives the fan,
/// so an idle write queued earlier is refused at the safety adapter (ADR-0038).
fn release_idle_fan() {
    revocation::withdraw_idle_fan();
    if matches!(
        fan_controller_owner(),
        Some(revocation::FanControllerOwner::Idle(_))
    ) {
        set_fan_controller_owner(None);
    }
}
enum OwnerInboxMessage {
    Wake(ProductionSessionWakeup),
    Transport(PoolTransportEvent),
    Asic(AsicWorkerEvent),
    Bwg(bwg::OwnerCommand),
}

/// Starts the single boot-lifetime production mining owner and its bounded I/O workers.
// Never inlined: its ~4.4 KiB frame must be released before the rest of startup runs. Inlined
// into `run_startup`, it stayed live under the Worker trust parse and overflowed the 16 KiB
// main task (queue-workaround boot loop, 2026-10-04).
#[inline(never)]
pub fn start() -> anyhow::Result<()> {
    let (sender, receiver) = queue::sync_channel(NOTIFICATION_CAPACITY);
    NOTIFICATIONS
        .set(sender.clone())
        .map_err(|_| anyhow::anyhow!("production mining session already started"))?;
    let adapter = OrdinaryEspProductionSessionAdapter::new(sender)?;
    std::thread::Builder::new()
        .name("production-mining-session".to_owned())
        .stack_size(OWNER_STACK_BYTES)
        .spawn(move || run_owner(receiver, adapter))
        .map(|_| ())
        .map_err(Into::into)
}
struct OrdinaryEspProductionSessionAdapter {
    mining_actuation: crate::mining_actuation_adapter::Ultra205MiningActuationAdapter,
    transports: PoolTransportWorkers,
    asic: AsicWorker,
    hashrate: ProductionHashrateMonitor,
    maybe_campaign_status: Option<CampaignStatusTracker>,
    campaign_status_publication: CampaignStatusPublicationSchedule,
    maybe_terminal_pool_persisted: Option<bool>,
    protocol_gate: crate::settings_adapter::ProductionProtocolGateDecision,
    readiness_trace: ReadinessTransitionTracker,
    maybe_bwg_session: Option<bwg::OwnerSession>,
    maybe_bwg_reply: Option<bwg::PendingReply>,
    maybe_cooling_generation: Option<revocation::WorkerGeneration>,
    live_hardware: live_hardware::LiveHardwareOwner,
}
impl OrdinaryEspProductionSessionAdapter {
    fn new(owner_sender: SyncSender<OwnerInboxMessage>) -> anyhow::Result<Self> {
        let transport_sender = owner_sender.clone();
        let transports = PoolTransportWorkers::spawn(move |event| {
            if transport_sender
                .send(OwnerInboxMessage::Transport(event))
                .is_err()
            {
                log::warn!("production_transport_event=discarded reason=owner_unavailable");
            }
        })?;
        let asic = AsicWorker::spawn(move |event| {
            if owner_sender.send(OwnerInboxMessage::Asic(event)).is_err() {
                log::warn!("production_asic_event=discarded reason=owner_unavailable");
            }
        })?;
        let maybe_campaign_status =
            match crate::settings_adapter::load_production_campaign_admission() {
                Ok(maybe_admission) => maybe_admission.map(|admission| {
                    CampaignStatusTracker::new(
                        admission.stage,
                        admission.maybe_lease,
                        admission.maybe_profile,
                    )
                }),
                Err(error) => {
                    log::warn!("production_campaign=blocked category={}", error.category());
                    None
                }
            };
        if maybe_campaign_status
            .as_ref()
            .is_some_and(CampaignStatusTracker::requires_requested_run_bootstrap)
        {
            crate::runtime_snapshot::apply_command_effects_run_bootstrap();
        }

        Ok(Self {
            mining_actuation: crate::mining_actuation_adapter::Ultra205MiningActuationAdapter::new(
            ),
            transports,
            asic,
            hashrate: ProductionHashrateMonitor::new(),
            maybe_campaign_status,
            campaign_status_publication: CampaignStatusPublicationSchedule::new(),
            maybe_terminal_pool_persisted: None,
            protocol_gate:
                crate::settings_adapter::ProductionProtocolGateDecision::PartitionOwnerUnavailable,
            readiness_trace: ReadinessTransitionTracker::default(),
            maybe_bwg_session: None,
            maybe_bwg_reply: None,
            maybe_cooling_generation: None,
            live_hardware: live_hardware::LiveHardwareOwner::default(),
        })
    }

    fn wake_event(
        &mut self,
        wakeup: Option<ProductionSessionWakeup>,
        now_ms: u64,
        snapshot: &ProductionSessionSnapshot,
        pending_observation_recovered: bool,
    ) -> ProductionSessionEvent {
        ProductionSessionEvent::Wake {
            wakeup,
            readiness: self.read_authoritative_readiness(
                wakeup,
                snapshot,
                pending_observation_recovered,
            ),
            now_ms,
        }
    }

    // Keep a distinct inbox-mapping boundary for native code-size and stack audits.

    fn maybe_execute(
        &mut self,
        effect: ProductionSessionEffect,
        now_ms: u64,
        progress: &mut dyn FnMut(TaskWatchdogOwnerSubphase),
    ) -> Option<ProductionSessionEvent> {
        match effect {
            ProductionSessionEffect::Publish(snapshot) => {
                if let Some(session) = self.maybe_bwg_session.as_ref() {
                    mining_progress::capture(
                        session.generation.raw(),
                        &snapshot,
                        session.expected_filter_counts,
                    );
                    let [accepted, rejected, qualified] = session.publication_counts(&snapshot);
                    revocation::publish_counts(session.generation, accepted, rejected, qualified);
                }
                if let Some(status) = self.maybe_campaign_status.as_mut() {
                    status.note_snapshot(&snapshot, now_ms);
                    let qualified = status.authorizes_actuation()
                        && self.maybe_bwg_session.is_none()
                        && revocation::permits(
                            self.maybe_bwg_session
                                .as_ref()
                                .map(|session| session.generation),
                        )
                        && snapshot.campaign_state
                            == bitaxe_stratum::v1::production_session::MiningCampaignState::Active
                        && crate::safety_adapter::safety_actuation_available()
                        && crate::asic_adapter::production::production_handle_available();
                    // A Gate lease owns the fan from its preparation until its safe-stop.
                    revocation::withdraw_idle_fan();
                    if self.maybe_bwg_session.is_none() {
                        set_fan_controller_owner(
                            qualified.then_some(revocation::FanControllerOwner::Campaign),
                        );
                    }
                } else if revocation::permits(None)
                    && cooling_core::baseline_safe(
                        &crate::safety_adapter::observation_snapshot(),
                        now_ms,
                    )
                {
                    // No lease is reserved, active or shutting down, and the chip has
                    // cooled to the baseline, so upstream's idle duties may lower the
                    // fan (ADR-0038). A Start that never prepared leaves the cooling
                    // review's 100% until this holds.
                    set_fan_controller_owner(Some(revocation::FanControllerOwner::Idle(
                        revocation::grant_idle_fan(),
                    )));
                } else {
                    revocation::withdraw_idle_fan();
                    if self.maybe_bwg_session.is_none() {
                        set_fan_controller_owner(None);
                    }
                }
                crate::runtime_snapshot::publish_production_session_snapshot(*snapshot);
                None
            }
            ProductionSessionEffect::BlockSubmissions
            | ProductionSessionEffect::InvalidateWorkAndSubmissions => {
                revocation::block_work();
                None
            }
            ProductionSessionEffect::StopAsicInteraction => {
                if crate::asic_adapter::production::block_production_dispatch().is_err() {
                    return Some(ProductionSessionEvent::EffectFailed {
                        maybe_pool: None,
                        reason: "asic_stop_failed",
                        now_ms,
                    });
                }
                None
            }
            ProductionSessionEffect::PrepareHardware { lease_id, profile } => {
                release_idle_fan();
                self.note_worker_preparation_started();
                self.mining_actuation.set_worker_generation(
                    self.maybe_bwg_session
                        .as_ref()
                        .map(|session| session.generation),
                );
                match self.mining_actuation.prepare(profile) {
                    Ok(()) if revocation::permits(self.maybe_bwg_session.as_ref().map(|session| session.generation)) => {
                        // Preparation proved the fan at 100%; the lease's controller now
                        // follows the device's fan settings until its permit ends.
                        if let Some(session) = self.maybe_bwg_session.as_ref() {
                            set_fan_controller_owner(Some(revocation::FanControllerOwner::Lease(
                                session.generation,
                            )));
                            self.live_hardware.prepared(profile);
                        }
                        Some(ProductionSessionEvent::HardwarePrepared { lease_id, now_ms: crate::runtime_uptime::millis() })
                    }
                    Ok(()) => Some(ProductionSessionEvent::HardwarePreparationFailed { lease_id,
                        failure: bitaxe_stratum::v1::production_session::HardwarePreparationFailure::Rejected,
                        now_ms: crate::runtime_uptime::millis() }),
                    Err(failure) => {
                        if self.maybe_bwg_session.is_some() {
                            admission_diagnostics::fail(admission_diagnostics::Failure::Preparation);
                        }
                        let original = failure.original();
                        let original_category = original.source().category();
                        let (rollback_step, rollback_detail) = failure
                            .maybe_safe_shutdown_failure()
                            .map_or(("none", "none"), |rollback| {
                                (rollback.step().label(), rollback.source().category())
                            });
                        if let Some(status) = self.maybe_campaign_status.as_mut() {
                            status.note_failure(
                                "hardware_preparation",
                                original.step().label(),
                                original_category,
                                rollback_step,
                                rollback_detail,
                            );
                        }
                        if let Some(rollback) = failure.maybe_safe_shutdown_failure() {
                            log::error!(
                                "production_mining_session=fail_closed action=hardware_prepare original_step={:?} original_category={original_category} rollback_step={:?} rollback_category={}",
                                original.step(),
                                rollback.step(),
                                rollback.source().category(),
                            );
                        } else {
                            log::error!(
                                "production_mining_session=fail_closed action=hardware_prepare original_step={:?} original_category={original_category}",
                                original.step(),
                            );
                        }
                        Some(ProductionSessionEvent::HardwarePreparationFailed {
                            lease_id,
                            failure: original.source().hardware_preparation_failure(),
                            now_ms,
                        })
                    }
                }
            }
            ProductionSessionEffect::ReadPoolConfiguration => {
                if self.maybe_bwg_session.is_some() {
                    admission_diagnostics::stage(admission_diagnostics::Stage::PoolActivation);
                }
                let maybe_pools = if let Some(session) = self.maybe_bwg_session.as_ref() {
                    Some(session.pools.clone())
                } else {
                    match crate::settings_adapter::read_production_pool_set() {
                        Ok(maybe_pools) => maybe_pools,
                        Err(error) => {
                            log::warn!(
                                "production_pool_configuration=unavailable category={}",
                                error.category()
                            );
                            None
                        }
                    }
                };
                if let Some(status) = self.maybe_campaign_status.as_mut() {
                    status.note_pool_configuration_read(maybe_pools.is_some());
                }
                Some(ProductionSessionEvent::PoolConfigurationLoaded(
                    maybe_pools.map(Box::new),
                ))
            }
            ProductionSessionEffect::ConnectPool {
                pool,
                transport_epoch,
                endpoint,
                protocol,
                maybe_pool_generation,
            } => {
                let command = match protocol {
                    bitaxe_stratum::v1::production_session::ProductionProtocolConfig::V1(_) => {
                        PoolTransportCommand::Connect {
                            transport_epoch,
                            endpoint,
                        }
                    }
                    bitaxe_stratum::v1::production_session::ProductionProtocolConfig::V2(
                        config,
                    ) => {
                        let Some(generation) = maybe_pool_generation else {
                            return Some(ProductionSessionEvent::TransportFailed {
                                pool,
                                transport_epoch,
                                failure: ProductionTransportFailure::Connect,
                                now_ms,
                            });
                        };
                        let Some(ipv4) = bitaxe_worker_control::v2::private_ipv4(&endpoint.host)
                        else {
                            return Some(ProductionSessionEvent::TransportFailed {
                                pool,
                                transport_epoch,
                                failure: ProductionTransportFailure::Connect,
                                now_ms,
                            });
                        };
                        PoolTransportCommand::ConnectV2 {
                            transport_epoch,
                            endpoint: std::net::SocketAddrV4::new(ipv4, endpoint.port),
                            authority: config.authority,
                            generation,
                            permit: revocation::stamp(
                                self.maybe_bwg_session.as_ref().map(|s| s.generation),
                            ),
                        }
                    }
                };
                self.try_send_transport(
                    pool,
                    transport_epoch,
                    ProductionTransportFailure::Connect,
                    command,
                    now_ms,
                )
            }
            ProductionSessionEffect::RecordV2Failure {
                message_type,
                reason,
            } => {
                crate::v2_serial_runtime::protocol_rejected(message_type, reason);
                None
            }
            ProductionSessionEffect::RecordV2Frame { frame } => {
                crate::v2_serial_runtime::validated_frame(&frame);
                None
            }
            ProductionSessionEffect::V2WorkReady { work, commitment } => {
                crate::v2_serial_runtime::work_ready(work, commitment);
                None
            }
            ProductionSessionEffect::WritePoolFrame {
                pool,
                transport_epoch,
                frame,
            } => self.try_send_transport(
                pool,
                transport_epoch,
                ProductionTransportFailure::Write,
                PoolTransportCommand::WriteFrame {
                    transport_epoch,
                    frame,
                    permit: revocation::stamp(
                        self.maybe_bwg_session.as_ref().map(|s| s.generation),
                    ),
                },
                now_ms,
            ),
            ProductionSessionEffect::WritePoolLine {
                pool,
                transport_epoch,
                line,
            } => self.try_send_transport(
                pool,
                transport_epoch,
                ProductionTransportFailure::Write,
                PoolTransportCommand::Write {
                    transport_epoch,
                    line,
                    permit: revocation::stamp(
                        self.maybe_bwg_session
                            .as_ref()
                            .map(|session| session.generation),
                    ),
                },
                now_ms,
            ),
            effect @ (ProductionSessionEffect::ApplyVersionMask { .. }
            | ProductionSessionEffect::DispatchAsic { .. }
            | ProductionSessionEffect::PollAsic { .. }) => {
                let command = AsicWorker::command_from_effect(effect).expect("matched ASIC effect");
                self.try_send_asic(command, now_ms)
            }
            ProductionSessionEffect::ClosePoolConnection {
                pool,
                transport_epoch,
            } => match self.transports.request_close(pool, transport_epoch) {
                Ok(()) => None,
                Err(TrySendError::Disconnected(_) | TrySendError::Full(_)) => {
                    Some(ProductionSessionEvent::TransportFailed {
                        pool,
                        transport_epoch,
                        failure: ProductionTransportFailure::Write,
                        now_ms,
                    })
                }
            },
            ProductionSessionEffect::SafeStopHardware { lease_id, purpose } => {
                if self.maybe_bwg_session.is_some() {
                    admission_diagnostics::fail_current();
                    admission_diagnostics::stage(admission_diagnostics::Stage::Cleanup);
                }
                if let Some(session) = self.maybe_bwg_session.as_ref() {
                    revocation::revoke_reason_at(
                        session.generation,
                        crate::runtime_uptime::millis(),
                        revocation::RevocationReason::ControlFailed,
                    );
                }
                release_idle_fan();
                set_fan_controller_owner(None);
                self.live_hardware.stopped();
                if let Some(status) = self.maybe_campaign_status.as_mut() {
                    status.note_safe_stop_pending();
                }
                let mut safe_stop_progress = |step| progress(safe_stop_subphase(step));
                match self
                    .mining_actuation
                    .safe_stop(purpose, &mut safe_stop_progress)
                {
                    Ok(()) => {
                        Some(ProductionSessionEvent::HardwareSafeStopConfirmed { lease_id, now_ms })
                    }
                    Err(failure) => {
                        admission_diagnostics::fail(admission_diagnostics::Failure::Cleanup);
                        log::error!(
                            "production_mining_session=fail_closed action=hardware_safe_stop failed_step={:?} category={}",
                            failure.step(),
                            failure.source().category(),
                        );
                        Some(ProductionSessionEvent::EffectFailed {
                            maybe_pool: None,
                            reason: "hardware_safe_stop_failed",
                            now_ms,
                        })
                    }
                }
            }
            ProductionSessionEffect::RecordScoreboard { candidate } => {
                if let Some(session) = self.maybe_bwg_session.as_mut() {
                    mining_progress::note_expected_filter(
                        &mut session.expected_filter_counts,
                        candidate.matches_expected_asic_filter(),
                    );
                }
                best_difficulty::record(candidate.difficulty());
                scoreboard::record(candidate)
            }
            ProductionSessionEffect::RecordBlockFound => {
                crate::runtime_snapshot::record_found_block();
                None
            }
        }
    }

    fn try_send_transport(
        &self,
        pool: ProductionPool,
        transport_epoch: bitaxe_stratum::v1::production_session::ProductionTransportEpoch,
        failure: ProductionTransportFailure,
        command: PoolTransportCommand,
        now_ms: u64,
    ) -> Option<ProductionSessionEvent> {
        match self.transports.try_send(pool, command) {
            Ok(()) => None,
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                Some(ProductionSessionEvent::TransportFailed {
                    pool,
                    transport_epoch,
                    failure,
                    now_ms,
                })
            }
        }
    }

    fn try_send_asic(
        &self,
        command: AsicWorkerCommand,
        now_ms: u64,
    ) -> Option<ProductionSessionEvent> {
        let generation = match &command {
            AsicWorkerCommand::ApplyVersionMask { generation, .. }
            | AsicWorkerCommand::Dispatch { generation, .. }
            | AsicWorkerCommand::Poll { generation, .. }
            | AsicWorkerCommand::ReadHashrateRegisters { generation } => *generation,
            AsicWorkerCommand::Shutdown => return None,
        };
        match self.asic.try_send(
            command,
            self.maybe_bwg_session
                .as_ref()
                .map(|session| session.generation),
        ) {
            Ok(()) => None,
            Err(TrySendError::Full(_)) => Some(ProductionSessionEvent::AsicInteractionFailed {
                generation,
                failure: ProductionAsicFailure::QueueFull,
                now_ms,
            }),
            Err(TrySendError::Disconnected(_)) => {
                Some(ProductionSessionEvent::AsicInteractionFailed {
                    generation,
                    failure: ProductionAsicFailure::WorkerDisconnected,
                    now_ms,
                })
            }
        }
    }
}

fn now() -> MonotonicMillis {
    MonotonicMillis::new(crate::runtime_uptime::millis())
}

fn elapsed_millis(started_at: Instant) -> u64 {
    u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX)
}
