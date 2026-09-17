use super::*;
use bitaxe_stratum::v2::{
    frame::Frame,
    messages::ServerMessage,
    noise::{
        diagnostic::{self, Failure, Observer, Phase},
        NoiseTransport,
    },
    standard::StandardEvent,
    standard_io::ChannelObserver,
};
use std::net::{SocketAddr, TcpStream};

pub(super) struct NativeObserver<'a> {
    maybe_share_io: Option<&'a mut dyn ShareIo>,
    maybe_submission: Option<u32>,
    write_completed: bool,
    config: Vec<bitaxe_stratum::v2::session::SessionConfig>,
    maybe_frame: Option<(u32, Option<u32>, String)>,
}
#[inline(never)]
pub(super) fn run_channel(mut inputs: Vec<Input>) {
    let Some(input) = inputs.first_mut() else {
        return;
    };
    if !permitted() {
        with_record(|r| {
            r.fail(Stage::Revoked, FailureCategory::Authority, now_us());
            r.begin(Operation::WorkerJoin, now_us());
        });
        return;
    }

    with_record(|r| r.event(Stage::Preparing, now_us(), None, None, None, None));
    let mut generators = match crate::noise_serial_runtime::prepare_rng(crate::crypto_entropy::fill)
    {
        Ok(v) => v,
        Err(error) => {
            with_record(|r| {
                r.fail(
                    Stage::Preparing,
                    if error == crate::noise_serial_runtime::RngFailure::Allocation {
                        FailureCategory::Allocation
                    } else {
                        FailureCategory::Admission
                    },
                    now_us(),
                )
            });
            with_record(|r| r.begin(Operation::WorkerJoin, now_us()));
            return;
        }
    };
    let Some(generator) = generators.first_mut() else {
        return;
    };
    diagnostic::run(
        input.endpoint.into(),
        input.authority,
        generator,
        &mut NativeObserver {
            config: std::mem::take(&mut input.config),
            maybe_frame: None,
            maybe_share_io: None,
            maybe_submission: None,
            write_completed: false,
        },
    );
}
impl Observer for NativeObserver<'_> {
    fn permitted(&mut self) -> bool {
        permitted()
    }
    fn now_us(&self) -> Option<u64> {
        now_us()
    }
    fn operation_started(&mut self, op: diagnostic::Operation) {
        with_record(|r| {
            r.begin(operation(op), now_us());
            if op == diagnostic::Operation::FrameWrite {
                if let Some(f) = self.maybe_submission.and_then(|s| r.share_mut(s)) {
                    f.maybe_write_started_at_device_us = now_us();
                    let ids = (f.channel_id, f.job_id, f.submission_sequence);
                    r.event(
                        Stage::Submission,
                        now_us(),
                        Some(ids.0),
                        Some(ids.1),
                        Some(ids.2),
                        None,
                    );
                }
            }
        });
    }
    fn frame_write_completed(&mut self) {
        self.write_completed = true;
        with_record(|r| {
            if let Some(f) = self.maybe_submission.and_then(|s| r.share_mut(s)) {
                f.maybe_write_completed_at_device_us = now_us();
            }
        });
    }
    fn frame_authenticated(&mut self, frame: &Frame) -> Result<(), Failure> {
        facts::received_acknowledgement(frame)
    }
    fn operation_finished(&mut self, op: diagnostic::Operation, failed: bool) {
        with_record(|r| r.end(operation(op), now_us(), failed));
    }
    fn connected(&mut self, stream: &TcpStream) -> Result<(), Failure> {
        let (SocketAddr::V4(local), SocketAddr::V4(remote)) = (
            stream.local_addr().map_err(|_| Failure::Io)?,
            stream.peer_addr().map_err(|_| Failure::Io)?,
        ) else {
            return Err(Failure::Io);
        };
        if !local.ip().is_private() || !remote.ip().is_private() {
            return Err(Failure::Io);
        }
        let mut data = DATA.lock().map_err(|_| Failure::Io)?;
        let time = now_us().ok_or(Failure::Clock)?;
        let record = data.maybe_record.as_ref().ok_or(Failure::Io)?;
        let (boot, generation, epoch) = record.binding();
        let (pool_generation, pool_epoch) = record.pool_binding().ok_or(Failure::Io)?;
        if data.maybe_connection.is_some() {
            return Err(Failure::Extra);
        }
        data.maybe_connection = Some(RetainedConnection {
            observed_at_us: time,
            boot_ordinal: boot,
            worker_generation: generation,
            serial_transport_epoch: epoch,
            pool_session_generation: pool_generation,
            pool_transport_epoch: pool_epoch,
            socket: SocketTuple {
                local_ipv4: local.ip().to_string(),
                local_port: local.port(),
                remote_ipv4: remote.ip().to_string(),
                remote_port: remote.port(),
            },
        });
        Ok(())
    }
    fn authenticated(
        &mut self,
        stream: &mut TcpStream,
        noise: &mut NoiseTransport,
    ) -> Result<(), Failure> {
        if let Some(io) = self.maybe_share_io.take() {
            return self.share_loop(stream, noise, io);
        }
        let config = self.config.pop().ok_or(Failure::Io)?;
        bitaxe_stratum::v2::standard_io::run_channel(stream, noise, config, self)
    }
    fn event(&mut self, event: diagnostic::Event) {
        with_record(|r| match event {
            diagnostic::Event::Cleanup => r.begin(Operation::WorkerJoin, now_us()),
            diagnostic::Event::SocketClosedWithoutClock => r.socket_closed(None),
            diagnostic::Event::Complete {
                phase: Phase::Connect,
                at_us,
                ..
            } => r.event(Stage::Connected, Some(at_us), None, None, None, None),
            diagnostic::Event::Complete {
                phase: Phase::Authenticate,
                at_us,
                ..
            } => r.event(Stage::Authenticated, Some(at_us), None, None, None, None),
            diagnostic::Event::Complete {
                phase: Phase::Close,
                at_us,
                ..
            } => r.socket_closed(Some(at_us)),
            _ => {}
        });
    }
    fn failed(&mut self, _phase: Phase, failure: Failure) {
        if failure == Failure::Cancelled && facts::expected_heartbeat_fault() {
            return;
        }
        let category = match failure {
            Failure::Cancelled => FailureCategory::Authority,
            Failure::Clock => FailureCategory::Clock,
            Failure::Timeout => FailureCategory::Timeout,
            Failure::Eof => FailureCategory::Eof,
            Failure::Extra => FailureCategory::Extra,
            Failure::Allocation => FailureCategory::Allocation,
            Failure::Authentication(_) => FailureCategory::Authentication,
            _ => FailureCategory::Protocol,
        };
        with_record(|r| r.fail(Stage::WorkerQuiescent, category, now_us()));
    }
}
impl ChannelObserver for NativeObserver<'_> {
    fn validated_frame(&mut self, frame: &Frame) -> Result<(), Failure> {
        use sha2::{Digest, Sha256};
        self.maybe_frame = match ServerMessage::decode(frame).map_err(|_| Failure::Io)? {
            ServerMessage::OpenStandardMiningChannelSuccess(v) => {
                Some((v.channel_id, None, hex(&Sha256::digest(frame.encode()))))
            }
            ServerMessage::NewMiningJob(v) => Some((
                v.channel_id,
                Some(v.job_id),
                hex(&Sha256::digest(frame.encode())),
            )),
            ServerMessage::SetTarget(v) => Some((
                v.channel_id,
                self.maybe_frame.as_ref().and_then(|(_, j, _)| *j),
                hex(&Sha256::digest(frame.encode())),
            )),
            ServerMessage::SetNewPrevHash(v) => Some((
                v.channel_id,
                Some(v.job_id),
                hex(&Sha256::digest(frame.encode())),
            )),
            _ => None,
        };
        Ok(())
    }
    fn protocol_event(&mut self, event: &StandardEvent) -> Result<(), Failure> {
        let kind = match event {
            StandardEvent::Setup => Stage::Setup,
            StandardEvent::Channel => Stage::Channel,
            StandardEvent::Job => Stage::Job,
            StandardEvent::Target => Stage::Target,
            StandardEvent::Work { .. } => Stage::WorkReady,
            StandardEvent::Send(_) => return Ok(()),
            StandardEvent::Accepted(_) => return Err(Failure::Io),
        };
        let (channel, job, digest) = self
            .maybe_frame
            .as_ref()
            .map_or((None, None, None), |(c, j, h)| {
                (Some(*c), *j, Some(h.clone()))
            });
        with_record(|r| {
            if let StandardEvent::Work { commitment, .. } = event {
                r.set_job_commitment(commitment.clone());
            }
            r.event(kind, now_us(), channel, job, None, digest);
        });
        Ok(())
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn operation(op: diagnostic::Operation) -> Operation {
    match op {
        diagnostic::Operation::InitiatorConstruction => Operation::InitiatorConstruction,
        diagnostic::Operation::ActOneConstruction => Operation::ActOneConstruction,
        diagnostic::Operation::Connect => Operation::Connect,
        diagnostic::Operation::ActOneWrite => Operation::ActOneWrite,
        diagnostic::Operation::ActTwoRead => Operation::ActTwoRead,
        diagnostic::Operation::ActTwoAuthentication => Operation::ActTwoAuthentication,
        diagnostic::Operation::FrameEncrypt => Operation::FrameEncrypt,
        diagnostic::Operation::FrameWrite => Operation::FrameWrite,
        diagnostic::Operation::FrameRead => Operation::FrameRead,
        diagnostic::Operation::HeaderDecrypt => Operation::HeaderDecrypt,
        diagnostic::Operation::PayloadDecrypt => Operation::PayloadDecrypt,
        diagnostic::Operation::SocketClose => Operation::SocketClose,
        diagnostic::Operation::WorkerJoin => Operation::WorkerJoin,
    }
}

#[inline(never)]
pub(super) fn run_share(
    endpoint: SocketAddrV4,
    authority: [u8; 32],
    io: &mut dyn ShareIo,
) -> Result<(), ()> {
    with_record(|r| r.event(Stage::Preparing, now_us(), None, None, None, None));
    let mut generators = crate::noise_serial_runtime::prepare_rng(crate::crypto_entropy::fill)
        .map_err(|error| {
            with_record(|r| {
                r.fail(
                    Stage::Preparing,
                    if error == crate::noise_serial_runtime::RngFailure::Allocation {
                        FailureCategory::Allocation
                    } else {
                        FailureCategory::Admission
                    },
                    now_us(),
                );
                r.begin(Operation::WorkerJoin, now_us());
            });
        })?;
    let generator = generators.first_mut().ok_or(())?;
    diagnostic::run(
        endpoint.into(),
        authority,
        generator,
        &mut NativeObserver {
            config: Vec::new(),
            maybe_frame: None,
            maybe_share_io: Some(io),
            maybe_submission: None,
            write_completed: false,
        },
    );
    DATA.lock().map_err(|_| ()).and_then(|d| {
        if d.maybe_record
            .as_ref()
            .is_some_and(|r| r.maybe_failure().is_none())
        {
            Ok(())
        } else {
            Err(())
        }
    })
}
impl NativeObserver<'_> {
    fn share_loop(
        &mut self,
        stream: &mut TcpStream,
        noise: &mut NoiseTransport,
        io: &mut dyn ShareIo,
    ) -> Result<(), Failure> {
        io.authenticated();
        let mut waiting = now_us().ok_or(Failure::Clock)?;
        let mut work_ready = false;
        while io.open() {
            if !self.permitted() {
                return Err(Failure::Cancelled);
            }
            if let Some((frame, permit)) = io.next_write().map_err(|_| Failure::Io)? {
                if !revocation::permits_work(permit) {
                    return Err(Failure::Cancelled);
                }
                self.maybe_submission = facts::prepare_submission(&frame)?;
                self.write_completed = false;
                let result =
                    bitaxe_stratum::v2::standard_io::write_frame(stream, noise, &frame, self);
                if self.write_completed {
                    if let Some(sequence) = self.maybe_submission {
                        io.written(sequence);
                        revocation::note_submission(permit.maybe_generation());
                    }
                }
                self.maybe_submission = None;
                result?;
                waiting = now_us().ok_or(Failure::Clock)?;
            }
            if let Some(frame) =
                bitaxe_stratum::v2::standard_io::maybe_read_frame(stream, noise, self)?
            {
                work_ready |= frame.header.message_type
                    == bitaxe_stratum::v2::messages::MessageType::SetNewPrevHash as u8;
                io.frame(frame);
                waiting = now_us().ok_or(Failure::Clock)?;
            }
            if !work_ready
                && now_us()
                    .and_then(|t| t.checked_sub(waiting))
                    .is_none_or(|elapsed| elapsed >= 10_000_000)
            {
                return Err(Failure::Timeout);
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        Ok(())
    }
}
