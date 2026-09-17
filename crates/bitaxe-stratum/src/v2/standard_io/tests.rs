use super::*;
use crate::v2::noise::{
    diagnostic::{Event, Phase},
    NoiseInitiator,
};
use noise_sv2::{NoiseCodec, Responder};
use rand::{CryptoRng, RngCore};
use std::net::TcpListener;

struct Capture {
    revoke_after: Operation,
    permitted: bool,
    received: Vec<Frame>,
}
impl Observer for Capture {
    fn permitted(&mut self) -> bool {
        self.permitted
    }
    fn now_us(&self) -> Option<u64> {
        Some(1000)
    }
    fn event(&mut self, _: Event) {}
    fn failed(&mut self, _: Phase, _: Failure) {}
    fn operation_finished(&mut self, operation: Operation, _: bool) {
        if operation == self.revoke_after {
            self.permitted = false;
        }
    }
    fn frame_authenticated(&mut self, frame: &Frame) -> Result<(), Failure> {
        self.received.push(frame.clone());
        Ok(())
    }
}
fn pair() -> (NoiseTransport, NoiseCodec) {
    let public = [
        0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b,
        0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8,
        0x17, 0x98,
    ];
    let mut private = [0; 32];
    private[31] = 1;
    let mut rng = TestRng(41);
    let mut initiator = NoiseInitiator::new(Some(public), &mut rng).expect("initiator");
    let mut responder =
        Responder::from_authority_kp_with_rng(&public, &private, Duration::from_secs(60), &mut rng)
            .expect("responder");
    let (act_two, codec) = responder
        .step_1_with_now_rng(initiator.act_one().expect("act one"), 100, &mut rng)
        .expect("act two");
    (
        initiator.complete(&act_two, 100).expect("authenticated"),
        codec,
    )
}
fn sockets() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let client = TcpStream::connect(listener.local_addr().expect("address")).expect("client");
    let (server, _) = listener.accept().expect("server");
    client.set_nonblocking(true).expect("nonblocking");
    (client, server)
}
fn send(codec: &mut NoiseCodec, stream: &mut TcpStream, tamper: bool) -> Frame {
    let frame = crate::v2::messages::SubmitSharesSuccess {
        channel_id: 9,
        last_sequence_number: 0,
        accepted_count: 1,
        shares_sum: 1024,
    }
    .encode()
    .expect("bounded actual ACK");
    let mut header = frame.header.encode().to_vec();
    let mut payload = frame.payload().to_vec();
    codec.encrypt(&mut header).expect("header");
    codec.encrypt(&mut payload).expect("payload");
    if tamper {
        payload[0] ^= 1;
    }
    stream.write_all(&header).expect("header sent");
    stream.write_all(&payload).expect("payload sent");
    frame
}
#[test]
fn fully_received_frame_survives_decrypt_boundary_revocation_without_further_io() {
    // Arrange
    let (mut noise, mut peer) = pair();
    let (mut client, mut server) = sockets();
    let frame = send(&mut peer, &mut server, false);
    let mut capture = Capture {
        revoke_after: Operation::PayloadDecrypt,
        permitted: true,
        received: vec![],
    };
    // Act
    let result = read_frame(&mut client, &mut noise, &mut capture);
    send(&mut peer, &mut server, false);
    let next = maybe_read_frame(&mut client, &mut noise, &mut capture);
    // Assert: actual plaintext is retained, but never returned for normal work.
    assert_eq!(result, Err(Failure::Cancelled));
    assert_eq!(next, Err(Failure::Cancelled));
    assert_eq!(capture.received, vec![frame]);
    assert_unread(&client);
}
#[test]
fn revoked_partial_frame_never_reads_payload_or_claims_observation() {
    // Arrange
    let (mut noise, mut peer) = pair();
    let (mut client, mut server) = sockets();
    send(&mut peer, &mut server, false);
    let mut capture = Capture {
        revoke_after: Operation::HeaderDecrypt,
        permitted: true,
        received: vec![],
    };
    // Act
    let result = read_frame(&mut client, &mut noise, &mut capture);
    // Assert
    assert_eq!(result, Err(Failure::Cancelled));
    assert!(capture.received.is_empty());
    assert_unread(&client);
}
#[test]
fn unauthenticated_payload_never_reaches_observation_hook() {
    // Arrange
    let (mut noise, mut peer) = pair();
    let (mut client, mut server) = sockets();
    send(&mut peer, &mut server, true);
    let mut capture = Capture {
        revoke_after: Operation::PayloadDecrypt,
        permitted: true,
        received: vec![],
    };
    // Act
    let result = read_frame(&mut client, &mut noise, &mut capture);
    // Assert
    assert_eq!(result, Err(Failure::Io));
    assert!(capture.received.is_empty());
}

struct TestRng(u64);
impl RngCore for TestRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn fill_bytes(&mut self, destination: &mut [u8]) {
        for chunk in destination.chunks_mut(8) {
            chunk.copy_from_slice(&self.next_u64().to_le_bytes()[..chunk.len()]);
        }
    }
    fn try_fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(destination);
        Ok(())
    }
}
impl CryptoRng for TestRng {}

fn assert_unread(client: &TcpStream) {
    // The production call above is nonblocking. The test observes eventual TCP
    // delivery separately; peer write completion need not mean immediate arrival.
    client
        .set_nonblocking(false)
        .expect("test observation blocks");
    client
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("bounded observation");
    assert!(client.peek(&mut [0; 1]).expect("unread peer bytes arrive") > 0);
}
