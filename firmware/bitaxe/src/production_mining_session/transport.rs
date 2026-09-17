//! Bounded per-pool TCP workers for the production mining owner.

#[path = "transport/borrow.rs"]
pub(crate) mod borrow;
use borrow::{NoiseBorrowHandle, NoiseBorrowWorker};
#[path = "transport/v2.rs"]
mod v2;

use super::revocation::{self, WorkPermit};
use std::fmt;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, TrySendError};
use std::time::Duration;

use bitaxe_stratum::v1::production_session::{
    ProductionPool, ProductionPoolEndpoint, ProductionTransportEpoch, ProductionTransportFailure,
};

const COMMAND_CAPACITY: usize = 8;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_millis(50);
const WRITE_TIMEOUT: Duration = Duration::from_millis(50);
const READ_BUFFER_BYTES: usize = 2 * 1024;
const WORKER_STACK_BYTES: usize = 12 * 1024;

#[derive(Clone, PartialEq, Eq)]
pub(super) enum PoolTransportCommand {
    Connect {
        transport_epoch: ProductionTransportEpoch,
        endpoint: ProductionPoolEndpoint,
    },
    Write {
        transport_epoch: ProductionTransportEpoch,
        line: String,
        permit: WorkPermit,
    },
    Close {
        transport_epoch: ProductionTransportEpoch,
    },
    Shutdown,
    ConnectV2 {
        transport_epoch: ProductionTransportEpoch,
        endpoint: std::net::SocketAddrV4,
        authority: [u8; 32],
        permit: WorkPermit,
        generation: bitaxe_stratum::v1::production_work::PoolSessionGeneration,
    },
    WriteFrame {
        transport_epoch: ProductionTransportEpoch,
        frame: bitaxe_stratum::v2::frame::Frame,
        permit: WorkPermit,
    },
    Diagnostic,
}

impl fmt::Debug for PoolTransportCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect {
                transport_epoch, ..
            } => formatter
                .debug_struct("PoolTransportCommand::Connect")
                .field("transport_epoch", transport_epoch)
                .field("endpoint", &"redacted")
                .finish(),
            Self::Write {
                transport_epoch, ..
            } => formatter
                .debug_struct("PoolTransportCommand::Write")
                .field("transport_epoch", transport_epoch)
                .field("line", &"redacted")
                .finish(),
            Self::Close { transport_epoch } => formatter
                .debug_struct("PoolTransportCommand::Close")
                .field("transport_epoch", transport_epoch)
                .finish(),
            Self::Shutdown => formatter.write_str("PoolTransportCommand::Shutdown"),
            Self::Diagnostic => formatter.write_str("PoolTransportCommand::Diagnostic"),
            Self::ConnectV2 { .. } => {
                formatter.write_str("PoolTransportCommand::ConnectV2(redacted)")
            }
            Self::WriteFrame { .. } => {
                formatter.write_str("PoolTransportCommand::WriteFrame(redacted)")
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum PoolTransportEvent {
    FrameWritten {
        pool: ProductionPool,
        transport_epoch: ProductionTransportEpoch,
        sequence: u32,
    },
    Frame {
        pool: ProductionPool,
        transport_epoch: ProductionTransportEpoch,
        frame: bitaxe_stratum::v2::frame::Frame,
    },
    Connected {
        pool: ProductionPool,
        transport_epoch: ProductionTransportEpoch,
    },
    Failed {
        pool: ProductionPool,
        transport_epoch: ProductionTransportEpoch,
        failure: ProductionTransportFailure,
    },
    Bytes {
        pool: ProductionPool,
        transport_epoch: ProductionTransportEpoch,
        bytes: Vec<u8>,
    },
    Closed {
        pool: ProductionPool,
        transport_epoch: ProductionTransportEpoch,
    },
}

impl fmt::Debug for PoolTransportEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameWritten { .. } => formatter.write_str("PoolTransportEvent::FrameWritten"),
            Self::Frame { .. } => formatter.write_str("PoolTransportEvent::Frame(redacted)"),
            Self::Bytes {
                pool,
                transport_epoch,
                bytes,
            } => formatter
                .debug_struct("PoolTransportEvent::Bytes")
                .field("pool", pool)
                .field("transport_epoch", transport_epoch)
                .field("byte_count", &bytes.len())
                .finish(),
            Self::Connected {
                pool,
                transport_epoch,
            } => formatter
                .debug_struct("PoolTransportEvent::Connected")
                .field("pool", pool)
                .field("transport_epoch", transport_epoch)
                .finish(),
            Self::Failed {
                pool,
                transport_epoch,
                failure,
            } => formatter
                .debug_struct("PoolTransportEvent::Failed")
                .field("pool", pool)
                .field("transport_epoch", transport_epoch)
                .field("failure", failure)
                .finish(),
            Self::Closed {
                pool,
                transport_epoch,
            } => formatter
                .debug_struct("PoolTransportEvent::Closed")
                .field("pool", pool)
                .field("transport_epoch", transport_epoch)
                .finish(),
        }
    }
}

pub(super) struct PoolTransportWorkers {
    primary: PoolTransportWorkerHandle,
    fallback: PoolTransportWorkerHandle,
}

impl PoolTransportWorkers {
    pub(super) fn spawn(
        emit: impl Fn(PoolTransportEvent) + Send + Sync + 'static,
    ) -> io::Result<Self> {
        let emit = std::sync::Arc::new(emit);
        let primary = spawn_worker(ProductionPool::Primary, emit.clone())?;
        let fallback = spawn_worker(ProductionPool::Fallback, emit)?;
        if !crate::noise_serial_runtime::install_transport(
            std::sync::Arc::downgrade(&primary.borrow),
            std::sync::Arc::downgrade(&fallback.borrow),
        ) {
            return Err(io::Error::other("noise_borrow_registration"));
        }
        Ok(Self { primary, fallback })
    }

    pub(super) fn try_send(
        &self,
        pool: ProductionPool,
        command: PoolTransportCommand,
    ) -> Result<(), TrySendError<PoolTransportCommand>> {
        match pool {
            ProductionPool::Primary => self.primary.borrow.send(command),
            ProductionPool::Fallback => self.fallback.borrow.send(command),
        }
    }

    pub(super) fn request_close(
        &self,
        pool: ProductionPool,
        transport_epoch: ProductionTransportEpoch,
    ) -> Result<(), TrySendError<PoolTransportCommand>> {
        let worker = match pool {
            ProductionPool::Primary => &self.primary,
            ProductionPool::Fallback => &self.fallback,
        };
        worker
            .requested_close_epoch
            .store(epoch_word(transport_epoch), Ordering::Release);
        let command = PoolTransportCommand::Close { transport_epoch };
        match worker.borrow.send(command) {
            Ok(()) | Err(TrySendError::Full(_)) => Ok(()),
            Err(error @ TrySendError::Disconnected(_)) => Err(error),
        }
    }
}

impl Drop for PoolTransportWorkers {
    fn drop(&mut self) {
        if self
            .primary
            .borrow
            .send(PoolTransportCommand::Shutdown)
            .is_err()
        {
            log::warn!("pool_transport_shutdown=degraded pool=primary");
        }
        if self
            .fallback
            .borrow
            .send(PoolTransportCommand::Shutdown)
            .is_err()
        {
            log::warn!("pool_transport_shutdown=degraded pool=fallback");
        }
    }
}

struct PoolTransportWorkerHandle {
    borrow: std::sync::Arc<NoiseBorrowHandle>,
    requested_close_epoch: std::sync::Arc<AtomicU32>,
}

fn spawn_worker(
    pool: ProductionPool,
    emit: std::sync::Arc<impl Fn(PoolTransportEvent) + Send + Sync + 'static>,
) -> io::Result<PoolTransportWorkerHandle> {
    // One reusable slot keeps large receive-result/command values off the
    // persistent caller stack while the lane runs borrowed cryptography.
    let mut command_slot = Vec::new();
    command_slot
        .try_reserve_exact(1)
        .map_err(|_| io::Error::from(io::ErrorKind::OutOfMemory))?;
    let (sender, receiver) = mpsc::sync_channel(COMMAND_CAPACITY);
    let borrow = std::sync::Arc::new(NoiseBorrowHandle::new(sender));
    let worker_borrow = borrow.worker();
    let requested_close_epoch = std::sync::Arc::new(AtomicU32::new(0));
    let worker_requested_close_epoch = requested_close_epoch.clone();
    std::thread::Builder::new()
        .name(match pool {
            ProductionPool::Primary => "stratum-primary".to_owned(),
            ProductionPool::Fallback => "stratum-fallback".to_owned(),
        })
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || {
            run_worker(
                pool,
                receiver,
                command_slot,
                &worker_requested_close_epoch,
                &worker_borrow,
                move |event| emit(event),
            );
        })?;
    Ok(PoolTransportWorkerHandle {
        borrow,
        requested_close_epoch,
    })
}

#[inline(never)]
fn run_worker(
    pool: ProductionPool,
    receiver: Receiver<PoolTransportCommand>,
    mut command_slot: Vec<PoolTransportCommand>,
    requested_close_epoch: &AtomicU32,
    borrow: &NoiseBorrowWorker,
    emit: impl Fn(PoolTransportEvent),
) {
    let mut maybe_connection: Option<PoolConnection> = None;
    borrow.ready();
    loop {
        let kind = match receive_command(&receiver, &mut command_slot, maybe_connection.is_some()) {
            Ok(kind) => kind,
            Err(()) => {
                borrow.unavailable();
                return;
            }
        };

        honor_requested_close(requested_close_epoch, &mut maybe_connection);
        if kind != NextCommand::Idle {
            borrow.begin_command();
            let keep_running = match kind {
                NextCommand::Diagnostic => maybe_connection.is_none() && borrow.run_job(),
                NextCommand::V2 => {
                    close_connection(&mut maybe_connection);
                    v2::run(
                        pool,
                        command_slot.first().expect("classified V2 command"),
                        &receiver,
                        requested_close_epoch,
                        borrow,
                        &emit,
                    )
                }
                NextCommand::Ordinary => {
                    apply_stored_command(pool, &mut command_slot, &mut maybe_connection, &emit)
                }
                NextCommand::Idle => true,
            };
            command_slot.clear();
            borrow.finish_command(maybe_connection.is_some());
            if kind == NextCommand::V2 {
                crate::v2_serial_runtime::share_completed();
            }
            if !keep_running {
                borrow.unavailable();
                return;
            }
        }
        honor_requested_close(requested_close_epoch, &mut maybe_connection);
        if let Some(connection) = maybe_connection.as_mut() {
            poll_connection(pool, connection, &emit);
            if connection.closed {
                maybe_connection = None;
            }
        }
        borrow.connection(maybe_connection.is_some());
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NextCommand {
    Idle,
    Diagnostic,
    V2,
    Ordinary,
}
#[inline(never)]
fn receive_command(
    receiver: &Receiver<PoolTransportCommand>,
    slot: &mut Vec<PoolTransportCommand>,
    connected: bool,
) -> Result<NextCommand, ()> {
    let maybe_command = if connected {
        match receiver.recv_timeout(READ_TIMEOUT) {
            Ok(c) => Some(c),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(_) => return Err(()),
        }
    } else {
        Some(receiver.recv().map_err(|_| ())?)
    };
    let Some(command) = maybe_command else {
        return Ok(NextCommand::Idle);
    };
    if matches!(command, PoolTransportCommand::Diagnostic) {
        return Ok(NextCommand::Diagnostic);
    }
    let kind = if matches!(command, PoolTransportCommand::ConnectV2 { .. }) {
        NextCommand::V2
    } else {
        NextCommand::Ordinary
    };
    slot.push(command);
    Ok(kind)
}
#[inline(never)]
fn apply_stored_command(
    pool: ProductionPool,
    slot: &mut Vec<PoolTransportCommand>,
    connection: &mut Option<PoolConnection>,
    emit: &impl Fn(PoolTransportEvent),
) -> bool {
    let Some(command) = slot.pop() else {
        return false;
    };
    apply_command(pool, command, connection, emit)
}

fn honor_requested_close(
    requested_close_epoch: &AtomicU32,
    maybe_connection: &mut Option<PoolConnection>,
) {
    let requested = requested_close_epoch.load(Ordering::Acquire);
    if maybe_connection
        .as_ref()
        .is_some_and(|connection| epoch_word(connection.transport_epoch) == requested)
    {
        close_connection(maybe_connection);
    }
}

fn epoch_word(transport_epoch: ProductionTransportEpoch) -> u32 {
    u32::try_from(transport_epoch.raw()).unwrap_or(u32::MAX)
}

struct PoolConnection {
    transport_epoch: ProductionTransportEpoch,
    stream: TcpStream,
    closed: bool,
}

#[inline(never)]
fn apply_command(
    pool: ProductionPool,
    command: PoolTransportCommand,
    maybe_connection: &mut Option<PoolConnection>,
    emit: &impl Fn(PoolTransportEvent),
) -> bool {
    match command {
        PoolTransportCommand::Connect {
            transport_epoch,
            endpoint,
        } => {
            close_connection(maybe_connection);
            match connect(&endpoint) {
                Ok(stream) => {
                    *maybe_connection = Some(PoolConnection {
                        transport_epoch,
                        stream,
                        closed: false,
                    });
                    emit(PoolTransportEvent::Connected {
                        pool,
                        transport_epoch,
                    });
                }
                Err(_) => emit(PoolTransportEvent::Failed {
                    pool,
                    transport_epoch,
                    failure: ProductionTransportFailure::Connect,
                }),
            }
        }
        PoolTransportCommand::Write {
            transport_epoch,
            line,
            permit,
        } => {
            let Some(connection) = maybe_connection.as_mut() else {
                emit(PoolTransportEvent::Failed {
                    pool,
                    transport_epoch,
                    failure: ProductionTransportFailure::Write,
                });
                return true;
            };
            if connection.transport_epoch != transport_epoch {
                return true;
            }
            if write_admitted_line(&mut connection.stream, line.as_bytes(), permit).is_err()
                || connection.stream.flush().is_err()
            {
                emit(PoolTransportEvent::Failed {
                    pool,
                    transport_epoch,
                    failure: ProductionTransportFailure::Write,
                });
                close_connection(maybe_connection);
            } else if serde_json::from_str::<serde_json::Value>(&line)
                .ok()
                .and_then(|value| {
                    value
                        .get("method")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .is_some_and(|method| method == "mining.submit")
            {
                revocation::note_submission(permit.maybe_generation());
            }
        }
        PoolTransportCommand::Close { transport_epoch } => {
            if maybe_connection
                .as_ref()
                .is_some_and(|connection| connection.transport_epoch == transport_epoch)
            {
                close_connection(maybe_connection);
            }
        }
        PoolTransportCommand::ConnectV2 { .. } | PoolTransportCommand::WriteFrame { .. } => {
            return false
        }
        PoolTransportCommand::Diagnostic => return false,
        PoolTransportCommand::Shutdown => {
            close_connection(maybe_connection);
            return false;
        }
    }
    true
}

fn write_admitted_line(
    stream: &mut impl Write,
    remaining: &[u8],
    permit: WorkPermit,
) -> io::Result<()> {
    write_while_admitted(stream, remaining, || revocation::permits_work(permit))
}

fn write_while_admitted(
    stream: &mut impl Write,
    mut remaining: &[u8],
    allowed: impl Fn() -> bool,
) -> io::Result<()> {
    while !remaining.is_empty() {
        if !allowed() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "production_generation_revoked",
            ));
        }
        let count = stream.write(remaining)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "production_pool_write_zero",
            ));
        }
        remaining = &remaining[count..];
    }
    if !allowed() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "production_generation_revoked",
        ));
    }
    Ok(())
}

fn connect(endpoint: &ProductionPoolEndpoint) -> io::Result<TcpStream> {
    let mut last_error = None;
    for address in (endpoint.host.as_str(), endpoint.port).to_socket_addrs()? {
        match TcpStream::connect_timeout(&address, CONNECT_TIMEOUT) {
            Ok(stream) => {
                stream.set_read_timeout(Some(READ_TIMEOUT))?;
                stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
                stream.set_nodelay(true)?;
                return Ok(stream);
            }
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::AddrNotAvailable, "pool address unavailable")
    }))
}

#[inline(never)]
fn poll_connection(
    pool: ProductionPool,
    connection: &mut PoolConnection,
    emit: &impl Fn(PoolTransportEvent),
) {
    let mut buffer = [0_u8; READ_BUFFER_BYTES];
    match connection.stream.read(&mut buffer) {
        Ok(0) => {
            connection.closed = true;
            emit(PoolTransportEvent::Closed {
                pool,
                transport_epoch: connection.transport_epoch,
            });
        }
        Ok(count) => emit(PoolTransportEvent::Bytes {
            pool,
            transport_epoch: connection.transport_epoch,
            bytes: buffer[..count].to_vec(),
        }),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
            ) => {}
        Err(_) => {
            connection.closed = true;
            emit(PoolTransportEvent::Failed {
                pool,
                transport_epoch: connection.transport_epoch,
                failure: ProductionTransportFailure::Read,
            });
        }
    }
}

fn close_connection(maybe_connection: &mut Option<PoolConnection>) {
    if let Some(connection) = maybe_connection.take() {
        if connection.stream.shutdown(Shutdown::Both).is_err() {
            log::warn!("pool_transport_close=degraded");
        }
    }
}

#[cfg(test)]
#[path = "transport/tests.rs"]
mod tests;
