use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::*;

#[test]
fn revoked_partial_pool_write_never_sends_its_remaining_payload() {
    // Arrange
    let gate = revocation::GenerationGate::new();
    let generation = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate(generation));
    let permit = gate.stamp(Some(generation));
    struct Writer<'a> {
        gate: &'a revocation::GenerationGate,
        generation: revocation::WorkerGeneration,
        writes: usize,
    }
    impl Write for Writer<'_> {
        fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
            self.writes += 1;
            self.gate.revoke(self.generation);
            Ok(1)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Writer {
        gate: &gate,
        generation,
        writes: 0,
    };
    // Act
    let result = write_while_admitted(&mut writer, b"closed-fixture\n", || {
        gate.permits_work(permit)
    });
    // Assert
    assert!(result.is_err());
    assert_eq!(writer.writes, 1);
}

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
    assert!(
        !workers.primary.borrow.reserve(),
        "connected worker cannot be borrowed"
    );
    workers
        .try_send(
            ProductionPool::Primary,
            PoolTransportCommand::Write {
                transport_epoch: epoch,
                line: "ping\n".to_owned(),
                permit: revocation::stamp(None),
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
        permit: revocation::stamp(None),
    };

    // Act
    let debug = format!("{command:?}");

    // Assert
    assert!(!debug.contains("sensitive-owner-worker-value"));
    assert!(debug.contains("redacted"));
}

#[test]
fn queued_or_inflight_ordinary_command_prevents_borrow_reservation() {
    // Arrange: use the real enqueue/dequeue gate with a controlled receiver.
    let (sender, receiver) = mpsc::sync_channel(8);
    let handle = NoiseBorrowHandle::new(sender);
    let worker = handle.worker();
    worker.ready();
    handle
        .send(PoolTransportCommand::Close {
            transport_epoch: next_epoch(),
        })
        .expect("queued");
    // Act / Assert
    assert!(!handle.reserve());
    let _command = receiver.recv().expect("dequeued");
    assert!(!handle.reserve());
    worker.begin_command();
    assert!(!handle.reserve());
    worker.finish_command(false);
    assert!(handle.reserve());
    assert!(handle.send(PoolTransportCommand::Shutdown).is_err());
    assert!(handle.release());
}

static BORROW_TEST: std::sync::OnceLock<std::sync::Mutex<Option<BorrowTest>>> =
    std::sync::OnceLock::new();
static JOB_DROPPED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static JOB_COMPLETED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
struct BorrowTest {
    entered: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
}
struct JobScope;
impl Drop for JobScope {
    fn drop(&mut self) {
        JOB_DROPPED.store(true, Ordering::Release);
    }
}
fn run_held_job() {
    let _scope = JobScope;
    let test = BORROW_TEST
        .get()
        .expect("test installed")
        .lock()
        .expect("test lock")
        .take()
        .expect("one job");
    test.entered.send(()).expect("test observes entry");
    test.release
        .recv_timeout(Duration::from_secs(3))
        .expect("bounded test release");
}
fn complete_held_job() {
    assert!(JOB_DROPPED.load(Ordering::Acquire));
    JOB_COMPLETED.store(true, Ordering::Release);
}
#[test]
fn existing_pool_worker_observes_scoped_job_return_before_completion_and_release() {
    // Arrange: actual twelve-KiB worker loop; no additional diagnostic thread.
    let (entered_sender, entered_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    assert!(BORROW_TEST
        .set(std::sync::Mutex::new(Some(BorrowTest {
            entered: entered_sender,
            release: release_receiver
        })))
        .is_ok());
    let workers = PoolTransportWorkers::spawn(|_| {}).expect("ordinary workers");
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while !workers.primary.borrow.reserve() {
        assert!(std::time::Instant::now() < deadline, "worker became idle");
        thread::sleep(Duration::from_millis(2));
    }
    // Act
    workers
        .primary
        .borrow
        .dispatch(run_held_job, complete_held_job)
        .expect("dispatch once");
    let entered = entered_receiver.recv_timeout(Duration::from_secs(2));
    let early_complete = JOB_COMPLETED.load(Ordering::Acquire);
    let early_release = workers.primary.borrow.release();
    let normal_rejected = workers
        .try_send(
            ProductionPool::Primary,
            PoolTransportCommand::Connect {
                transport_epoch: next_epoch(),
                endpoint: ProductionPoolEndpoint {
                    host: "127.0.0.1".into(),
                    port: 9,
                },
            },
        )
        .is_err();
    let released = release_sender.send(());
    while !JOB_COMPLETED.load(Ordering::Acquire) && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(2));
    }
    // Assert
    assert!(entered.is_ok() && released.is_ok());
    assert!(!early_complete && !early_release && normal_rejected);
    assert!(JOB_DROPPED.load(Ordering::Acquire));
    assert!(JOB_COMPLETED.load(Ordering::Acquire));
    assert!(workers.primary.borrow.release());
}
