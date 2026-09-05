use super::*;
use bitaxe_stratum::v1::production_session::{
    LivePoolCredentials, LiveRuntimeConfig, MiningCampaignLease, MiningCampaignLeaseId,
    MiningCampaignMonotonicDeadline, MiningCampaignState, MiningCampaignStopCondition,
    MiningHardwareProfilePreset, MiningHardwareState, ProductionPoolConfiguration,
    ProductionPoolEndpoint, ProductionPoolSet,
};
use bitaxe_worker_control::{LeaseDeadlines, WorkerLeaseGrant, WorkerLeaseRenewal};

pub(super) struct OwnerSession {
    pub(super) generation: revocation::WorkerGeneration,
    pub(super) worker_lease_id: String,
    pub(super) lease: MiningCampaignLease,
    pub(super) pools: ProductionPoolSet,
    pub(super) accepted_baseline: u64,
    pub(super) rejected_baseline: u64,
    pub(super) correlated_baseline: u64,
}

pub(super) enum OwnerCommand {
    Start {
        generation: revocation::WorkerGeneration,
        worker_lease_id: String,
        deadline: MiningCampaignMonotonicDeadline,
        pools: ProductionPoolSet,
        reply: SyncSender<Result<(), Error>>,
    },
    Renew {
        generation: revocation::WorkerGeneration,
        worker_lease_id: String,
        deadline: MiningCampaignMonotonicDeadline,
        reply: SyncSender<Result<(), Error>>,
    },
    SafeStop {
        reply: SyncSender<Result<(), Error>>,
    },
}

pub(super) enum PendingReply {
    Start(SyncSender<Result<(), Error>>),
    Renew(SyncSender<Result<(), Error>>),
    SafeStop(SyncSender<Result<(), Error>>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Rejected,
    Unavailable,
    TimedOut,
}

pub(crate) fn start(
    grant: &WorkerLeaseGrant,
    deadlines: LeaseDeadlines,
    generation: revocation::WorkerGeneration,
) -> Result<(), Error> {
    if !revocation::is_live(generation) {
        return Err(Error::Rejected);
    }
    let deadline = deadline(deadlines)?;
    if !revocation::set_lease_deadline(generation, deadlines.expires_at_monotonic_milliseconds()) {
        return Err(Error::Rejected);
    }
    let pools = pool_set(grant)?;
    request(
        |reply| OwnerCommand::Start {
            generation,
            worker_lease_id: grant.lease_id().to_owned(),
            deadline,
            pools,
            reply,
        },
        deadlines,
    )
}

pub(crate) fn renew(
    renewal: &WorkerLeaseRenewal,
    deadlines: LeaseDeadlines,
    generation: revocation::WorkerGeneration,
) -> Result<(), Error> {
    if !revocation::permits(Some(generation)) {
        return Err(Error::Rejected);
    }
    let deadline = deadline(deadlines)?;
    if !revocation::set_lease_deadline(generation, deadlines.expires_at_monotonic_milliseconds()) {
        return Err(Error::Rejected);
    }
    request(
        |reply| OwnerCommand::Renew {
            generation,
            worker_lease_id: renewal.lease_id().to_owned(),
            deadline,
            reply,
        },
        deadlines,
    )
}

pub(crate) fn safe_stop() -> Result<(), Error> {
    let (reply, receiver) = mpsc::sync_channel(1);
    notifications()?
        .try_send(OwnerInboxMessage::Bwg(OwnerCommand::SafeStop { reply }))
        .map_err(|_| Error::Unavailable)?;
    receiver
        .recv_timeout(Duration::from_secs(140))
        .map_err(|_| Error::TimedOut)?
}

fn request(
    command: impl FnOnce(SyncSender<Result<(), Error>>) -> OwnerCommand,
    deadlines: LeaseDeadlines,
) -> Result<(), Error> {
    let (reply, receiver) = mpsc::sync_channel(1);
    notifications()?
        .try_send(OwnerInboxMessage::Bwg(command(reply)))
        .map_err(|_| Error::Unavailable)?;
    let wait = deadlines
        .expires_at_monotonic_milliseconds()
        .saturating_sub(crate::runtime_uptime::millis())
        .min(65_000);
    receiver
        .recv_timeout(Duration::from_millis(wait.max(1)))
        .map_err(|_| Error::TimedOut)?
}

fn notifications() -> Result<&'static SyncSender<OwnerInboxMessage>, Error> {
    NOTIFICATIONS.get().ok_or(Error::Unavailable)
}

fn deadline(deadlines: LeaseDeadlines) -> Result<MiningCampaignMonotonicDeadline, Error> {
    MiningCampaignMonotonicDeadline::new(deadlines.expires_at_monotonic_milliseconds())
        .map_err(|_| Error::Rejected)
}

fn lease(
    id: MiningCampaignLeaseId,
    deadline: MiningCampaignMonotonicDeadline,
) -> MiningCampaignLease {
    MiningCampaignLease::new(
        id,
        MiningHardwareProfilePreset::Conservative.profile(),
        MiningCampaignStopCondition::MonotonicDeadline { deadline },
    )
}

fn pool_set(grant: &WorkerLeaseGrant) -> Result<ProductionPoolSet, Error> {
    let authority = grant
        .stratum_endpoint()
        .strip_prefix("stratum+tcp://")
        .and_then(|value| value.strip_suffix('/'))
        .ok_or(Error::Rejected)?;
    let (host, port) = authority.rsplit_once(':').ok_or(Error::Rejected)?;
    let port = port.parse::<u16>().map_err(|_| Error::Rejected)?;
    Ok(ProductionPoolSet {
        primary: Some(ProductionPoolConfiguration {
            endpoint: ProductionPoolEndpoint {
                host: host.to_owned(),
                port,
            },
            runtime: LiveRuntimeConfig {
                model: "bitaxe-ultra".to_owned(),
                version: crate::semantic_version().to_owned(),
                credentials: LivePoolCredentials {
                    username: grant.stratum_username().to_owned(),
                    password: grant.stratum_password().to_owned(),
                },
            },
        }),
        fallback: None,
        prefer_fallback: false,
    })
}

impl OrdinaryEspProductionSessionAdapter {
    pub(super) fn event(
        &mut self,
        command: OwnerCommand,
        now_ms: u64,
        snapshot: &ProductionSessionSnapshot,
        maybe_next_lease_id: Option<MiningCampaignLeaseId>,
    ) -> ProductionSessionEvent {
        match command {
            OwnerCommand::Start {
                generation,
                worker_lease_id,
                deadline,
                pools,
                reply,
            } => {
                if self.maybe_bwg_session.is_some()
                    || !matches!(
                        snapshot.campaign_state,
                        MiningCampaignState::Unavailable | MiningCampaignState::Consumed
                    )
                    || !matches!(
                        snapshot.hardware_state,
                        MiningHardwareState::Unprepared | MiningHardwareState::Stopped
                    )
                {
                    let _ = reply.try_send(Err(Error::Rejected));
                    return self.wake_event(None, now_ms, snapshot, false);
                }
                let Some(id) = maybe_next_lease_id else {
                    let _ = reply.try_send(Err(Error::Rejected));
                    return self.wake_event(None, now_ms, snapshot, false);
                };
                revocation::check_deadline(now_ms);
                if !revocation::activate(generation, now_ms) {
                    let _ = reply.try_send(Err(Error::Rejected));
                    return self.wake_event(None, now_ms, snapshot, false);
                }
                FAN_CONTROLLER_ACTUATION_QUALIFIED.store(false, Ordering::Release);
                let session = OwnerSession {
                    generation,
                    worker_lease_id,
                    lease: lease(id, deadline),
                    pools,
                    accepted_baseline: snapshot.mining.counters.accepted,
                    rejected_baseline: snapshot.mining.counters.rejected,
                    correlated_baseline: snapshot.mining.counters.qualified_candidates,
                };
                self.maybe_bwg_session = Some(session);
                self.maybe_bwg_reply = Some(PendingReply::Start(reply));
                self.wake_event(
                    Some(ProductionSessionWakeup::ObservationsChanged),
                    now_ms,
                    snapshot,
                    false,
                )
            }
            OwnerCommand::Renew {
                generation,
                worker_lease_id,
                deadline,
                reply,
            } => {
                let same_lease = self.maybe_bwg_session.as_ref().is_some_and(|session| {
                    session.worker_lease_id == worker_lease_id
                        && session.generation == generation
                        && revocation::permits(Some(generation))
                });
                if !same_lease || snapshot.campaign_state != MiningCampaignState::Active {
                    let _ = reply.try_send(Err(Error::Rejected));
                    return self.wake_event(None, now_ms, snapshot, false);
                }
                let Some(id) = self
                    .maybe_bwg_session
                    .as_ref()
                    .map(|session| session.lease.id())
                else {
                    let _ = reply.try_send(Err(Error::Rejected));
                    return self.wake_event(None, now_ms, snapshot, false);
                };
                let lease = lease(id, deadline);
                if let Some(session) = self.maybe_bwg_session.as_mut() {
                    session.lease = lease;
                }
                self.maybe_bwg_reply = Some(PendingReply::Renew(reply));
                ProductionSessionEvent::CampaignLeaseRenewed { lease, now_ms }
            }
            OwnerCommand::SafeStop { reply } => {
                if let Some(session) = self.maybe_bwg_session.as_ref() {
                    revocation::revoke_at(session.generation, now_ms);
                }
                if self.maybe_bwg_session.is_none() {
                    let _ = reply.try_send(Ok(()));
                    return self.wake_event(None, now_ms, snapshot, false);
                }
                self.maybe_bwg_reply = Some(PendingReply::SafeStop(reply));
                ProductionSessionEvent::CampaignLeaseRevoked
            }
        }
    }

    pub(super) fn complete_reply(&mut self, snapshot: &ProductionSessionSnapshot) {
        let maybe_result = match self.maybe_bwg_reply.as_ref() {
            Some(PendingReply::Start(_) | PendingReply::Renew(_))
                if snapshot.campaign_state == MiningCampaignState::Active
                    && self
                        .maybe_bwg_session
                        .as_ref()
                        .is_some_and(|session| revocation::permits(Some(session.generation))) =>
            {
                Some(Ok(()))
            }
            Some(PendingReply::SafeStop(_))
                if snapshot.campaign_state == MiningCampaignState::Consumed
                    && snapshot.hardware_state == MiningHardwareState::Stopped =>
            {
                Some(Ok(()))
            }
            Some(PendingReply::Start(_) | PendingReply::Renew(_))
                if snapshot.campaign_state == MiningCampaignState::Consumed =>
            {
                Some(Err(Error::Rejected))
            }
            _ => None,
        };
        let Some(result) = maybe_result else {
            self.finish_consumed_generation(snapshot);
            return;
        };
        let Some(reply) = self.maybe_bwg_reply.take() else {
            return;
        };
        match reply {
            PendingReply::Start(sender)
            | PendingReply::Renew(sender)
            | PendingReply::SafeStop(sender) => {
                let _ = sender.try_send(result);
            }
        }
        self.finish_consumed_generation(snapshot);
    }

    fn finish_consumed_generation(&mut self, snapshot: &ProductionSessionSnapshot) {
        if snapshot.campaign_state == MiningCampaignState::Consumed
            && snapshot.hardware_state == MiningHardwareState::Stopped
        {
            if let Some(session) = self.maybe_bwg_session.as_ref() {
                revocation::revoke_reason_at(
                    session.generation,
                    crate::runtime_uptime::millis(),
                    revocation::RevocationReason::ControlFailed,
                );
                if crate::worker_acceptance_budget::finish(session.generation).is_err() {
                    return;
                }
                revocation::finish_shutdown(session.generation);
            }
            self.maybe_bwg_session = None;
        }
    }
}
