//! Narrow adapter over the official Stratum Reference Implementation Noise crate.

use std::fmt;

use noise_sv2::{Initiator, NoiseCodec, INITIATOR_EXPECTED_HANDSHAKE_MESSAGE_SIZE};
use rand::{CryptoRng, Rng};

use super::frame::{Frame, FrameHeader, FRAME_HEADER_LEN};
use super::StratumV2Error;

pub const ACT_ONE_LEN: usize = 64;
pub const ACT_TWO_LEN: usize = INITIATOR_EXPECTED_HANDSHAKE_MESSAGE_SIZE;
pub const ENCRYPTED_HEADER_LEN: usize = FRAME_HEADER_LEN + noise_sv2::AEAD_MAC_LEN;

pub struct NoiseInitiator {
    inner: Option<Box<Initiator>>,
    act_one_sent: bool,
}

impl fmt::Debug for NoiseInitiator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NoiseInitiator")
            .field(
                "state",
                &if self.act_one_sent {
                    "awaiting_response"
                } else {
                    "ready"
                },
            )
            .finish()
    }
}

impl NoiseInitiator {
    pub fn new<R: Rng + CryptoRng + ?Sized>(
        maybe_authority_public_key: Option<[u8; 32]>,
        rng: &mut R,
    ) -> Result<Self, StratumV2Error> {
        let inner = match maybe_authority_public_key {
            Some(key) => Initiator::from_raw_k_with_rng(key, rng),
            None => Initiator::without_pk_with_rng(rng),
        }
        .map_err(|_| StratumV2Error::NoiseHandshake)?;
        Ok(Self {
            inner: Some(inner),
            act_one_sent: false,
        })
    }

    pub fn act_one(&mut self) -> Result<[u8; ACT_ONE_LEN], StratumV2Error> {
        if self.act_one_sent {
            return Err(StratumV2Error::InvalidNoiseState);
        }
        let inner = self
            .inner
            .as_mut()
            .ok_or(StratumV2Error::InvalidNoiseState)?;
        let act_one = inner.step_0().map_err(|_| StratumV2Error::NoiseHandshake)?;
        self.act_one_sent = true;
        Ok(act_one)
    }

    pub fn complete(
        mut self,
        act_two: &[u8],
        unix_time_seconds: u32,
    ) -> Result<NoiseTransport, StratumV2Error> {
        if !self.act_one_sent || act_two.len() != ACT_TWO_LEN {
            return Err(StratumV2Error::InvalidNoiseState);
        }
        let response: [u8; ACT_TWO_LEN] = act_two
            .try_into()
            .map_err(|_| StratumV2Error::InvalidNoiseState)?;
        let mut inner = self.inner.take().ok_or(StratumV2Error::InvalidNoiseState)?;
        let codec = inner
            .step_2_with_now(response, unix_time_seconds)
            .map_err(|_| StratumV2Error::NoiseAuthentication)?;
        Ok(NoiseTransport {
            codec,
            send_budget: NonceBudget::new(),
            receive_budget: NonceBudget::new(),
            maybe_pending_header: None,
            poisoned: false,
        })
    }
}

pub struct NoiseTransport {
    codec: NoiseCodec,
    send_budget: NonceBudget,
    receive_budget: NonceBudget,
    maybe_pending_header: Option<FrameHeader>,
    poisoned: bool,
}

impl fmt::Debug for NoiseTransport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NoiseTransport")
            .field(
                "state",
                &if self.poisoned {
                    "poisoned"
                } else {
                    "established"
                },
            )
            .finish()
    }
}

impl NoiseTransport {
    pub fn encrypt_frame(&mut self, frame: &Frame) -> Result<Vec<u8>, StratumV2Error> {
        self.require_usable()?;
        let operation_count = 1 + usize::from(!frame.payload().is_empty());
        self.send_budget.reserve(operation_count)?;

        let mut encrypted_header = frame.header.encode().to_vec();
        if self.codec.encrypt(&mut encrypted_header).is_err() {
            self.poisoned = true;
            return Err(StratumV2Error::NoiseAuthentication);
        }
        let mut output = encrypted_header;
        if !frame.payload().is_empty() {
            let mut encrypted_payload = frame.payload().to_vec();
            if self.codec.encrypt(&mut encrypted_payload).is_err() {
                self.poisoned = true;
                return Err(StratumV2Error::NoiseAuthentication);
            }
            output.extend_from_slice(&encrypted_payload);
        }
        Ok(output)
    }

    pub fn decrypt_frame(
        &mut self,
        encrypted_header: &[u8],
        encrypted_payload: &[u8],
    ) -> Result<Frame, StratumV2Error> {
        let pending = self.decrypt_header(encrypted_header)?;
        self.decrypt_payload(pending, encrypted_payload)
    }

    pub fn decrypt_header(
        &mut self,
        encrypted_header: &[u8],
    ) -> Result<DecryptedNoiseHeader, StratumV2Error> {
        self.require_usable()?;
        if self.maybe_pending_header.is_some() {
            return self.poison(StratumV2Error::InvalidNoiseState);
        }
        if encrypted_header.len() != ENCRYPTED_HEADER_LEN {
            return self.poison(StratumV2Error::FrameLengthMismatch {
                expected: ENCRYPTED_HEADER_LEN,
                actual: encrypted_header.len(),
            });
        }

        self.receive_budget.reserve(1)?;
        let mut header_bytes = encrypted_header.to_vec();
        if self.codec.decrypt(&mut header_bytes).is_err() {
            return self.poison(StratumV2Error::NoiseAuthentication);
        }
        let header = match FrameHeader::parse(&header_bytes) {
            Ok(header) => header,
            Err(error) => return self.poison(error),
        };
        let encrypted_payload_len = if header.payload_len == 0 {
            0
        } else {
            header.payload_len + noise_sv2::AEAD_MAC_LEN
        };
        self.maybe_pending_header = Some(header);
        Ok(DecryptedNoiseHeader {
            encrypted_payload_len,
        })
    }

    pub fn decrypt_payload(
        &mut self,
        pending: DecryptedNoiseHeader,
        encrypted_payload: &[u8],
    ) -> Result<Frame, StratumV2Error> {
        self.require_usable()?;
        let header = self
            .maybe_pending_header
            .take()
            .ok_or(StratumV2Error::InvalidNoiseState)?;
        if encrypted_payload.len() != pending.encrypted_payload_len {
            return self.poison(StratumV2Error::FrameLengthMismatch {
                expected: pending.encrypted_payload_len,
                actual: encrypted_payload.len(),
            });
        }

        let payload = if encrypted_payload.is_empty() {
            Vec::new()
        } else {
            self.receive_budget.reserve(1)?;
            let mut payload = encrypted_payload.to_vec();
            if self.codec.decrypt(&mut payload).is_err() {
                return self.poison(StratumV2Error::NoiseAuthentication);
            }
            payload
        };
        Frame::new(header.extension_type, header.message_type, payload)
    }

    fn require_usable(&self) -> Result<(), StratumV2Error> {
        if self.poisoned {
            Err(StratumV2Error::InvalidNoiseState)
        } else {
            Ok(())
        }
    }

    fn poison<T>(&mut self, error: StratumV2Error) -> Result<T, StratumV2Error> {
        self.poisoned = true;
        Err(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecryptedNoiseHeader {
    encrypted_payload_len: usize,
}

impl DecryptedNoiseHeader {
    #[must_use]
    pub const fn encrypted_payload_len(self) -> usize {
        self.encrypted_payload_len
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NonceBudget {
    used: u64,
}

impl NonceBudget {
    const fn new() -> Self {
        Self { used: 0 }
    }

    fn reserve(&mut self, count: usize) -> Result<(), StratumV2Error> {
        let count = u64::try_from(count).map_err(|_| StratumV2Error::NoiseNonceExhausted)?;
        self.used = self
            .used
            .checked_add(count)
            .ok_or(StratumV2Error::NoiseNonceExhausted)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use noise_sv2::Responder;
    use rand::{CryptoRng, RngCore};

    use super::*;

    const AUTHORITY_PRIVATE_KEY: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ];
    const AUTHORITY_PUBLIC_KEY: [u8; 32] = [
        0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b,
        0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8,
        0x17, 0x98,
    ];

    #[test]
    fn official_noise_handshake_round_trips_split_sv2_frame() {
        // Arrange
        let (mut initiator_transport, mut responder_codec) = established_pair();
        let frame = Frame::new(0x8000, 0x1a, vec![1, 2, 3, 4]).expect("frame");

        // Act
        let encrypted = initiator_transport
            .encrypt_frame(&frame)
            .expect("encrypt frame");
        let mut header = encrypted[..ENCRYPTED_HEADER_LEN].to_vec();
        responder_codec
            .decrypt(&mut header)
            .expect("decrypt header");
        let mut payload = encrypted[ENCRYPTED_HEADER_LEN..].to_vec();
        responder_codec
            .decrypt(&mut payload)
            .expect("decrypt payload");

        // Assert
        assert_eq!(FrameHeader::parse(&header), Ok(frame.header));
        assert_eq!(payload, frame.payload());
    }

    #[test]
    fn official_noise_transport_rejects_tamper_and_stays_poisoned() {
        // Arrange
        let (mut initiator_transport, mut responder_codec) = established_pair();
        let frame = Frame::new(0, 1, vec![9]).expect("frame");
        let mut encrypted_header = frame.header.encode().to_vec();
        responder_codec
            .encrypt(&mut encrypted_header)
            .expect("responder encrypt header");
        encrypted_header[0] ^= 1;
        let mut encrypted_payload = frame.payload().to_vec();
        responder_codec
            .encrypt(&mut encrypted_payload)
            .expect("responder encrypt payload");

        // Act
        let tampered = initiator_transport.decrypt_frame(&encrypted_header, &encrypted_payload);
        let repeated = initiator_transport.decrypt_frame(&encrypted_header, &encrypted_payload);

        // Assert
        assert_eq!(tampered, Err(StratumV2Error::NoiseAuthentication));
        assert_eq!(repeated, Err(StratumV2Error::InvalidNoiseState));
    }

    #[test]
    fn nonce_budget_fails_before_the_official_cipher_counter_can_wrap() {
        // Arrange
        let mut budget = NonceBudget { used: u64::MAX - 1 };

        // Act
        let final_nonce = budget.reserve(1);
        let exhausted = budget.reserve(1);

        // Assert
        assert_eq!(final_nonce, Ok(()));
        assert_eq!(exhausted, Err(StratumV2Error::NoiseNonceExhausted));
    }

    fn established_pair() -> (NoiseTransport, NoiseCodec) {
        let mut initiator_rng = DeterministicRng::new(1);
        let mut responder_rng = DeterministicRng::new(2);
        let mut initiator =
            NoiseInitiator::new(Some(AUTHORITY_PUBLIC_KEY), &mut initiator_rng).expect("initiator");
        let act_one = initiator.act_one().expect("act one");
        let mut responder = Responder::from_authority_kp_with_rng(
            &AUTHORITY_PUBLIC_KEY,
            &AUTHORITY_PRIVATE_KEY,
            Duration::from_secs(60),
            &mut responder_rng,
        )
        .expect("responder");
        let (act_two, responder_codec) = responder
            .step_1_with_now_rng(act_one, 100, &mut responder_rng)
            .expect("act two");
        let initiator_transport = initiator.complete(&act_two, 100).expect("complete");
        (initiator_transport, responder_codec)
    }

    struct DeterministicRng(u64);

    impl DeterministicRng {
        const fn new(seed: u64) -> Self {
            Self(seed)
        }
    }

    impl RngCore for DeterministicRng {
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
                let bytes = self.next_u64().to_le_bytes();
                chunk.copy_from_slice(&bytes[..chunk.len()]);
            }
        }

        fn try_fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(destination);
            Ok(())
        }
    }

    impl CryptoRng for DeterministicRng {}
}
