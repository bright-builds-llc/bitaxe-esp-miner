//! Shared production network-only diagnostic with checkpoints around opaque crypto.
use super::{NoiseCompletionFailure, NoiseInitiator, ACT_TWO_LEN};
use crate::v2::frame::Frame;
use rand::{CryptoRng, RngCore};
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Prepare,
    Connect,
    ActOne,
    ActTwo,
    Authenticate,
    Proof,
    Close,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CryptoOperation {
    InitiatorConstruction,
    ActOneConstruction,
    ActTwoAuthentication,
    ProofEncryption,
}
/// Actual invocation boundaries; idle socket waiting is not a frame read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    InitiatorConstruction,
    ActOneConstruction,
    Connect,
    ActOneWrite,
    ActTwoRead,
    ActTwoAuthentication,
    FrameEncrypt,
    FrameWrite,
    FrameRead,
    HeaderDecrypt,
    PayloadDecrypt,
    SocketClose,
    WorkerJoin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Cancelled,
    Clock,
    Timeout,
    Eof,
    Extra,
    Allocation,
    Io,
    Authentication(NoiseCompletionFailure),
    BeforeEpoch,
    TimeOverflow,
}
#[derive(Clone, Copy, Debug)]
pub enum Event {
    SocketClosedWithoutClock,
    Complete {
        phase: Phase,
        at_us: u64,
        duration_us: u64,
        bytes: Option<u16>,
    },
    SocketOpened(u16),
    Cleanup,
}
pub trait Observer {
    fn operation_started(&mut self, _operation: Operation) {}
    fn frame_write_completed(&mut self) {}
    /// Observes already received, authenticated plaintext before the final
    /// authority check. This hook grants no protocol I/O or work authority.
    fn frame_authenticated(&mut self, _frame: &Frame) -> Result<(), Failure> {
        Ok(())
    }
    fn operation_finished(&mut self, _operation: Operation, _failed: bool) {}
    fn connected(&mut self, _stream: &TcpStream) -> Result<(), Failure> {
        Ok(())
    }
    /// Continues on the same authenticated owner. The historical diagnostic
    /// keeps its exact reserved proof; Standard callers supply their protocol.
    fn authenticated(
        &mut self,
        stream: &mut TcpStream,
        noise: &mut super::NoiseTransport,
    ) -> Result<(), Failure>
    where
        Self: Sized,
    {
        encrypt_and_send_proof(stream, noise, self)
    }
    /// Bounds the receive allocation before reading any act-two bytes.
    fn reserve_act_two(&mut self, buffer: &mut Vec<u8>) -> Result<(), Failure> {
        buffer
            .try_reserve_exact(ACT_TWO_LEN)
            .map_err(|_| Failure::Allocation)
    }
    /// Uses the actual fallible allocation boundary; tests may inject its failure.
    fn reserve_transport(&mut self, slot: &mut Vec<super::NoiseTransport>) -> Result<(), Failure> {
        slot.try_reserve_exact(1).map_err(|_| Failure::Allocation)
    }
    /// Optional observation hook; it neither grants authority nor interrupts crypto.
    fn entering_crypto(&mut self, _operation: CryptoOperation) {}
    /// Includes native generation/session validity and the absolute authority deadline.
    fn permitted(&mut self) -> bool;
    fn now_us(&self) -> Option<u64>;
    fn event(&mut self, event: Event);
    fn failed(&mut self, phase: Phase, failure: Failure);
}
fn now(observer: &impl Observer) -> Result<u64, Failure> {
    observer.now_us().ok_or(Failure::Clock)
}
fn check(observer: &mut impl Observer) -> Result<(), Failure> {
    if observer.permitted() {
        Ok(())
    } else {
        Err(Failure::Cancelled)
    }
}
fn complete(
    observer: &mut impl Observer,
    phase: Phase,
    start: u64,
    limit: u64,
    bytes: Option<u16>,
) -> Result<(), Failure> {
    check(observer)?;
    let end = now(observer)?;
    let duration = end.checked_sub(start).ok_or(Failure::Clock)?;
    if duration > limit {
        return Err(Failure::Timeout);
    }
    observer.event(Event::Complete {
        phase,
        at_us: end,
        duration_us: duration,
        bytes,
    });
    Ok(())
}

/// Runs at most one connection and proof. Socket ownership ends before return.
pub fn run<R: RngCore + CryptoRng>(
    endpoint: SocketAddr,
    authority: [u8; 32],
    rng: &mut R,
    observer: &mut impl Observer,
) {
    let mut maybe_stream = None;
    let mut phase = Phase::Prepare;
    let result = exchange(
        endpoint,
        authority,
        rng,
        observer,
        &mut maybe_stream,
        &mut phase,
    );
    if let Err(failure) = result {
        observer.failed(phase, failure);
    }
    observer.event(Event::Cleanup);
    if let Some(stream) = maybe_stream.take() {
        let maybe_start = observer.now_us();
        observer.operation_started(Operation::SocketClose);
        let shutdown = stream.shutdown(Shutdown::Both);
        drop(stream);
        observer.operation_finished(Operation::SocketClose, shutdown.is_err());
        if shutdown.is_err_and(|error| error.kind() != std::io::ErrorKind::NotConnected) {
            observer.failed(Phase::Close, Failure::Io);
        }
        match (maybe_start, observer.now_us()) {
            (Some(start), Some(end)) if end >= start => observer.event(Event::Complete {
                phase: Phase::Close,
                at_us: end,
                duration_us: end - start,
                bytes: None,
            }),
            _ => {
                observer.failed(Phase::Close, Failure::Clock);
                observer.event(Event::SocketClosedWithoutClock);
            }
        }
    }
}

fn exchange<R: RngCore + CryptoRng>(
    endpoint: SocketAddr,
    authority: [u8; 32],
    rng: &mut R,
    observer: &mut impl Observer,
    maybe_stream: &mut Option<TcpStream>,
    phase: &mut Phase,
) -> Result<(), Failure> {
    // Act-one and connection scratch ends before the larger authentication call.
    let initiator = prepare_connected(endpoint, authority, rng, observer, maybe_stream, phase)?;
    let stream = maybe_stream.as_mut().ok_or(Failure::Io)?;
    *phase = Phase::ActTwo;
    let mut act_two = Zeroizing::new(Vec::new());
    observer.reserve_act_two(&mut act_two)?;
    if act_two.capacity() < ACT_TWO_LEN {
        return Err(Failure::Allocation);
    }
    act_two.resize(ACT_TWO_LEN, 0);
    let started = now(observer)?;
    observer.operation_started(Operation::ActTwoRead);
    let received = read_act_two(stream, &mut act_two, observer, started);
    observer.operation_finished(Operation::ActTwoRead, received.is_err());
    received?;
    complete(
        observer,
        Phase::ActTwo,
        started,
        10_000_000,
        Some(ACT_TWO_LEN as u16),
    )?;
    check(observer)?;
    reject_buffered_extra(stream)?;
    *phase = Phase::Authenticate;
    check(observer)?;
    let started = now(observer)?;
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Failure::BeforeEpoch)?
        .as_secs()
        .try_into()
        .map_err(|_| Failure::TimeOverflow)?;
    let mut noise = Vec::new();
    observer.reserve_transport(&mut noise)?;
    check(observer)?;
    observer.entering_crypto(CryptoOperation::ActTwoAuthentication);
    observer.operation_started(Operation::ActTwoAuthentication);
    check(observer)?;
    let authenticated = initiator.complete_diagnostic_into(
        act_two.as_slice().try_into().map_err(|_| Failure::Io)?,
        time,
        &mut noise,
    );
    observer.operation_finished(Operation::ActTwoAuthentication, authenticated.is_err());
    authenticated.map_err(Failure::Authentication)?;
    complete(observer, Phase::Authenticate, started, 120_000_000, None)?;
    *phase = Phase::ActTwo;
    check(observer)?;
    reject_buffered_extra(stream)?;
    *phase = Phase::Proof;
    observer.authenticated(stream, noise.first_mut().ok_or(Failure::Io)?)
}
// Keep preparation's act-one buffer and connection result out of the live
// authentication caller frame on the unchanged12-KiB pool worker.
#[inline(never)]
fn prepare_connected<R: RngCore + CryptoRng>(
    endpoint: SocketAddr,
    authority: [u8; 32],
    rng: &mut R,
    observer: &mut impl Observer,
    maybe_stream: &mut Option<TcpStream>,
    phase: &mut Phase,
) -> Result<NoiseInitiator, Failure> {
    check(observer)?;
    let prepare_start = now(observer)?;
    observer.entering_crypto(CryptoOperation::InitiatorConstruction);
    observer.operation_started(Operation::InitiatorConstruction);
    check(observer)?;
    let prepared = NoiseInitiator::new(Some(authority), rng);
    observer.operation_finished(Operation::InitiatorConstruction, prepared.is_err());
    let mut initiator =
        prepared.map_err(|_| Failure::Authentication(NoiseCompletionFailure::PublicKey))?;
    check(observer)?;
    if now(observer)?
        .checked_sub(prepare_start)
        .ok_or(Failure::Clock)?
        > 60_000_000
    {
        return Err(Failure::Timeout);
    }
    observer.entering_crypto(CryptoOperation::ActOneConstruction);
    observer.operation_started(Operation::ActOneConstruction);
    check(observer)?;
    let act_one = initiator.act_one();
    observer.operation_finished(Operation::ActOneConstruction, act_one.is_err());
    let act_one = act_one.map_err(|_| Failure::Authentication(NoiseCompletionFailure::Other))?;
    complete(observer, Phase::Prepare, prepare_start, 60_000_000, None)?;

    *phase = Phase::Connect;
    check(observer)?;
    let started = now(observer)?;
    observer.operation_started(Operation::Connect);
    let connected = TcpStream::connect_timeout(&endpoint, Duration::from_secs(5));
    observer.operation_finished(Operation::Connect, connected.is_err());
    *maybe_stream = Some(connected.map_err(|_| Failure::Io)?);
    let stream = maybe_stream.as_mut().ok_or(Failure::Io)?;
    // Even cancellation after connect records the actual owned socket before cleanup.
    observer.event(Event::SocketOpened(
        stream.local_addr().map_err(|_| Failure::Io)?.port(),
    ));
    observer.connected(stream)?;
    complete(observer, Phase::Connect, started, 5_000_000, None)?;
    stream.set_nonblocking(true).map_err(|_| Failure::Io)?;
    stream.set_nodelay(true).map_err(|_| Failure::Io)?;
    *phase = Phase::ActOne;
    observer.operation_started(Operation::ActOneWrite);
    let written = transfer_write(stream, &act_one, observer, Phase::ActOne);
    observer.operation_finished(Operation::ActOneWrite, written.is_err());
    written?;
    Ok(initiator)
}
fn read_act_two(
    stream: &mut TcpStream,
    output: &mut [u8],
    observer: &mut impl Observer,
    started: u64,
) -> Result<(), Failure> {
    let mut received = 0;
    while received < output.len() {
        io_checkpoint(observer, started, 10_000_000)?;
        match stream.read(&mut output[received..]) {
            Ok(0) => return Err(Failure::Eof),
            Ok(count) => received += count,
            Err(error) if retryable(&error) => std::thread::sleep(Duration::from_millis(2)),
            Err(_) => return Err(Failure::Io),
        }
    }
    Ok(())
}
#[inline(never)]
fn encrypt_and_send_proof(
    stream: &mut TcpStream,
    noise: &mut super::NoiseTransport,
    observer: &mut impl Observer,
) -> Result<(), Failure> {
    check(observer)?;
    let frame = Frame::new(0xffff, 0xff, Vec::new()).map_err(|_| Failure::Io)?;
    observer.entering_crypto(CryptoOperation::ProofEncryption);
    let proof = noise.encrypt_frame(&frame).map_err(|_| Failure::Io)?;
    check(observer)?;
    transfer_write(stream, &proof, observer, Phase::Proof)
}
fn transfer_write(
    stream: &mut TcpStream,
    bytes: &[u8],
    observer: &mut impl Observer,
    phase: Phase,
) -> Result<(), Failure> {
    check(observer)?;
    let started = now(observer)?;
    let mut written = 0;
    while written < bytes.len() {
        io_checkpoint(observer, started, 2_000_000)?;
        match stream.write(&bytes[written..]) {
            Ok(0) => return Err(Failure::Io),
            Ok(count) => written += count,
            Err(error) if retryable(&error) => std::thread::sleep(Duration::from_millis(2)),
            Err(_) => return Err(Failure::Io),
        }
    }
    loop {
        io_checkpoint(observer, started, 2_000_000)?;
        match stream.flush() {
            Ok(()) => break,
            Err(error) if retryable(&error) => std::thread::sleep(Duration::from_millis(2)),
            Err(_) => return Err(Failure::Io),
        }
    }
    complete(observer, phase, started, 2_000_000, Some(written as u16))
}
fn io_checkpoint(observer: &mut impl Observer, started: u64, limit: u64) -> Result<(), Failure> {
    check(observer)?;
    if now(observer)?.checked_sub(started).ok_or(Failure::Clock)? >= limit {
        return Err(Failure::Timeout);
    }
    Ok(())
}
fn retryable(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
    )
}
/// Observes only bytes already available now; it does not promise future silence.
fn reject_buffered_extra(stream: &TcpStream) -> Result<(), Failure> {
    match stream.peek(&mut [0; 1]) {
        Ok(0) => Err(Failure::Eof),
        Ok(_) => Err(Failure::Extra),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(()),
        Err(_) => Err(Failure::Io),
    }
}

#[cfg(test)]
mod tests;
