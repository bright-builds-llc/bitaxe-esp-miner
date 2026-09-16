use super::*;
use std::net::TcpListener;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::Instant;

struct HeldRng {
    entered: mpsc::SyncSender<()>,
    maybe_release: Option<mpsc::Receiver<()>>,
}
impl RngCore for HeldRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }
    fn next_u64(&mut self) -> u64 {
        u64::from(self.next_u32())
    }
    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        if let Some(release) = self.maybe_release.take() {
            self.entered.send(()).expect("test observes entry");
            release
                .recv_timeout(Duration::from_secs(3))
                .expect("bounded release");
        }
        bytes.fill(1);
    }
    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(bytes);
        Ok(())
    }
}
impl CryptoRng for HeldRng {}
struct Observations {
    fail_reservation: bool,
    fail_act_two: bool,
    cancelled: Arc<AtomicBool>,
    began: Instant,
    operations: Vec<CryptoOperation>,
    events: Vec<Event>,
    failures: Vec<(Phase, Failure)>,
}
impl Observer for Observations {
    fn reserve_act_two(&mut self, slot: &mut Vec<u8>) -> Result<(), Failure> {
        slot.try_reserve_exact(if self.fail_act_two {
            usize::MAX
        } else {
            ACT_TWO_LEN
        })
        .map_err(|_| Failure::Allocation)
    }
    fn reserve_transport(
        &mut self,
        slot: &mut Vec<super::super::NoiseTransport>,
    ) -> Result<(), Failure> {
        slot.try_reserve_exact(if self.fail_reservation { usize::MAX } else { 1 })
            .map_err(|_| Failure::Allocation)
    }
    fn entering_crypto(&mut self, operation: CryptoOperation) {
        self.operations.push(operation);
    }
    fn permitted(&mut self) -> bool {
        !self.cancelled.load(Ordering::Acquire)
    }
    fn now_us(&self) -> Option<u64> {
        Some(1_000_000 + self.began.elapsed().as_micros() as u64)
    }
    fn event(&mut self, event: Event) {
        self.events.push(event);
    }
    fn failed(&mut self, phase: Phase, failure: Failure) {
        self.failures.push((phase, failure));
    }
}
#[test]
fn cancellation_during_real_initiator_call_allows_return_but_no_next_crypto_or_network_phase() {
    // Arrange: the actual helper and pinned Noise library reach a controlled entropy seam.
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    listener.set_nonblocking(true).expect("nonblocking");
    let endpoint = listener.local_addr().expect("endpoint");
    let (entered_sender, entered_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let cancelled = Arc::new(AtomicBool::new(false));
    let worker_cancelled = Arc::clone(&cancelled);
    let authority = [
        0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b,
        0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8,
        0x17, 0x98,
    ];
    let worker = std::thread::spawn(move || {
        let mut rng = HeldRng {
            entered: entered_sender,
            maybe_release: Some(release_receiver),
        };
        let mut observer = Observations {
            fail_reservation: false,
            fail_act_two: false,
            cancelled: worker_cancelled,
            began: Instant::now(),
            operations: Vec::new(),
            events: Vec::new(),
            failures: Vec::new(),
        };
        run(endpoint, authority, &mut rng, &mut observer);
        observer
    });
    // Act: revocation is independent; completion is proved only after release and join.
    let entered = entered_receiver.recv_timeout(Duration::from_secs(1));
    cancelled.store(true, Ordering::Release);
    let unfinished = !worker.is_finished();
    let released = release_sender.send(());
    let observer = worker.join().expect("actual completion");
    // Assert
    assert!(entered.is_ok() && released.is_ok() && unfinished);
    assert_eq!(
        observer.operations,
        [CryptoOperation::InitiatorConstruction]
    );
    assert_eq!(observer.failures, [(Phase::Prepare, Failure::Cancelled)]);
    assert!(observer
        .events
        .iter()
        .all(|event| matches!(event, Event::Cleanup)));
    assert!(listener
        .accept()
        .is_err_and(|error| error.kind() == std::io::ErrorKind::WouldBlock));
}

fn responder_failure_case(
    extra: bool,
    fail_reservation: bool,
    fail_act_two: bool,
) -> (usize, Observations) {
    // Arrange: real responder sends a valid act two plus an extra byte in one write.
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    listener
        .set_nonblocking(true)
        .expect("nonblocking listener");
    let endpoint = listener.local_addr().expect("endpoint");
    let public = [
        0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b,
        0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8,
        0x17, 0x98,
    ];
    let peer = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(2))
                }
                Err(error) => panic!("bounded fixture accept: {error}"),
            }
        };
        stream
            .set_nonblocking(false)
            .expect("blocking bounded fixture peer");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("read bound");
        let mut act_one = [0; 64];
        stream.read_exact(&mut act_one).expect("act one");
        if fail_act_two {
            return match stream.read(&mut [0; 22]) {
                Ok(count) => count,
                Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => 0,
                Err(error) => panic!("failed receive allocation must close: {error}"),
            };
        }
        let mut private = [0; 32];
        private[31] = 1;
        let (entered, _receiver) = mpsc::sync_channel(1);
        let mut rng = HeldRng {
            entered,
            maybe_release: None,
        };
        let mut responder = noise_sv2::Responder::from_authority_kp_with_rng(
            &public,
            &private,
            Duration::from_secs(u32::MAX.into()),
            &mut rng,
        )
        .expect("responder");
        let (act_two, _codec) = responder
            .step_1_with_now_rng(act_one, 0, &mut rng)
            .expect("act two");
        let mut oversized = act_two.to_vec();
        if extra {
            oversized.push(1);
        }
        stream.write_all(&oversized).expect("coalesced response");
        let mut proof = [0; 22];
        match stream.read(&mut proof) {
            Ok(count) => count,
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => 0,
            Err(error) => panic!("client closes without proof: {error}"),
        }
    });
    let (entered, _receiver) = mpsc::sync_channel(1);
    let mut rng = HeldRng {
        entered,
        maybe_release: None,
    };
    let mut observer = Observations {
        fail_reservation,
        fail_act_two,
        cancelled: Arc::new(AtomicBool::new(false)),
        began: Instant::now(),
        operations: Vec::new(),
        events: Vec::new(),
        failures: Vec::new(),
    };
    // Act
    run(endpoint, public, &mut rng, &mut observer);
    let proof_bytes = peer.join().expect("peer joined");
    (proof_bytes, observer)
}

#[test]
fn coalesced_extra_act_two_byte_prevents_authentication_and_proof() {
    // Arrange / Act
    let (proof_bytes, observer) = responder_failure_case(true, false, false);
    // Assert
    assert_eq!(proof_bytes, 0);
    assert!(observer.failures.contains(&(Phase::ActTwo, Failure::Extra)));
    assert!(!observer
        .operations
        .contains(&CryptoOperation::ActTwoAuthentication));
    assert!(!observer
        .operations
        .contains(&CryptoOperation::ProofEncryption));
}

#[test]
fn fallible_transport_storage_failure_prevents_authentication_and_protocol_success() {
    // Arrange / Act: actual try_reserve_exact fails at an injected impossible capacity.
    let (proof_bytes, observer) = responder_failure_case(false, true, false);
    // Assert
    assert_eq!(proof_bytes, 0);
    assert!(observer
        .failures
        .contains(&(Phase::Authenticate, Failure::Allocation)));
    assert!(!observer
        .operations
        .contains(&CryptoOperation::ActTwoAuthentication));
    assert!(!observer
        .operations
        .contains(&CryptoOperation::ProofEncryption));
}

#[test]
fn fallible_act_two_buffer_failure_closes_without_authentication_or_proof() {
    // Arrange / Act
    let (proof_bytes, observer) = responder_failure_case(false, false, true);
    // Assert
    assert_eq!(proof_bytes, 0);
    assert!(observer
        .failures
        .contains(&(Phase::ActTwo, Failure::Allocation)));
    assert!(!observer
        .operations
        .contains(&CryptoOperation::ActTwoAuthentication));
    assert!(!observer
        .operations
        .contains(&CryptoOperation::ProofEncryption));
}
