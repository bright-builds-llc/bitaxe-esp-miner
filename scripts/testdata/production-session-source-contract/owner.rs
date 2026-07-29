use std::time::Duration;

const OWNER_STACK_BYTES: usize = 16 * 1024;
const NOTIFICATION_CAPACITY: usize = 8;
const AUTHORITATIVE_REREAD_INTERVAL: Duration = Duration::from_secs(1);

struct OrdinaryEspProductionSessionAdapter {
    maybe_campaign_lease: Option<()>,
    actuation_qualified: bool,
}

fn drive_owner() {
    let adapter_state = OrdinaryEspProductionSessionAdapter {
        maybe_campaign_lease: None,
        actuation_qualified: false,
    };
    sender.try_send(wakeup);
    ProductionSessionNotificationOutcome::Coalesced;
    receiver.recv_timeout(AUTHORITATIVE_REREAD_INTERVAL);
    ProductionSessionEvent::Wake;
    drive_session(&mut session, &mut adapter, event);
    adapter.maybe_execute(effect);
}
