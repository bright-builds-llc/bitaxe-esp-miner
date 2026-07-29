use std::time::Duration;

mod asic_worker;
mod transport;

const OWNER_STACK_BYTES: usize = 16 * 1024;
const NOTIFICATION_CAPACITY: usize = 16;
const AUTHORITATIVE_REREAD_INTERVAL: Duration = Duration::from_secs(1);

enum OwnerInboxMessage {}

struct OrdinaryEspProductionSessionAdapter;

fn drive_owner() {
    sender.try_send(OwnerInboxMessage::Wake(wakeup));
    ProductionSessionNotificationOutcome::Coalesced;
    receiver.recv_timeout(AUTHORITATIVE_REREAD_INTERVAL);
    adapter.event_from_inbox(message, now_ms);
    drive_session(&mut session, &mut adapter, event, now_ms);
    adapter.maybe_execute(effect, now_ms);
}
