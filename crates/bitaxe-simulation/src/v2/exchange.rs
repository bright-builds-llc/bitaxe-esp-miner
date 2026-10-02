use crate::SimulationError;
use bitaxe_stratum::v2::{
    frame::{Frame, FrameHeader, FRAME_HEADER_LEN},
    noise::{NoiseInitiator, NoiseTransport, ENCRYPTED_HEADER_LEN},
};
use bitaxe_virtual_board::{scheduler::DeterministicScheduler, transport::VirtualTransport};
use noise_sv2::{NoiseCodec, Responder};
use rand::{CryptoRng, RngCore};
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

pub struct Exchange {
    pub client: NoiseTransport,
    server: NoiseCodec,
    outbound: VirtualTransport,
    inbound: VirtualTransport,
    pub delivered_frames: u32,
}
impl Exchange {
    pub fn new(seed: u64, wrong_authority: bool) -> Result<Self, SimulationError> {
        let mut client_rng = SyntheticRng::new(seed);
        let mut responder_rng = SyntheticRng::new(seed ^ 0xaaccee);
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
        let mut responder = Responder::from_authority_kp_with_rng(
            &PUBLIC,
            &PRIVATE,
            Duration::from_secs(3600),
            &mut responder_rng,
        )
        .map_err(|_| SimulationError::Boundary("noise_responder"))?;
        let (act_two, server) = responder
            .step_1_with_now_rng(act_one, 100, &mut responder_rng)
            .map_err(|_| SimulationError::Boundary("noise_act_two"))?;
        let client = client
            .complete(&act_two, 100)
            .map_err(|_| SimulationError::Boundary("noise_authentication"))?;
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
    pub fn encrypt_client(&mut self, frame: &Frame) -> Result<Vec<u8>, SimulationError> {
        self.client
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
        self.server
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
            self.server
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
        self.server
            .encrypt(&mut encoded)
            .map_err(|_| SimulationError::Boundary("server_encrypt"))?;
        if !frame.payload().is_empty() {
            let mut payload = frame.payload().to_vec();
            self.server
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
            .client
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
