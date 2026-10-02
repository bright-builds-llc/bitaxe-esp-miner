use crate::SimulationError;
use bitaxe_stratum::v2::{
    frame::{Frame, FrameHeader, FRAME_HEADER_LEN},
    noise::{NoiseInitiator, NoiseTransport, ACT_ONE_LEN, ACT_TWO_LEN, ENCRYPTED_HEADER_LEN},
};
use bitaxe_virtual_board::{scheduler::DeterministicScheduler, transport::VirtualTransport};
use noise_sv2::{NoiseCodec, Responder};
use rand::{CryptoRng, RngCore};
use std::sync::OnceLock;
use std::time::Duration;

pub(crate) const PRIVATE: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];
pub(crate) const PUBLIC: [u8; 32] = [
    0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
    0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
];
pub(crate) const OTHER_PUBLIC: [u8; 32] = [
    0xc6, 0x04, 0x7f, 0x94, 0x41, 0xed, 0x7d, 0x6d, 0x30, 0x45, 0x40, 0x6e, 0x95, 0xc0, 0x7c, 0xd8,
    0x5c, 0x77, 0x8e, 0x4b, 0x8c, 0xef, 0x3c, 0xa7, 0xab, 0xac, 0x09, 0xb9, 0x5c, 0x70, 0x9e, 0xe5,
];

/// Fixed public synthetic responder trust, never a physical deployment key.
pub fn synthetic_authority_public_key() -> [u8; 32] {
    PUBLIC
}

/// Synthetic seeded randomness is never used by a physical signer or device.
pub(crate) struct SyntheticRng(DeterministicScheduler<()>);
impl SyntheticRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self(DeterministicScheduler::new(seed))
    }
}
impl RngCore for SyntheticRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0.next_random()
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
impl CryptoRng for SyntheticRng {}

/// Owners live in reserved heap slots so the frame holding an `Exchange` stays small
/// while certificate verification runs.
pub struct Exchange {
    client: Vec<NoiseTransport>,
    server: Vec<NoiseCodec>,
    outbound: VirtualTransport,
    inbound: VirtualTransport,
    pub delivered_frames: u32,
}
impl Exchange {
    #[inline(never)]
    pub fn new(seed: u64, wrong_authority: bool) -> Result<Self, SimulationError> {
        let (client, server) = run_on_handshake_stack(seed, wrong_authority)?;
        let mut outbound = VirtualTransport::new(65536);
        outbound.connect()?;
        let mut inbound = VirtualTransport::new(65536);
        inbound.connect()?;
        Ok(Self {
            client,
            server,
            outbound,
            inbound,
            delivered_frames: 0,
        })
    }
    fn client(&mut self) -> Result<&mut NoiseTransport, SimulationError> {
        self.client
            .first_mut()
            .ok_or(SimulationError::Boundary("noise_client_missing"))
    }
    fn server(&mut self) -> Result<&mut NoiseCodec, SimulationError> {
        self.server
            .first_mut()
            .ok_or(SimulationError::Boundary("noise_server_missing"))
    }
    pub fn encrypt_client(&mut self, frame: &Frame) -> Result<Vec<u8>, SimulationError> {
        self.client()?
            .encrypt_frame(frame)
            .map_err(|_| SimulationError::Boundary("client_encrypt"))
    }
    pub fn deliver_client(&mut self, encrypted: &[u8]) -> Result<Frame, SimulationError> {
        self.deliver_client_with_write_observer(encrypted, || {})
    }
    pub fn deliver_client_with_write_observer(
        &mut self,
        encrypted: &[u8],
        mut written: impl FnMut(),
    ) -> Result<Frame, SimulationError> {
        self.outbound.send(0, encrypted)?;
        let bytes = self
            .outbound
            .receive(0)?
            .ok_or(SimulationError::Boundary("client_write_not_completed"))?;
        written();
        if bytes.len() < ENCRYPTED_HEADER_LEN {
            return Err(SimulationError::Boundary("server_header_length"));
        }
        let mut header = bytes[..ENCRYPTED_HEADER_LEN].to_vec();
        self.server()?
            .decrypt(&mut header)
            .map_err(|_| SimulationError::Boundary("server_header_authentication"))?;
        if header.len() != FRAME_HEADER_LEN {
            return Err(SimulationError::Boundary("server_header_length"));
        }
        let parsed = FrameHeader::parse(&header)
            .map_err(|_| SimulationError::Boundary("server_header_parse"))?;
        let expected = parsed.payload_len
            + if parsed.payload_len == 0 {
                0
            } else {
                noise_sv2::AEAD_MAC_LEN
            };
        if bytes.len() - ENCRYPTED_HEADER_LEN != expected {
            return Err(SimulationError::Boundary("server_payload_length"));
        }
        let mut payload = bytes[ENCRYPTED_HEADER_LEN..].to_vec();
        if !payload.is_empty() {
            self.server()?
                .decrypt(&mut payload)
                .map_err(|_| SimulationError::Boundary("server_payload_authentication"))?;
        }
        self.delivered_frames += 1;
        Frame::new(parsed.extension_type, parsed.message_type, payload)
            .map_err(|_| SimulationError::Boundary("server_frame"))
    }
    pub fn client_to_server(&mut self, frame: &Frame) -> Result<Frame, SimulationError> {
        let encrypted = self.encrypt_client(frame)?;
        self.deliver_client(&encrypted)
    }
    pub fn server_to_client(
        &mut self,
        frame: &Frame,
        tamper: bool,
    ) -> Result<Frame, SimulationError> {
        let mut encoded = frame.header.encode().to_vec();
        self.server()?
            .encrypt(&mut encoded)
            .map_err(|_| SimulationError::Boundary("server_encrypt"))?;
        if !frame.payload().is_empty() {
            let mut payload = frame.payload().to_vec();
            self.server()?
                .encrypt(&mut payload)
                .map_err(|_| SimulationError::Boundary("server_encrypt"))?;
            encoded.extend(payload);
        }
        if tamper {
            let last = encoded
                .last_mut()
                .ok_or(SimulationError::Boundary("empty_ciphertext"))?;
            *last ^= 1;
        }
        self.inbound.send(0, &encoded)?;
        let bytes = self
            .inbound
            .receive(0)?
            .ok_or(SimulationError::Boundary("server_write_not_completed"))?;
        if bytes.len() < ENCRYPTED_HEADER_LEN {
            return Err(SimulationError::Boundary("client_header_length"));
        }
        let frame = self
            .client()?
            .decrypt_frame(
                &bytes[..ENCRYPTED_HEADER_LEN],
                &bytes[ENCRYPTED_HEADER_LEN..],
            )
            .map_err(|_| SimulationError::Boundary("client_authentication"))?;
        self.delivered_frames += 1;
        Ok(frame)
    }
    pub fn inject_close_rejection(&mut self) {
        self.outbound.reject_close = true;
    }
    pub fn close(&mut self) -> Result<(), SimulationError> {
        let first = self.outbound.close();
        let second = self.inbound.close();
        first?;
        second?;
        Ok(())
    }
    pub fn released(&self) -> bool {
        !self.outbound.connected
            && !self.inbound.connected
            && self.outbound.queued_bytes() == 0
            && self.inbound.queued_bytes() == 0
    }
}

/// Certificate verification alone needs about 10.6 KB below the handshake, so the
/// handshake runs on its own stack instead of the control caller's, as on the device.
pub const HANDSHAKE_STACK_BYTES: usize = 16 * 1024;
// The deepest measured target descent from `handshake` is responder ECDH at
// 12,192 bytes, plus 224 bytes of thread entry; the 2,048-byte margin must fit.
const _: () = assert!(HANDSHAKE_STACK_BYTES >= 12_192 + 224 + 2_048);

// The contract above is measured on optimized Xtensa frames. Unoptimized host
// frames are far larger, so host builds use at least the standard std stack.
const HELPER_STACK_BYTES: usize = if cfg!(target_os = "espidf") {
    HANDSHAKE_STACK_BYTES
} else {
    2 * 1024 * 1024
};

static HANDSHAKE_STACK_OBSERVER: OnceLock<fn()> = OnceLock::new();

/// Registers a target adapter that samples the helper stack just before the helper
/// exits. It runs on the helper thread and must not allocate. Only the first call wins.
pub fn set_handshake_stack_observer(observer: fn()) -> bool {
    HANDSHAKE_STACK_OBSERVER.set(observer).is_ok()
}

#[inline(never)]
fn run_on_handshake_stack(
    seed: u64,
    wrong_authority: bool,
) -> Result<(Vec<NoiseTransport>, Vec<NoiseCodec>), SimulationError> {
    let mut client = reserve_owner()?;
    let mut server = reserve_owner()?;
    // Joined before return, so ordering and outcomes stay deterministic.
    let helper = std::thread::Builder::new()
        .name("noise-handshake".into())
        .stack_size(HELPER_STACK_BYTES)
        .spawn(move || {
            let outcome = handshake(seed, wrong_authority, &mut client, &mut server);
            if let Some(observer) = HANDSHAKE_STACK_OBSERVER.get() {
                observer();
            }
            outcome.map(|()| (client, server)).map_err(boundary_label)
        })
        .map_err(|_| SimulationError::Boundary("noise_handshake_spawn"))?;
    helper
        .join()
        .map_err(|_| SimulationError::Boundary("noise_handshake_panicked"))?
        .map_err(SimulationError::Boundary)
}

fn boundary_label(error: SimulationError) -> &'static str {
    match error {
        SimulationError::Boundary(label) => label,
        _ => "noise_handshake_failed",
    }
}

fn reserve_owner<T>() -> Result<Vec<T>, SimulationError> {
    let mut slot = Vec::new();
    slot.try_reserve_exact(1)
        .map_err(|_| SimulationError::Boundary("noise_owner_reservation"))?;
    Ok(slot)
}

// Runs on the handshake stack; construction, responder and completion use sibling frames.
#[inline(never)]
fn handshake(
    seed: u64,
    wrong_authority: bool,
    client_slot: &mut Vec<NoiseTransport>,
    server_slot: &mut Vec<NoiseCodec>,
) -> Result<(), SimulationError> {
    let (client, act_one) = prepare_initiator(seed, wrong_authority)?;
    let mut act_two = [0_u8; ACT_TWO_LEN];
    let mut responder_rng = SyntheticRng::new(seed ^ 0xaaccee);
    let mut responder = construct_responder(&mut responder_rng)
        .map_err(|()| SimulationError::Boundary("noise_responder"))?;
    step_responder(
        &mut responder,
        act_one,
        &mut responder_rng,
        &mut act_two,
        server_slot,
    )
    .map_err(|()| SimulationError::Boundary("noise_act_two"))?;
    drop(responder);
    client
        .complete_diagnostic_into(&act_two, 100, client_slot)
        .map_err(|_| SimulationError::Boundary("noise_authentication"))
}

#[inline(never)]
fn prepare_initiator(
    seed: u64,
    wrong_authority: bool,
) -> Result<(NoiseInitiator, [u8; ACT_ONE_LEN]), SimulationError> {
    let mut client_rng = SyntheticRng::new(seed);
    let mut client = NoiseInitiator::new(
        Some(if wrong_authority {
            OTHER_PUBLIC
        } else {
            PUBLIC
        }),
        &mut client_rng,
    )
    .map_err(|_| SimulationError::Boundary("noise_initiator"))?;
    let act_one = client
        .act_one()
        .map_err(|_| SimulationError::Boundary("noise_act_one"))?;
    Ok((client, act_one))
}

// Construction scratch stays out of the sibling frame that hosts ECDH and signing.
#[inline(never)]
pub(crate) fn construct_responder(rng: &mut SyntheticRng) -> Result<Box<Responder>, ()> {
    Responder::from_authority_kp_with_rng(&PUBLIC, &PRIVATE, Duration::from_secs(3600), rng)
        .map_err(|_| ())
}

#[inline(never)]
pub(crate) fn step_responder(
    responder: &mut Responder,
    act_one: [u8; ACT_ONE_LEN],
    rng: &mut SyntheticRng,
    act_two: &mut [u8; ACT_TWO_LEN],
    server_slot: &mut Vec<NoiseCodec>,
) -> Result<(), ()> {
    if !server_slot.is_empty() || server_slot.capacity() == 0 {
        return Err(());
    }
    // Consuming the return slot in place avoids a second stack copy of the codec.
    match responder.step_1_with_now_rng(act_one, 100, rng) {
        Ok((response, server)) => {
            *act_two = response;
            // Capacity was reserved fallibly before opaque crypto; this push cannot allocate.
            server_slot.push(server);
            Ok(())
        }
        Err(_) => Err(()),
    }
}
