use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bitaxe_stratum::v2::connection_order::{prepare_before_connect, PrepareBeforeConnectError};
use bitaxe_stratum::v2::frame::Frame;
use bitaxe_stratum::v2::messages::ServerMessage;
use bitaxe_stratum::v2::noise::{
    DecryptedNoiseHeader, NoiseCompletionFailure, NoiseInitiator, NoisePreparationStage,
    NoiseTransport, ACT_TWO_LEN, ENCRYPTED_HEADER_LEN,
};
use bitaxe_stratum::v2::MAX_FRAME_PAYLOAD;
use esp_idf_svc::sys;
use rand::{CryptoRng, RngCore};

use crate::settings_adapter::V2PoolSettings;

const COMMAND_CAPACITY: usize = 8;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
const READ_TIMEOUT: Duration = Duration::from_millis(20);
const WRITE_TIMEOUT: Duration = Duration::from_secs(2);
const ADDRESS_CAPACITY: usize = 8;
const WORKER_STACK_BYTES: usize = 24 * 1024;
const ENCRYPTED_BUFFER_CAPACITY: usize = (MAX_FRAME_PAYLOAD + 16 + ENCRYPTED_HEADER_LEN) * 2;
const MAX_NOISE_PREPARATION: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransportFailure {
    Resolve,
    Connect,
    Configure,
    Handshake,
    Write,
    Read,
    Frame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoiseDiagnosticStage {
    NoisePrepared,
    TcpConnected,
    ActOneCreated,
    ActOneSent,
    ActTwoReceived,
    TimeSampled,
    Authenticated,
}

impl NoiseDiagnosticStage {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::NoisePrepared => "noise_prepared",
            Self::TcpConnected => "tcp_connected",
            Self::ActOneCreated => "act_one_created",
            Self::ActOneSent => "act_one_sent",
            Self::ActTwoReceived => "act_two_received",
            Self::TimeSampled => "time_sampled",
            Self::Authenticated => "authenticated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoiseDiagnosticFailure {
    Resolve,
    Connect,
    Configure,
    Rng,
    ActOne,
    ActOneWrite,
    ActTwoRead,
    PreparationSlow,
    ClockBeforeEpoch,
    ClockOverflow,
    Completion(NoiseCompletionFailure),
}

impl NoiseDiagnosticFailure {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Resolve => "resolve",
            Self::Connect => "connect",
            Self::Configure => "configure",
            Self::Rng => "rng",
            Self::ActOne => "act_one",
            Self::ActOneWrite => "act_one_write",
            Self::ActTwoRead => "act_two_read",
            Self::PreparationSlow => "preparation_slow",
            Self::ClockBeforeEpoch => "clock_before_epoch",
            Self::ClockOverflow => "clock_overflow",
            Self::Completion(failure) => failure.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoiseDiagnosticTimingKind {
    KeypairPreparation,
    ActOneConstruction,
    Connect,
    ActOneWrite,
    ActTwoRead,
}

impl NoiseDiagnosticTimingKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::KeypairPreparation => "keypair_preparation",
            Self::ActOneConstruction => "act_one_construction",
            Self::Connect => "connect",
            Self::ActOneWrite => "act_one_write",
            Self::ActTwoRead => "act_two_read",
        }
    }
}

pub(super) enum TransportCommand {
    Send(Frame),
    Close,
}

impl core::fmt::Debug for TransportCommand {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Send(_) => formatter.write_str("TransportCommand::Send(redacted)"),
            Self::Close => formatter.write_str("TransportCommand::Close"),
        }
    }
}

pub(super) enum TransportEvent {
    Established,
    Message(ServerMessage),
    Failed(TransportFailure),
    Closed,
}

impl core::fmt::Debug for TransportEvent {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Established => formatter.write_str("TransportEvent::Established"),
            Self::Message(message) => formatter
                .debug_tuple("TransportEvent::Message")
                .field(message)
                .finish(),
            Self::Failed(failure) => formatter
                .debug_tuple("TransportEvent::Failed")
                .field(failure)
                .finish(),
            Self::Closed => formatter.write_str("TransportEvent::Closed"),
        }
    }
}

pub(super) struct TransportHandle {
    sender: SyncSender<TransportCommand>,
}

impl TransportHandle {
    pub(super) fn spawn(
        settings: V2PoolSettings,
        emit: impl Fn(TransportEvent) + Send + 'static,
    ) -> io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(COMMAND_CAPACITY);
        std::thread::Builder::new()
            .name("stratum-v2-transport".to_owned())
            .stack_size(WORKER_STACK_BYTES)
            .spawn(move || run_worker(settings, receiver, emit))?;
        Ok(Self { sender })
    }

    pub(super) fn try_send(
        &self,
        command: TransportCommand,
    ) -> Result<(), TrySendError<TransportCommand>> {
        self.sender.try_send(command)
    }
}

impl Drop for TransportHandle {
    fn drop(&mut self) {
        let _ = self.sender.try_send(TransportCommand::Close);
    }
}

fn run_worker(
    settings: V2PoolSettings,
    receiver: Receiver<TransportCommand>,
    emit: impl Fn(TransportEvent),
) {
    let result = connect_and_run(settings, &receiver, &emit);
    if let Err(failure) = result {
        emit(TransportEvent::Failed(failure));
    }
    emit(TransportEvent::Closed);
}

fn connect_and_run(
    settings: V2PoolSettings,
    receiver: &Receiver<TransportCommand>,
    emit: &impl Fn(TransportEvent),
) -> Result<(), TransportFailure> {
    let addresses = (
        settings.session.endpoint_host.as_str(),
        settings.session.endpoint_port,
    )
        .to_socket_addrs()
        .map_err(|_| TransportFailure::Resolve)?
        .take(ADDRESS_CAPACITY + 1)
        .collect::<Vec<_>>();
    if addresses.is_empty() || addresses.len() > ADDRESS_CAPACITY {
        return Err(TransportFailure::Resolve);
    }
    let mut rng = EspHardwareRng;
    let preparation_started = std::time::Instant::now();
    let (prepared, mut stream) = prepare_before_connect(
        || {
            let prepared = NoiseInitiator::prepare(settings.maybe_authority_public_key, &mut rng)
                .map_err(|_| TransportFailure::Handshake)?;
            if preparation_started.elapsed() > MAX_NOISE_PREPARATION {
                return Err(TransportFailure::Handshake);
            }
            Ok(prepared)
        },
        || connect_first(&addresses).ok_or(TransportFailure::Connect),
    )
    .map_err(map_prepare_connect_failure)?;
    stream
        .set_read_timeout(Some(HANDSHAKE_TIMEOUT))
        .map_err(|_| TransportFailure::Configure)?;
    stream
        .set_write_timeout(Some(WRITE_TIMEOUT))
        .map_err(|_| TransportFailure::Configure)?;
    stream
        .write_all(prepared.act_one())
        .map_err(|_| TransportFailure::Handshake)?;
    let mut act_two = [0; ACT_TWO_LEN];
    stream
        .read_exact(&mut act_two)
        .map_err(|_| TransportFailure::Handshake)?;
    let unix_time_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TransportFailure::Handshake)?
        .as_secs()
        .try_into()
        .map_err(|_| TransportFailure::Handshake)?;
    let noise = prepared
        .complete(&act_two, unix_time_seconds)
        .map_err(|_| TransportFailure::Handshake)?;
    stream
        .set_read_timeout(Some(READ_TIMEOUT))
        .map_err(|_| TransportFailure::Configure)?;
    emit(TransportEvent::Established);
    run_encrypted_loop(stream, noise, receiver, emit)
}

pub(crate) fn run_noise_diagnostic(
    settings: V2PoolSettings,
    emit_stage: impl Fn(NoiseDiagnosticStage),
    emit_timing: impl Fn(NoiseDiagnosticTimingKind, u32),
) -> Result<(), NoiseDiagnosticFailure> {
    let addresses = (
        settings.session.endpoint_host.as_str(),
        settings.session.endpoint_port,
    )
        .to_socket_addrs()
        .map_err(|_| NoiseDiagnosticFailure::Resolve)?
        .take(ADDRESS_CAPACITY + 1)
        .collect::<Vec<_>>();
    if addresses.is_empty() || addresses.len() > ADDRESS_CAPACITY {
        return Err(NoiseDiagnosticFailure::Resolve);
    }
    let mut rng = EspHardwareRng;
    let preparation_started = std::time::Instant::now();
    let mut keypair_preparation_ms = 0_u32;
    let mut connect_ms = 0_u32;
    let (prepared, mut stream) = prepare_before_connect(
        || {
            let prepared = NoiseInitiator::prepare_with_observer(
                settings.maybe_authority_public_key,
                &mut rng,
                |stage| match stage {
                    NoisePreparationStage::KeypairReady => {
                        keypair_preparation_ms = elapsed_ms(preparation_started);
                        emit_timing(
                            NoiseDiagnosticTimingKind::KeypairPreparation,
                            keypair_preparation_ms,
                        );
                    }
                    NoisePreparationStage::ActOneReady => {
                        let total_ms = elapsed_ms(preparation_started);
                        emit_timing(
                            NoiseDiagnosticTimingKind::ActOneConstruction,
                            total_ms.saturating_sub(keypair_preparation_ms),
                        );
                    }
                },
            )
            .map_err(|_| NoiseDiagnosticFailure::Rng)?;
            if preparation_started.elapsed() > MAX_NOISE_PREPARATION {
                return Err(NoiseDiagnosticFailure::PreparationSlow);
            }
            emit_stage(NoiseDiagnosticStage::NoisePrepared);
            Ok(prepared)
        },
        || {
            let connect_started = std::time::Instant::now();
            let stream = connect_first(&addresses).ok_or(NoiseDiagnosticFailure::Connect)?;
            connect_ms = elapsed_ms(connect_started);
            Ok(stream)
        },
    )
    .map_err(map_diagnostic_prepare_connect_failure)?;
    emit_timing(NoiseDiagnosticTimingKind::Connect, connect_ms);
    emit_stage(NoiseDiagnosticStage::TcpConnected);
    stream
        .set_read_timeout(Some(HANDSHAKE_TIMEOUT))
        .map_err(|_| NoiseDiagnosticFailure::Configure)?;
    stream
        .set_write_timeout(Some(WRITE_TIMEOUT))
        .map_err(|_| NoiseDiagnosticFailure::Configure)?;
    emit_stage(NoiseDiagnosticStage::ActOneCreated);
    let write_started = std::time::Instant::now();
    let write_result = stream.write_all(prepared.act_one());
    emit_timing(
        NoiseDiagnosticTimingKind::ActOneWrite,
        elapsed_ms(write_started),
    );
    write_result.map_err(|_| NoiseDiagnosticFailure::ActOneWrite)?;
    emit_stage(NoiseDiagnosticStage::ActOneSent);
    let mut act_two = [0; ACT_TWO_LEN];
    let read_started = std::time::Instant::now();
    let read_result = stream.read_exact(&mut act_two);
    emit_timing(
        NoiseDiagnosticTimingKind::ActTwoRead,
        elapsed_ms(read_started),
    );
    read_result.map_err(|_| NoiseDiagnosticFailure::ActTwoRead)?;
    emit_stage(NoiseDiagnosticStage::ActTwoReceived);
    let unix_time_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| NoiseDiagnosticFailure::ClockBeforeEpoch)?
        .as_secs()
        .try_into()
        .map_err(|_| NoiseDiagnosticFailure::ClockOverflow)?;
    emit_stage(NoiseDiagnosticStage::TimeSampled);
    let mut noise = prepared
        .complete_diagnostic(&act_two, unix_time_seconds)
        .map_err(NoiseDiagnosticFailure::Completion)?;
    emit_stage(NoiseDiagnosticStage::Authenticated);
    let proof = Frame::new(0, 0, Vec::new())
        .map_err(|_| NoiseDiagnosticFailure::Completion(NoiseCompletionFailure::Other))?;
    let encrypted = noise
        .encrypt_frame(&proof)
        .map_err(|_| NoiseDiagnosticFailure::Completion(NoiseCompletionFailure::Other))?;
    stream
        .write_all(&encrypted)
        .map_err(|_| NoiseDiagnosticFailure::Completion(NoiseCompletionFailure::Other))
}

fn connect_first(addresses: &[std::net::SocketAddr]) -> Option<TcpStream> {
    addresses
        .iter()
        .find_map(|address| TcpStream::connect_timeout(address, CONNECT_TIMEOUT).ok())
}

fn map_prepare_connect_failure(
    failure: PrepareBeforeConnectError<TransportFailure, TransportFailure>,
) -> TransportFailure {
    match failure {
        PrepareBeforeConnectError::Preparation(failure)
        | PrepareBeforeConnectError::Connection(failure) => failure,
    }
}

fn map_diagnostic_prepare_connect_failure(
    failure: PrepareBeforeConnectError<NoiseDiagnosticFailure, NoiseDiagnosticFailure>,
) -> NoiseDiagnosticFailure {
    match failure {
        PrepareBeforeConnectError::Preparation(failure)
        | PrepareBeforeConnectError::Connection(failure) => failure,
    }
}

fn elapsed_ms(started: std::time::Instant) -> u32 {
    started.elapsed().as_millis().try_into().unwrap_or(u32::MAX)
}

fn run_encrypted_loop(
    mut stream: TcpStream,
    mut noise: NoiseTransport,
    receiver: &Receiver<TransportCommand>,
    emit: &impl Fn(TransportEvent),
) -> Result<(), TransportFailure> {
    let mut receive = ReceiveState::new();
    loop {
        loop {
            match receiver.try_recv() {
                Ok(TransportCommand::Send(frame)) => {
                    let encrypted = noise
                        .encrypt_frame(&frame)
                        .map_err(|_| TransportFailure::Frame)?;
                    stream
                        .write_all(&encrypted)
                        .map_err(|_| TransportFailure::Write)?;
                }
                Ok(TransportCommand::Close) | Err(TryRecvError::Disconnected) => return Ok(()),
                Err(TryRecvError::Empty) => break,
            }
        }

        let mut chunk = [0; 512];
        match stream.read(&mut chunk) {
            Ok(0) => return Ok(()),
            Ok(count) => {
                receive.push(&chunk[..count])?;
                while let Some(frame) = receive.next_frame(&mut noise)? {
                    let message =
                        ServerMessage::decode(&frame).map_err(|_| TransportFailure::Frame)?;
                    emit(TransportEvent::Message(message));
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(_) => return Err(TransportFailure::Read),
        }
    }
}

struct ReceiveState {
    bytes: Vec<u8>,
    maybe_header: Option<DecryptedNoiseHeader>,
}

impl ReceiveState {
    fn new() -> Self {
        Self {
            bytes: Vec::with_capacity(ENCRYPTED_BUFFER_CAPACITY),
            maybe_header: None,
        }
    }

    fn push(&mut self, bytes: &[u8]) -> Result<(), TransportFailure> {
        if self.bytes.len().saturating_add(bytes.len()) > ENCRYPTED_BUFFER_CAPACITY {
            return Err(TransportFailure::Frame);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn next_frame(
        &mut self,
        noise: &mut NoiseTransport,
    ) -> Result<Option<Frame>, TransportFailure> {
        if self.maybe_header.is_none() {
            if self.bytes.len() < ENCRYPTED_HEADER_LEN {
                return Ok(None);
            }
            let encrypted_header = self.bytes.drain(..ENCRYPTED_HEADER_LEN).collect::<Vec<_>>();
            self.maybe_header = Some(
                noise
                    .decrypt_header(&encrypted_header)
                    .map_err(|_| TransportFailure::Frame)?,
            );
        }
        let pending = self.maybe_header.expect("header was established");
        let payload_len = pending.encrypted_payload_len();
        if self.bytes.len() < payload_len {
            return Ok(None);
        }
        let encrypted_payload = self.bytes.drain(..payload_len).collect::<Vec<_>>();
        self.maybe_header = None;
        noise
            .decrypt_payload(pending, &encrypted_payload)
            .map(Some)
            .map_err(|_| TransportFailure::Frame)
    }
}

struct EspHardwareRng;

impl RngCore for EspHardwareRng {
    fn next_u32(&mut self) -> u32 {
        // SAFETY: ESP-IDF's hardware RNG function has no preconditions.
        unsafe { sys::esp_random() }
    }

    fn next_u64(&mut self) -> u64 {
        (u64::from(self.next_u32()) << 32) | u64::from(self.next_u32())
    }

    fn fill_bytes(&mut self, destination: &mut [u8]) {
        for chunk in destination.chunks_mut(4) {
            let bytes = self.next_u32().to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }

    fn try_fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(destination);
        Ok(())
    }
}

impl CryptoRng for EspHardwareRng {}
