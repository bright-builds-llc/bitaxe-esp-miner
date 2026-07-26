//! Thin ESP owner for the production mining session.

use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use bitaxe_stratum::v1::production_session::{
    ProductionMiningSession, ProductionPoolAvailability, ProductionReadiness,
    ProductionSessionAction, ProductionSessionNotificationOutcome, ProductionSessionWakeup,
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
        let readiness = adapter.read_authoritative_readiness();
        let now_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
        let actions = session.on_wakeup(maybe_wakeup, readiness, now_ms);
        apply_actions(&mut session, &mut adapter, actions, now_ms);
        if shutdown_requested {
            return;
        }
    }
}

fn apply_actions(
    session: &mut ProductionMiningSession,
    adapter: &mut OrdinaryEspProductionSessionAdapter,
    actions: Vec<ProductionSessionAction>,
    now_ms: u64,
) {
    for action in actions {
        match action {
            ProductionSessionAction::ReadPoolConfiguration => {
                let maybe_availability = adapter.read_pool_configuration();
                let availability = maybe_availability.unwrap_or(ProductionPoolAvailability {
                    primary_configured: false,
                    fallback_configured: false,
                    prefer_fallback: false,
                });
                let follow_up = session.on_pool_configuration(availability);
                apply_actions(session, adapter, follow_up, now_ms);
            }
            ProductionSessionAction::ConnectPool(pool) => {
                log::error!(
                    "production_mining_session=fail_closed reason=ordinary_adapter_unqualified action=connect pool={pool:?}"
                );
                let follow_up = session.on_connection_result(pool, false, now_ms);
                apply_actions(session, adapter, follow_up, now_ms);
            }
            ProductionSessionAction::BlockSubmissions => adapter.block_submissions(),
            ProductionSessionAction::InvalidateWorkAndSubmissions => {
                adapter.invalidate_work_and_submissions();
            }
            ProductionSessionAction::StopAsicInteraction => adapter.stop_asic_interaction(),
            ProductionSessionAction::ClosePoolConnection => adapter.close_pool_connection(),
            ProductionSessionAction::Publish(projection) => {
                crate::runtime_snapshot::publish_production_session_projection(projection);
            }
        }
    }
}

struct OrdinaryEspProductionSessionAdapter;

impl OrdinaryEspProductionSessionAdapter {
    fn read_authoritative_readiness(&mut self) -> ProductionReadiness {
        let mining = crate::runtime_snapshot::mining_runtime_state();
        let wifi = crate::wifi_adapter::current_wifi_snapshot();
        ProductionReadiness {
            operator_intent: mining.operator_intent,
            network_ready: wifi.wifi_status == "connected",
            stratum_v1_supported: crate::settings_adapter::configured_protocol_is_v1(),
            safety_prerequisites_fresh: false,
            production_asic_ready: false,
            actuation_qualified: false,
        }
    }

    fn read_pool_configuration(&mut self) -> Option<ProductionPoolAvailability> {
        log::error!(
            "production_mining_session=fail_closed reason=pool_configuration_requested_by_unqualified_adapter"
        );
        None
    }

    fn block_submissions(&mut self) {}

    fn invalidate_work_and_submissions(&mut self) {}

    fn stop_asic_interaction(&mut self) {}

    fn close_pool_connection(&mut self) {}
}
