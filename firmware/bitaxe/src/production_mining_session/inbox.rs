use super::*;
impl OrdinaryEspProductionSessionAdapter {
    #[inline(never)]
    pub(super) fn event_from_inbox(
        &mut self,
        message: OwnerInboxMessage,
        now_ms: u64,
        snapshot: &ProductionSessionSnapshot,
        maybe_next_lease_id: Option<bitaxe_stratum::v1::production_session::MiningCampaignLeaseId>,
    ) -> ProductionSessionEvent {
        match message {
            OwnerInboxMessage::Wake(wakeup) => {
                self.wake_event(Some(wakeup), now_ms, snapshot, false)
            }
            OwnerInboxMessage::Transport(event) => match event {
                PoolTransportEvent::FrameWritten {
                    pool,
                    transport_epoch,
                    sequence,
                } => ProductionSessionEvent::FrameWritten {
                    pool,
                    transport_epoch,
                    sequence,
                    now_ms,
                },
                PoolTransportEvent::Frame {
                    pool,
                    transport_epoch,
                    frame,
                } => ProductionSessionEvent::TransportFrame {
                    pool,
                    transport_epoch,
                    frame,
                    now_ms,
                },
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
                AsicWorkerEvent::Dispatched { generation, job_id } => {
                    ProductionSessionEvent::AsicDispatched {
                        generation,
                        job_id,
                        now_ms,
                    }
                }
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
                } => {
                    crate::v2_serial_runtime::asic_failed(generation, failure);
                    ProductionSessionEvent::AsicInteractionFailed {
                        generation,
                        failure,
                        now_ms,
                    }
                }
            },
            OwnerInboxMessage::Bwg(command) => {
                self.event(command, now_ms, snapshot, maybe_next_lease_id)
            }
        }
    }
}
