//! Thin ESP owner and qualified live-I/O adapter for the Production Mining Session.

mod asic_worker;
mod campaign_status;
mod hashrate;
mod transport;
pub(crate) mod watchdog;

use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use bitaxe_core::runtime_orchestration::{PeriodicDeadline, PRODUCTION_REREAD_CADENCE_MS};
use bitaxe_safety::observation::{MonotonicMillis, Observation};
use bitaxe_safety::power::POWER_SAMPLE_STALE_AFTER_MS;
use bitaxe_stratum::v1::production_session::{
    AsicPollCompletion, ProductionAsicFailure, ProductionMiningSession, ProductionPool,
    ProductionReadiness, ProductionSessionEffect, ProductionSessionEvent,
    ProductionSessionNotificationOutcome, ProductionSessionSnapshot, ProductionSessionWakeup,
    ProductionTransportFailure,
};
use bitaxe_stratum::v1::production_work::ProductionNonceObservation;

use self::asic_worker::{AsicWorker, AsicWorkerCommand, AsicWorkerEvent};
use self::campaign_status::{CampaignObservationFreshness, CampaignStatusTracker};
use self::hashrate::ProductionHashrateMonitor;
use self::transport::{PoolTransportCommand, PoolTransportEvent, PoolTransportWorkers};

const OWNER_STACK_BYTES: usize = 16 * 1024;
const NOTIFICATION_CAPACITY: usize = 16;
static NOTIFICATIONS: OnceLock<SyncSender<OwnerInboxMessage>> = OnceLock::new();

enum OwnerInboxMessage {
    Wake(ProductionSessionWakeup),
    Transport(PoolTransportEvent),
    Asic(AsicWorkerEvent),
}

/// Starts the single boot-lifetime production mining owner and its bounded I/O workers.
pub fn start() -> anyhow::Result<()> {
    let (sender, receiver) = mpsc::sync_channel(NOTIFICATION_CAPACITY);
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

/// Non-blockingly wakes the owner with a category-only notification.
#[must_use]
pub fn notify(wakeup: ProductionSessionWakeup) -> ProductionSessionNotificationOutcome {
    let Some(sender) = NOTIFICATIONS.get() else {
        return ProductionSessionNotificationOutcome::OwnerUnavailable;
    };

    match sender.try_send(OwnerInboxMessage::Wake(wakeup)) {
        Ok(()) => ProductionSessionNotificationOutcome::Queued,
        Err(TrySendError::Full(_)) => ProductionSessionNotificationOutcome::Coalesced,
        Err(TrySendError::Disconnected(_)) => {
            ProductionSessionNotificationOutcome::OwnerUnavailable
        }
    }
}

fn run_owner(
    receiver: Receiver<OwnerInboxMessage>,
    mut adapter: OrdinaryEspProductionSessionAdapter,
) {
    let started_at = Instant::now();
    let mut session = ProductionMiningSession::new();
    let mut task_watchdog =
        watchdog::ProductionTaskWatchdog::subscribe(crate::runtime_uptime::millis());
    let mut readiness_schedule = PeriodicDeadline::new(0, PRODUCTION_REREAD_CADENCE_MS)
        .expect("production reread cadence is nonzero");

    loop {
        let before_wait_ms = elapsed_millis(started_at);
        let wait = Duration::from_millis(
            readiness_schedule
                .next_deadline_ms()
                .saturating_sub(before_wait_ms),
        );
        let maybe_message = match receiver.recv_timeout(wait) {
            Ok(message) => Some(message),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => Some(OwnerInboxMessage::Wake(
                ProductionSessionWakeup::ShutdownRequested,
            )),
        };
        let shutdown_requested = matches!(
            maybe_message,
            Some(OwnerInboxMessage::Wake(
                ProductionSessionWakeup::ShutdownRequested
            ))
        );
        let now_ms = elapsed_millis(started_at);
        let message_reads_readiness = matches!(maybe_message, Some(OwnerInboxMessage::Wake(_)));
        if let Some(message) = maybe_message {
            let event = adapter.event_from_inbox(message, now_ms);
            drive_session(&mut session, &mut adapter, event, now_ms);
        }
        if readiness_schedule.is_due(now_ms) {
            if !message_reads_readiness {
                let event = adapter.wake_event(None, now_ms);
                drive_session(&mut session, &mut adapter, event, now_ms);
            }
            if readiness_schedule.advance_past(now_ms).is_err() {
                log::error!(
                    "production_mining_session=fail_closed reason=readiness_deadline_overflow"
                );
                let event =
                    adapter.wake_event(Some(ProductionSessionWakeup::ShutdownRequested), now_ms);
                drive_session(&mut session, &mut adapter, event, now_ms);
                adapter.publish_campaign_status(&session.snapshot(), now_ms);
                return;
            }
        }
        adapter.publish_campaign_status(&session.snapshot(), now_ms);
        adapter.service_hashrate_monitor(&session.snapshot(), now_ms);
        task_watchdog.feed(crate::runtime_uptime::millis());
        if shutdown_requested {
            return;
        }
    }
}

fn drive_session(
    session: &mut ProductionMiningSession,
    adapter: &mut OrdinaryEspProductionSessionAdapter,
    initial_event: ProductionSessionEvent,
    now_ms: u64,
) {
    let mut events = VecDeque::from([initial_event]);
    while let Some(event) = events.pop_front() {
        let effects = match session.handle(event) {
            Ok(effects) => effects,
            Err(error) => {
                log::error!(
                    "production_mining_session=fail_closed reason=engine_error error={error}"
                );
                return;
            }
        };
        for effect in effects {
            if let Some(feedback) = adapter.maybe_execute(effect, now_ms) {
                events.push_back(feedback);
            }
        }
    }
}

struct OrdinaryEspProductionSessionAdapter {
    mining_actuation: crate::mining_actuation_adapter::Ultra205MiningActuationAdapter,
    transports: PoolTransportWorkers,
    asic: AsicWorker,
    hashrate: ProductionHashrateMonitor,
    maybe_campaign_status: Option<CampaignStatusTracker>,
    maybe_terminal_pool_persisted: Option<bool>,
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

        Ok(Self {
            mining_actuation: crate::mining_actuation_adapter::Ultra205MiningActuationAdapter::new(
            ),
            transports,
            asic,
            hashrate: ProductionHashrateMonitor::new(),
            maybe_campaign_status,
            maybe_terminal_pool_persisted: None,
        })
    }

    fn wake_event(
        &mut self,
        wakeup: Option<ProductionSessionWakeup>,
        now_ms: u64,
    ) -> ProductionSessionEvent {
        ProductionSessionEvent::Wake {
            wakeup,
            readiness: self.read_authoritative_readiness(),
            now_ms,
        }
    }

    fn event_from_inbox(
        &mut self,
        message: OwnerInboxMessage,
        now_ms: u64,
    ) -> ProductionSessionEvent {
        match message {
            OwnerInboxMessage::Wake(wakeup) => self.wake_event(Some(wakeup), now_ms),
            OwnerInboxMessage::Transport(event) => match event {
                PoolTransportEvent::Connected {
                    pool,
                    transport_epoch,
                } => ProductionSessionEvent::TransportConnected {
                    pool,
                    transport_epoch,
                    now_ms,
                },
                PoolTransportEvent::Failed {
                    pool,
                    transport_epoch,
                    failure,
                } => ProductionSessionEvent::TransportFailed {
                    pool,
                    transport_epoch,
                    failure,
                    now_ms,
                },
                PoolTransportEvent::Bytes {
                    pool,
                    transport_epoch,
                    bytes,
                } => ProductionSessionEvent::TransportBytes {
                    pool,
                    transport_epoch,
                    bytes,
                    now_ms,
                },
                PoolTransportEvent::Closed {
                    pool,
                    transport_epoch,
                } => ProductionSessionEvent::TransportClosed {
                    pool,
                    transport_epoch,
                    now_ms,
                },
            },
            OwnerInboxMessage::Asic(event) => match event {
                AsicWorkerEvent::Result { generation, result } => {
                    ProductionSessionEvent::AsicResult {
                        observation: ProductionNonceObservation {
                            observed_generation: generation,
                            result,
                        },
                        now_ms,
                    }
                }
                AsicWorkerEvent::PollTimedOut { generation } => {
                    ProductionSessionEvent::AsicPollTimedOut { generation, now_ms }
                }
                AsicWorkerEvent::PollCompleted {
                    generation,
                    completion,
                } => ProductionSessionEvent::AsicPollCompleted {
                    generation,
                    completion,
                    now_ms,
                },
                AsicWorkerEvent::RegisterRead {
                    generation,
                    read,
                    observed_at_us,
                } => {
                    self.hashrate.observe(read, observed_at_us);
                    ProductionSessionEvent::AsicPollCompleted {
                        generation,
                        completion: AsicPollCompletion::RegisterRead,
                        now_ms,
                    }
                }
                AsicWorkerEvent::Failed {
                    generation,
                    failure,
                } => ProductionSessionEvent::AsicInteractionFailed {
                    generation,
                    failure,
                    now_ms,
                },
            },
        }
    }

    fn read_authoritative_readiness(&mut self) -> ProductionReadiness {
        let mining = crate::runtime_snapshot::mining_runtime_state();
        let wifi = crate::wifi_adapter::current_wifi_snapshot();
        let observations = crate::safety_adapter::observation_snapshot();
        let safety_prerequisites_fresh = observations.is_ultra_205_mining_safe_at(now());
        let maybe_campaign_lease = self
            .maybe_campaign_status
            .as_ref()
            .and_then(CampaignStatusTracker::maybe_lease);
        let operator_intent = self
            .maybe_campaign_status
            .as_ref()
            .map_or(mining.operator_intent, |status| {
                status.operator_intent(mining.operator_intent)
            });
        let actuation_qualified = self
            .maybe_campaign_status
            .as_ref()
            .is_some_and(CampaignStatusTracker::authorizes_actuation)
            && crate::safety_adapter::safety_actuation_available()
            && crate::asic_adapter::production::production_handle_available();
        ProductionReadiness {
            operator_intent,
            network_ready: wifi.wifi_status == "connected",
            stratum_v1_supported: crate::settings_adapter::configured_protocol_is_v1(),
            safety_prerequisites_fresh,
            maybe_campaign_lease,
            actuation_qualified,
        }
    }

    fn maybe_execute(
        &mut self,
        effect: ProductionSessionEffect,
        now_ms: u64,
    ) -> Option<ProductionSessionEvent> {
        match effect {
            ProductionSessionEffect::Publish(snapshot) => {
                if let Some(status) = self.maybe_campaign_status.as_mut() {
                    status.note_snapshot(&snapshot, now_ms);
                }
                crate::runtime_snapshot::publish_production_session_snapshot(*snapshot);
                None
            }
            ProductionSessionEffect::BlockSubmissions
            | ProductionSessionEffect::InvalidateWorkAndSubmissions => None,
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
                match self.mining_actuation.prepare(profile) {
                    Ok(()) => Some(ProductionSessionEvent::HardwarePrepared { lease_id, now_ms }),
                    Err(failure) => {
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
                let maybe_pools = match crate::settings_adapter::read_production_pool_set() {
                    Ok(maybe_pools) => maybe_pools,
                    Err(error) => {
                        log::warn!(
                            "production_pool_configuration=unavailable category={}",
                            error.category()
                        );
                        None
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
            } => self.try_send_transport(
                pool,
                transport_epoch,
                ProductionTransportFailure::Connect,
                PoolTransportCommand::Connect {
                    transport_epoch,
                    endpoint,
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
            ProductionSessionEffect::SafeStopHardware { lease_id } => {
                if let Some(status) = self.maybe_campaign_status.as_mut() {
                    status.note_safe_stop_pending();
                }
                match self.mining_actuation.safe_stop() {
                    Ok(()) => {
                        Some(ProductionSessionEvent::HardwareSafeStopConfirmed { lease_id, now_ms })
                    }
                    Err(failure) => {
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
        match self.asic.try_send(command) {
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

    fn publish_campaign_status(
        &mut self,
        snapshot: &bitaxe_stratum::v1::production_session::ProductionSessionSnapshot,
        now_ms: u64,
    ) {
        if self.maybe_campaign_status.is_none() {
            return;
        }
        if snapshot.campaign_state
            == bitaxe_stratum::v1::production_session::MiningCampaignState::Consumed
            && self.maybe_terminal_pool_persisted.is_none()
        {
            self.maybe_terminal_pool_persisted = Some(matches!(
                crate::settings_adapter::read_production_pool_set(),
                Ok(Some(_))
            ));
        }
        let pool_config_persisted = self.maybe_terminal_pool_persisted.unwrap_or(false);
        let Some(status) = self.maybe_campaign_status.as_ref() else {
            return;
        };
        let observations = crate::safety_adapter::observation_snapshot();
        let safety_now = now();
        let safety_fresh = observations.is_ultra_205_mining_safe_at(safety_now);
        let observation_freshness = CampaignObservationFreshness {
            power_watts: is_current(&observations.power_watts, safety_now),
            bus_voltage_volts: is_current(&observations.bus_voltage_volts, safety_now),
            current_amps: is_current(&observations.current_amps, safety_now),
            chip_temp_celsius: is_current(&observations.chip_temp_celsius, safety_now),
            vr_temp_celsius: is_current(&observations.vr_temp_celsius, safety_now),
            fan_rpm: is_current(&observations.fan_rpm, safety_now),
        };
        let marker = status.marker(
            snapshot,
            now_ms,
            safety_fresh,
            observation_freshness,
            crate::settings_adapter::start_mining_on_boot(),
            pool_config_persisted,
        );
        crate::info_retained(&format!("mining_campaign_status={marker}"));
    }

    fn service_hashrate_monitor(&mut self, snapshot: &ProductionSessionSnapshot, now_ms: u64) {
        let Ok(maybe_tick) = self.hashrate.service_snapshot(snapshot, now_ms) else {
            log::warn!("hashrate_monitor=unavailable category=schedule_overflow");
            return;
        };
        let Some(tick) = maybe_tick else { return };
        crate::runtime_snapshot::publish_hashrate_snapshot(tick.snapshot);
        if tick.request_registers
            && self
                .asic
                .try_send(AsicWorkerCommand::ReadHashrateRegisters {
                    generation: snapshot.generation,
                })
                .is_err()
        {
            log::warn!("hashrate_monitor_read=skipped category=worker_unavailable");
        }
    }
}

fn is_current<T>(observation: &Observation<T>, now: MonotonicMillis) -> bool {
    observation.is_fresh()
        && observation.maybe_last_good().is_some_and(|sample| {
            now.get().saturating_sub(sample.acquired_at().get())
                <= u64::from(POWER_SAMPLE_STALE_AFTER_MS)
        })
}

fn now() -> MonotonicMillis {
    MonotonicMillis::new(crate::runtime_uptime::millis())
}

fn elapsed_millis(started_at: Instant) -> u64 {
    u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX)
}
