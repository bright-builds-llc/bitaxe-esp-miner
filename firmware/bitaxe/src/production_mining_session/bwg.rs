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
    pub(super) preparation_started: bool,
    pub(super) expected_filter_counts: [u64; 2],
}

impl OwnerSession {
    pub(super) fn publication_counts(&self, snapshot: &ProductionSessionSnapshot) -> [u64; 3] {
        [
            snapshot
                .lifetime_share_counters
                .accepted
                .saturating_sub(self.accepted_baseline),
            snapshot
                .lifetime_share_counters
                .rejected
                .saturating_sub(self.rejected_baseline),
            snapshot
                .lifetime_share_counters
                .qualified_candidates
                .saturating_sub(self.correlated_baseline),
        ]
    }
}

pub(super) enum OwnerCommand {
    Cooling {
        generation: revocation::WorkerGeneration,
        restore: bool,
        reply: SyncSender<Result<serde_json::Value, Error>>,
    },
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

pub(crate) fn cooling(
    generation: revocation::WorkerGeneration,
    restore: bool,
) -> Result<serde_json::Value, Error> {
    let (reply, receiver) = mpsc::sync_channel(1);
    notifications()?
        .try_send(OwnerInboxMessage::Bwg(OwnerCommand::Cooling {
            generation,
            restore,
            reply,
        }))
        .map_err(|_| Error::Unavailable)?;
    receiver
        .recv_timeout(Duration::from_secs(4))
        .map_err(|_| Error::TimedOut)?
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
                maybe_suggested_difficulty: grant.maybe_suggested_difficulty(),
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
            OwnerCommand::Cooling {
                generation,
                restore,
                reply,
            } => {
                let result = self.cooling_command(generation, restore, snapshot);
                let _ = reply.try_send(result);
                self.wake_event(None, now_ms, snapshot, false)
            }
            OwnerCommand::Start {
                generation,
                worker_lease_id,
                deadline,
                pools,
                reply,
            } => {
                if self.maybe_bwg_session.is_some()
                    || self
                        .maybe_cooling_generation
                        .is_some_and(|owned| owned != generation)
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
                self.maybe_cooling_generation = None; // Ownership transfers to ordered mining cleanup.
                FAN_CONTROLLER_ACTUATION_QUALIFIED.store(false, Ordering::Release);
                let session = OwnerSession {
                    generation,
                    worker_lease_id,
                    lease: lease(id, deadline),
                    pools,
                    accepted_baseline: snapshot.lifetime_share_counters.accepted,
                    rejected_baseline: snapshot.lifetime_share_counters.rejected,
                    correlated_baseline: snapshot.lifetime_share_counters.qualified_candidates,
                    preparation_started: false,
                    expected_filter_counts: [0; 2],
                };
                admission_diagnostics::stage(admission_diagnostics::Stage::Readiness);
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
                    let result = self.restore_owned_cooling(snapshot);
                    let _ = reply.try_send(result);
                    return self.wake_event(None, now_ms, snapshot, false);
                }
                self.maybe_bwg_reply = Some(PendingReply::SafeStop(reply));
                ProductionSessionEvent::CampaignLeaseRevoked
            }
        }
    }

    fn cooling_command(
        &mut self,
        generation: revocation::WorkerGeneration,
        restore: bool,
        snapshot: &ProductionSessionSnapshot,
    ) -> Result<serde_json::Value, Error> {
        if self.maybe_bwg_session.is_some()
            || !Self::cooling_hardware_idle(snapshot)
            || !revocation::is_live(generation)
            || self
                .maybe_cooling_generation
                .is_some_and(|owned| owned != generation)
        {
            return Err(Error::Rejected);
        }
        if restore {
            if self.maybe_cooling_generation != Some(generation) {
                return Err(Error::Rejected);
            }
            let result = cooling::restore(generation).map_err(Self::cooling_error)?;
            if !revocation::release_unbudgeted_reservation(generation)
                && revocation::maybe_revoked() != Some(generation)
            {
                return Err(Error::Rejected);
            }
            self.maybe_cooling_generation = None;
            return Ok(result);
        }
        if self.maybe_cooling_generation.is_some() || !revocation::begin_reservation(generation) {
            return Err(Error::Rejected);
        }
        self.maybe_cooling_generation = Some(generation);
        FAN_CONTROLLER_ACTUATION_QUALIFIED.store(false, Ordering::Release);
        cooling::qualify(generation).map_err(Self::cooling_error)
    }

    fn cooling_hardware_idle(snapshot: &ProductionSessionSnapshot) -> bool {
        matches!(
            snapshot.campaign_state,
            MiningCampaignState::Unavailable | MiningCampaignState::Consumed
        ) && matches!(
            snapshot.hardware_state,
            MiningHardwareState::Unprepared | MiningHardwareState::Stopped
        )
    }

    fn cooling_error(error: cooling_core::CoolingError) -> Error {
        match error {
            cooling_core::CoolingError::TimedOut => Error::TimedOut,
            _ => Error::Rejected,
        }
    }

    pub(super) fn restore_owned_cooling(
        &mut self,
        snapshot: &ProductionSessionSnapshot,
    ) -> Result<(), Error> {
        let Some(generation) = self.maybe_cooling_generation else {
            return Ok(());
        };
        if self.maybe_bwg_session.is_some() || !Self::cooling_hardware_idle(snapshot) {
            return Err(Error::Rejected);
        }
        cooling::restore(generation).map_err(Self::cooling_error)?;
        if !revocation::release_unbudgeted_reservation(generation)
            && revocation::maybe_revoked() != Some(generation)
        {
            return Err(Error::Rejected);
        }
        self.maybe_cooling_generation = None;
        Ok(())
    }

    pub(super) fn note_worker_preparation_started(&mut self) {
        if let Some(session) = self.maybe_bwg_session.as_mut() {
            // Even cancelled or failed preparation must follow ordered safe stop.
            session.preparation_started = true;
            admission_diagnostics::stage(admission_diagnostics::Stage::Preparation);
        }
    }

    pub(super) fn complete_reply(&mut self, snapshot: &ProductionSessionSnapshot) {
        let never_prepared = self
            .maybe_bwg_session
            .as_ref()
            .is_some_and(|session| !session.preparation_started);
        if never_prepared && matches!(self.maybe_bwg_reply, Some(PendingReply::Start(_))) {
            admission_diagnostics::fail(admission_diagnostics::Failure::Readiness);
            if let Some(session) = self.maybe_bwg_session.as_ref() {
                revocation::revoke_reason_at(
                    session.generation,
                    crate::runtime_uptime::millis(),
                    revocation::RevocationReason::ControlFailed,
                );
            }
        }
        let retired = self.finish_consumed_generation(snapshot);
        let maybe_result = match self.maybe_bwg_reply.as_ref() {
            Some(PendingReply::Start(_)) if never_prepared => Some(Err(Error::Rejected)),
            Some(PendingReply::Start(_) | PendingReply::Renew(_))
                if snapshot.campaign_state == MiningCampaignState::Active
                    && self
                        .maybe_bwg_session
                        .as_ref()
                        .is_some_and(|session| revocation::permits(Some(session.generation))) =>
            {
                if let Some(worker) = self.maybe_bwg_session.as_ref() {
                    owner_resources::capture(
                        worker.generation.raw(),
                        owner_resources::Phase::Active,
                    );
                }
                admission_diagnostics::stage(admission_diagnostics::Stage::Active);
                Some(Ok(()))
            }
            Some(PendingReply::SafeStop(_)) if retired => Some(Ok(())),
            Some(PendingReply::Start(_) | PendingReply::Renew(_))
                if retired || snapshot.campaign_state == MiningCampaignState::Consumed =>
            {
                Some(Err(Error::Rejected))
            }
            _ => None,
        };
        let Some(result) = maybe_result else {
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
    }

    fn finish_consumed_generation(&mut self, snapshot: &ProductionSessionSnapshot) -> bool {
        let Some(session) = self.maybe_bwg_session.as_ref() else {
            return false;
        };
        // A rejected candidate has no hardware effects to undo and no core
        // terminal receipt to await. Its reserved budget must still be finalized.
        let unstarted_revoked =
            !session.preparation_started && !revocation::is_live(session.generation);
        let stopped = session.preparation_started
            && snapshot.campaign_state == MiningCampaignState::Consumed
            && snapshot.hardware_state == MiningHardwareState::Stopped;
        if unstarted_revoked || stopped {
            admission_diagnostics::stage(admission_diagnostics::Stage::Cleanup);
            revocation::revoke_reason_at(
                session.generation,
                crate::runtime_uptime::millis(),
                revocation::RevocationReason::ControlFailed,
            );
            if crate::worker_acceptance_budget::finish(session.generation).is_err() {
                admission_diagnostics::fail(admission_diagnostics::Failure::Cleanup);
                return false;
            }
            mining_progress::capture(
                session.generation.raw(),
                snapshot,
                session.expected_filter_counts,
            );
            owner_resources::capture(
                session.generation.raw(),
                owner_resources::Phase::ShutdownComplete,
            );
            revocation::finish_shutdown(session.generation);
            self.maybe_bwg_session = None;
            admission_diagnostics::stage(admission_diagnostics::Stage::Complete);
            return true;
        }
        false
    }
}
