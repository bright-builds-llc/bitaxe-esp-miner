//! Historical boot-only Noise diagnostic source, deliberately not a Rust module.
//! The current firmware has no NVS admission or owner route to this code.
//! Retained for historical source readers; this file grants no execution authority.
use super::*;
use bitaxe_stratum::v2::noise::{NoiseCompletionFailure, NoisePreparationStage};
const DIAGNOSTIC_PROOF_EXTENSION: u16 = 0xffff;
const DIAGNOSTIC_PROOF_MESSAGE: u8 = 0xff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoiseDiagnosticStage {
    MonitorArmed,
    NoisePrepared,
    TcpConnected,
    ActOneCreated,
    ActOneSent,
    ActTwoReceived,
    TimeSampled,
    Authenticated,
    EncryptedProofSent,
}

impl NoiseDiagnosticStage {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::MonitorArmed => "monitor_armed",
            Self::NoisePrepared => "noise_prepared",
            Self::TcpConnected => "tcp_connected",
            Self::ActOneCreated => "act_one_created",
            Self::ActOneSent => "act_one_sent",
            Self::ActTwoReceived => "act_two_received",
            Self::TimeSampled => "time_sampled",
            Self::Authenticated => "authenticated",
            Self::EncryptedProofSent => "encrypted_proof_sent",
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
    ProofWrite,
}

impl NoiseDiagnosticTimingKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::KeypairPreparation => "keypair_preparation",
            Self::ActOneConstruction => "act_one_construction",
            Self::Connect => "connect",
            Self::ActOneWrite => "act_one_write",
            Self::ActTwoRead => "act_two_read",
            Self::ProofWrite => "proof_write",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoiseDiagnosticEvent {
    Stage(NoiseDiagnosticStage),
    Timing(NoiseDiagnosticTimingKind, u32),
    LocalPort(u16),
    SocketError {
        phase: &'static str,
        category: &'static str,
    },
    ActOneBytesWritten(u16),
    ProofBytesWritten(u16),
}

pub(crate) fn run_noise_diagnostic(
    settings: V2PoolSettings,
    mut emit: impl FnMut(NoiseDiagnosticEvent),
) -> Result<(), NoiseDiagnosticFailure> {
    if settings.maybe_authority_public_key.is_none() {
        return Err(NoiseDiagnosticFailure::Configure);
    }
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
                        emit(NoiseDiagnosticEvent::Timing(
                            NoiseDiagnosticTimingKind::KeypairPreparation,
                            keypair_preparation_ms,
                        ));
                    }
                    NoisePreparationStage::ActOneReady => {
                        let total_ms = elapsed_ms(preparation_started);
                        emit(NoiseDiagnosticEvent::Timing(
                            NoiseDiagnosticTimingKind::ActOneConstruction,
                            total_ms.saturating_sub(keypair_preparation_ms),
                        ));
                    }
                },
            )
            .map_err(|_| NoiseDiagnosticFailure::Rng)?;
            if preparation_started.elapsed() > MAX_NOISE_PREPARATION {
                return Err(NoiseDiagnosticFailure::PreparationSlow);
            }
            emit(NoiseDiagnosticEvent::Stage(
                NoiseDiagnosticStage::NoisePrepared,
            ));
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
    emit(NoiseDiagnosticEvent::Timing(
        NoiseDiagnosticTimingKind::Connect,
        connect_ms,
    ));
    emit(NoiseDiagnosticEvent::Stage(
        NoiseDiagnosticStage::TcpConnected,
    ));
    let local_port = stream
        .local_addr()
        .map_err(|_| NoiseDiagnosticFailure::Configure)?
        .port();
    emit(NoiseDiagnosticEvent::LocalPort(local_port));
    emit_socket_error(&stream, "pre_act_one", &mut emit);
    stream
        .set_nodelay(true)
        .map_err(|_| NoiseDiagnosticFailure::Configure)?;
    stream
        .set_read_timeout(Some(HANDSHAKE_TIMEOUT))
        .map_err(|_| NoiseDiagnosticFailure::Configure)?;
    stream
        .set_write_timeout(Some(WRITE_TIMEOUT))
        .map_err(|_| NoiseDiagnosticFailure::Configure)?;
    emit(NoiseDiagnosticEvent::Stage(
        NoiseDiagnosticStage::ActOneCreated,
    ));
    let write_started = std::time::Instant::now();
    stream
        .write_all(prepared.act_one())
        .map_err(|_| NoiseDiagnosticFailure::ActOneWrite)?;
    stream
        .flush()
        .map_err(|_| NoiseDiagnosticFailure::ActOneWrite)?;
    emit(NoiseDiagnosticEvent::ActOneBytesWritten(
        prepared.act_one().len().try_into().unwrap_or(u16::MAX),
    ));
    emit_socket_error(&stream, "post_act_one", &mut emit);
    emit(NoiseDiagnosticEvent::Timing(
        NoiseDiagnosticTimingKind::ActOneWrite,
        elapsed_ms(write_started),
    ));
    emit(NoiseDiagnosticEvent::Stage(
        NoiseDiagnosticStage::ActOneSent,
    ));
    let mut act_two = [0; ACT_TWO_LEN];
    let read_started = std::time::Instant::now();
    let read_result = stream.read_exact(&mut act_two);
    emit(NoiseDiagnosticEvent::Timing(
        NoiseDiagnosticTimingKind::ActTwoRead,
        elapsed_ms(read_started),
    ));
    read_result.map_err(|_| NoiseDiagnosticFailure::ActTwoRead)?;
    emit(NoiseDiagnosticEvent::Stage(
        NoiseDiagnosticStage::ActTwoReceived,
    ));
    emit_socket_error(&stream, "post_act_two", &mut emit);
    let unix_time_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| NoiseDiagnosticFailure::ClockBeforeEpoch)?
        .as_secs()
        .try_into()
        .map_err(|_| NoiseDiagnosticFailure::ClockOverflow)?;
    emit(NoiseDiagnosticEvent::Stage(
        NoiseDiagnosticStage::TimeSampled,
    ));
    let mut noise = prepared
        .complete_diagnostic(&act_two, unix_time_seconds)
        .map_err(NoiseDiagnosticFailure::Completion)?;
    emit(NoiseDiagnosticEvent::Stage(
        NoiseDiagnosticStage::Authenticated,
    ));
    let proof = Frame::new(
        DIAGNOSTIC_PROOF_EXTENSION,
        DIAGNOSTIC_PROOF_MESSAGE,
        Vec::new(),
    )
    .map_err(|_| NoiseDiagnosticFailure::Completion(NoiseCompletionFailure::Other))?;
    let encrypted = noise
        .encrypt_frame(&proof)
        .map_err(|_| NoiseDiagnosticFailure::Completion(NoiseCompletionFailure::Other))?;
    let proof_started = std::time::Instant::now();
    stream
        .write_all(&encrypted)
        .map_err(|_| NoiseDiagnosticFailure::Completion(NoiseCompletionFailure::Other))?;
    stream
        .flush()
        .map_err(|_| NoiseDiagnosticFailure::Completion(NoiseCompletionFailure::Other))?;
    emit(NoiseDiagnosticEvent::ProofBytesWritten(
        encrypted.len().try_into().unwrap_or(u16::MAX),
    ));
    emit_socket_error(&stream, "post_proof", &mut emit);
    emit(NoiseDiagnosticEvent::Timing(
        NoiseDiagnosticTimingKind::ProofWrite,
        elapsed_ms(proof_started),
    ));
    emit(NoiseDiagnosticEvent::Stage(
        NoiseDiagnosticStage::EncryptedProofSent,
    ));
    Ok(())
}

fn emit_socket_error(
    stream: &TcpStream,
    phase: &'static str,
    emit: &mut impl FnMut(NoiseDiagnosticEvent),
) {
    let category = match stream.take_error() {
        Ok(None) => "none",
        Ok(Some(error)) => socket_error_category(error.kind()),
        Err(_) => "query_failed",
    };
    emit(NoiseDiagnosticEvent::SocketError { phase, category });
}

fn socket_error_category(kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::WouldBlock => "would_block",
        io::ErrorKind::NotConnected => "not_connected",
        io::ErrorKind::OutOfMemory => "out_of_memory",
        io::ErrorKind::InvalidInput => "invalid_input",
        io::ErrorKind::Unsupported => "unsupported",
        io::ErrorKind::ConnectionAborted => "connection_aborted",
        io::ErrorKind::ConnectionReset => "connection_reset",
        io::ErrorKind::BrokenPipe => "broken_pipe",
        io::ErrorKind::TimedOut => "timed_out",
        _ => "other",
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

