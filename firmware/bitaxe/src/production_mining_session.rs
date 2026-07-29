//! Thin ESP owner and fail-closed adapter for the Production Mining Session.

use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use bitaxe_safety::observation::MonotonicMillis;
use bitaxe_stratum::v1::production_session::{
    ProductionMiningSession, ProductionReadiness, ProductionSessionEffect, ProductionSessionEvent,
    ProductionSessionNotificationOutcome, ProductionSessionWakeup,
};

const OWNER_STACK_BYTES: usize = 16 * 1024;
const NOTIFICATION_CAPACITY: usize = 8;
const AUTHORITATIVE_REREAD_INTERVAL: Duration = Duration::from_secs(1);

static NOTIFICATIONS: OnceLock<SyncSender<ProductionSessionWakeup>> = OnceLock::new();

/// Starts the single boot-lifetime production mining owner.
pub fn start() -> anyhow::Result<()> {
    let (sender, receiver) = mpsc::sync_channel(NOTIFICATION_CAPACITY);
    NOTIFICATIONS
        .set(sender)
        .map_err(|_| anyhow::anyhow!("production mining session already started"))?;

    std::thread::Builder::new()
        .name("production-mining-session".to_owned())
        .stack_size(OWNER_STACK_BYTES)
        .spawn(move || run_owner(receiver))
        .map(|_| ())
        .map_err(Into::into)
}

/// Non-blockingly wakes the owner with a category-only notification.
#[must_use]
pub fn notify(wakeup: ProductionSessionWakeup) -> ProductionSessionNotificationOutcome {
    let Some(sender) = NOTIFICATIONS.get() else {
        return ProductionSessionNotificationOutcome::OwnerUnavailable;
    };

    match sender.try_send(wakeup) {
        Ok(()) => ProductionSessionNotificationOutcome::Queued,
        Err(TrySendError::Full(_)) => ProductionSessionNotificationOutcome::Coalesced,
        Err(TrySendError::Disconnected(_)) => {
            ProductionSessionNotificationOutcome::OwnerUnavailable
        }
    }
}

fn run_owner(receiver: Receiver<ProductionSessionWakeup>) {
    let started_at = Instant::now();
    let mut session = ProductionMiningSession::new();
    let mut adapter = OrdinaryEspProductionSessionAdapter;

    loop {
        let maybe_wakeup = match receiver.recv_timeout(AUTHORITATIVE_REREAD_INTERVAL) {
            Ok(wakeup) => Some(wakeup),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Some(ProductionSessionWakeup::ShutdownRequested)
            }
        };
        let shutdown_requested = matches!(
            maybe_wakeup,
            Some(ProductionSessionWakeup::ShutdownRequested)
        );
        let now_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
        let event = ProductionSessionEvent::Wake {
            wakeup: maybe_wakeup,
            readiness: adapter.read_authoritative_readiness(),
            now_ms,
        };
        drive_session(&mut session, &mut adapter, event);
        if shutdown_requested {
            return;
        }
    }
}

fn drive_session(
    session: &mut ProductionMiningSession,
    adapter: &mut OrdinaryEspProductionSessionAdapter,
    initial_event: ProductionSessionEvent,
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
            if let Some(feedback) = adapter.maybe_execute(effect) {
                events.push_back(feedback);
            }
        }
    }
}

struct OrdinaryEspProductionSessionAdapter;

impl OrdinaryEspProductionSessionAdapter {
    fn read_authoritative_readiness(&mut self) -> ProductionReadiness {
        let mining = crate::runtime_snapshot::mining_runtime_state();
        let wifi = crate::wifi_adapter::current_wifi_snapshot();
        let observations = crate::safety_adapter::observation_snapshot();
        let safety_prerequisites_fresh = observations.is_ultra_205_mining_safe_at(now());
        ProductionReadiness {
            operator_intent: mining.operator_intent,
            network_ready: wifi.wifi_status == "connected",
            stratum_v1_supported: crate::settings_adapter::configured_protocol_is_v1(),
            safety_prerequisites_fresh,
            maybe_campaign_lease: None,
            actuation_qualified: false,
        }
    }

    fn maybe_execute(&mut self, effect: ProductionSessionEffect) -> Option<ProductionSessionEvent> {
        match effect {
            ProductionSessionEffect::Publish(snapshot) => {
                crate::runtime_snapshot::publish_production_session_snapshot(snapshot);
                None
            }
            ProductionSessionEffect::BlockSubmissions
            | ProductionSessionEffect::InvalidateWorkAndSubmissions
            | ProductionSessionEffect::StopAsicInteraction
            | ProductionSessionEffect::ClosePoolConnection(_) => None,
            ProductionSessionEffect::PrepareHardware { .. } => {
                Self::maybe_reject_safety_gated_effect(None, "hardware_prepare")
            }
            ProductionSessionEffect::ReadPoolConfiguration => {
                log::error!(
                    "production_mining_session=fail_closed reason=ordinary_adapter_unqualified action=pool_configuration"
                );
                Some(ProductionSessionEvent::PoolConfigurationLoaded(None))
            }
            ProductionSessionEffect::ConnectPool(pool) => {
                log::error!(
                    "production_mining_session=fail_closed reason=ordinary_adapter_unqualified action=connect pool={pool:?}"
                );
                Some(ProductionSessionEvent::TransportConnectFailed {
                    pool,
                    now_ms: crate::runtime_uptime::millis(),
                })
            }
            ProductionSessionEffect::WritePoolLine { pool, .. } => {
                Self::maybe_reject_effect(Some(pool), "pool_write")
            }
            ProductionSessionEffect::ApplyVersionMask(_) => {
                Self::maybe_reject_effect(None, "version_mask")
            }
            ProductionSessionEffect::DispatchAsic { .. } => {
                Self::maybe_reject_safety_gated_effect(None, "asic_dispatch")
            }
            ProductionSessionEffect::PollAsic { .. } => {
                Self::maybe_reject_effect(None, "asic_poll")
            }
            ProductionSessionEffect::SafeStopHardware { .. } => {
                Self::maybe_reject_effect(None, "hardware_safe_stop")
            }
        }
    }

    fn maybe_reject_safety_gated_effect(
        maybe_pool: Option<bitaxe_stratum::v1::production_session::ProductionPool>,
        action: &'static str,
    ) -> Option<ProductionSessionEvent> {
        if crate::safety_adapter::observation_snapshot().is_ultra_205_mining_safe_at(now()) {
            return Self::maybe_reject_effect(maybe_pool, action);
        }

        log::error!(
            "production_mining_session=fail_closed reason=safety_observations_not_ready action={action}"
        );
        Some(ProductionSessionEvent::EffectFailed {
            maybe_pool,
            reason: "safety_observations_not_ready",
            now_ms: crate::runtime_uptime::millis(),
        })
    }

    fn maybe_reject_effect(
        maybe_pool: Option<bitaxe_stratum::v1::production_session::ProductionPool>,
        action: &'static str,
    ) -> Option<ProductionSessionEvent> {
        log::error!(
            "production_mining_session=fail_closed reason=ordinary_adapter_unqualified action={action}"
        );
        Some(ProductionSessionEvent::EffectFailed {
            maybe_pool,
            reason: "ordinary_adapter_unqualified",
            now_ms: crate::runtime_uptime::millis(),
        })
    }
}

fn now() -> MonotonicMillis {
    MonotonicMillis::new(crate::runtime_uptime::millis())
}
