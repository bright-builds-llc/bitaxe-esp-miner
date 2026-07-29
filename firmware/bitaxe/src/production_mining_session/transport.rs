//! Bounded per-pool TCP workers for the production mining owner.

use std::fmt;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::time::Duration;

use bitaxe_stratum::v1::production_session::{
    ProductionPool, ProductionPoolEndpoint, ProductionTransportEpoch, ProductionTransportFailure,
};

const COMMAND_CAPACITY: usize = 8;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_millis(50);
const WRITE_TIMEOUT: Duration = Duration::from_secs(2);
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
    },
    Close {
        transport_epoch: ProductionTransportEpoch,
    },
    Shutdown,
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
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum PoolTransportEvent {
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
        Ok(Self { primary, fallback })
    }

    pub(super) fn try_send(
        &self,
        pool: ProductionPool,
        command: PoolTransportCommand,
    ) -> Result<(), TrySendError<PoolTransportCommand>> {
        match pool {
            ProductionPool::Primary => self.primary.sender.try_send(command),
            ProductionPool::Fallback => self.fallback.sender.try_send(command),
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
        match worker.sender.try_send(command) {
            Ok(()) | Err(TrySendError::Full(_)) => Ok(()),
            Err(error @ TrySendError::Disconnected(_)) => Err(error),
        }
    }
}

impl Drop for PoolTransportWorkers {
    fn drop(&mut self) {
        if self
            .primary
            .sender
            .try_send(PoolTransportCommand::Shutdown)
            .is_err()
        {
            log::warn!("pool_transport_shutdown=degraded pool=primary");
        }
        if self
            .fallback
            .sender
            .try_send(PoolTransportCommand::Shutdown)
            .is_err()
        {
            log::warn!("pool_transport_shutdown=degraded pool=fallback");
        }
    }
}

struct PoolTransportWorkerHandle {
    sender: SyncSender<PoolTransportCommand>,
    requested_close_epoch: std::sync::Arc<AtomicU32>,
}

fn spawn_worker(
    pool: ProductionPool,
    emit: std::sync::Arc<impl Fn(PoolTransportEvent) + Send + Sync + 'static>,
) -> io::Result<PoolTransportWorkerHandle> {
    let (sender, receiver) = mpsc::sync_channel(COMMAND_CAPACITY);
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
                &worker_requested_close_epoch,
                move |event| emit(event),
            );
        })?;
    Ok(PoolTransportWorkerHandle {
        sender,
        requested_close_epoch,
    })
}

fn run_worker(
    pool: ProductionPool,
    receiver: Receiver<PoolTransportCommand>,
    requested_close_epoch: &AtomicU32,
    emit: impl Fn(PoolTransportEvent),
) {
    let mut maybe_connection: Option<PoolConnection> = None;
    loop {
        let command = if maybe_connection.is_some() {
            match receiver.recv_timeout(READ_TIMEOUT) {
                Ok(command) => Some(command),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        } else {
            match receiver.recv() {
                Ok(command) => Some(command),
                Err(_) => return,
            }
        };

        honor_requested_close(requested_close_epoch, &mut maybe_connection);
        if let Some(command) = command {
            if !apply_command(pool, command, &mut maybe_connection, &emit) {
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
    }
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
            if connection.stream.write_all(line.as_bytes()).is_err()
                || connection.stream.flush().is_err()
            {
                emit(PoolTransportEvent::Failed {
                    pool,
                    transport_epoch,
                    failure: ProductionTransportFailure::Write,
                });
                close_connection(maybe_connection);
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
        PoolTransportCommand::Shutdown => {
            close_connection(maybe_connection);
            return false;
        }
    }
    true
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
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use super::*;

    fn next_epoch() -> ProductionTransportEpoch {
        ProductionTransportEpoch::initial().next()
    }

    #[test]
    fn loopback_worker_connects_writes_and_preserves_partial_bytes() {
        // Arrange
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener should bind");
        let address = listener
            .local_addr()
            .expect("loopback listener should expose its address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("worker should connect");
            let mut request = [0_u8; 5];
            stream
                .read_exact(&mut request)
                .expect("worker should write the complete line");
            assert_eq!(&request, b"ping\n");
            stream.write_all(b"{\"id\"").expect("first fragment");
            thread::sleep(Duration::from_millis(25));
            stream.write_all(b":1}\n").expect("second fragment");
        });
        let (event_sender, event_receiver) = mpsc::channel();
        let workers =
            PoolTransportWorkers::spawn(move |event| event_sender.send(event).expect("receiver"))
                .expect("workers should spawn");
        let epoch = next_epoch();

        // Act
        workers
            .try_send(
                ProductionPool::Primary,
                PoolTransportCommand::Connect {
                    transport_epoch: epoch,
                    endpoint: ProductionPoolEndpoint {
                        host: address.ip().to_string(),
                        port: address.port(),
                    },
                },
            )
            .expect("connect command should queue");
        assert_eq!(
            event_receiver
                .recv_timeout(Duration::from_secs(2))
                .expect("connected event"),
            PoolTransportEvent::Connected {
                pool: ProductionPool::Primary,
                transport_epoch: epoch,
            }
        );
        workers
            .try_send(
                ProductionPool::Primary,
                PoolTransportCommand::Write {
                    transport_epoch: epoch,
                    line: "ping\n".to_owned(),
                },
            )
            .expect("write command should queue");

        let mut received = Vec::new();
        while !received.ends_with(b"\n") {
            match event_receiver
                .recv_timeout(Duration::from_secs(2))
                .expect("transport event")
            {
                PoolTransportEvent::Bytes { bytes, .. } => received.extend(bytes),
                PoolTransportEvent::Closed { .. } if received.ends_with(b"\n") => break,
                other => panic!("unexpected event: {other:?}"),
            }
        }

        // Assert
        assert_eq!(received, b"{\"id\":1}\n");
        workers
            .request_close(ProductionPool::Primary, epoch)
            .expect("close request should register");
        server.join().expect("loopback server should finish");
    }

    #[test]
    fn failed_connect_is_typed_and_debug_output_is_redacted() {
        // Arrange
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener should bind");
        let address = listener
            .local_addr()
            .expect("loopback listener should expose its address");
        drop(listener);
        let (event_sender, event_receiver) = mpsc::channel();
        let workers =
            PoolTransportWorkers::spawn(move |event| event_sender.send(event).expect("receiver"))
                .expect("workers should spawn");
        let epoch = next_epoch();
        let command = PoolTransportCommand::Connect {
            transport_epoch: epoch,
            endpoint: ProductionPoolEndpoint {
                host: address.ip().to_string(),
                port: address.port(),
            },
        };

        // Act
        let debug = format!("{command:?}");
        workers
            .try_send(ProductionPool::Fallback, command)
            .expect("connect command should queue");
        let event = event_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("failed event");

        // Assert
        assert!(!debug.contains(&address.port().to_string()));
        assert!(debug.contains("redacted"));
        assert_eq!(
            event,
            PoolTransportEvent::Failed {
                pool: ProductionPool::Fallback,
                transport_epoch: epoch,
                failure: ProductionTransportFailure::Connect,
            }
        );
    }

    #[test]
    fn write_debug_never_contains_pool_line() {
        // Arrange
        let command = PoolTransportCommand::Write {
            transport_epoch: next_epoch(),
            line: "sensitive-owner-worker-value".to_owned(),
        };

        // Act
        let debug = format!("{command:?}");

        // Assert
        assert!(!debug.contains("sensitive-owner-worker-value"));
        assert!(debug.contains("redacted"));
    }
}
