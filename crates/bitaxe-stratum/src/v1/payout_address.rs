//! Pure Bitcoin payout-address codecs and standard output-script validation.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/base58.c`
//! - `reference/esp-miner/components/stratum/segwit_addr.c`
//! - `reference/esp-miner/components/stratum/coinbase_decoder.c`
//! - Parity checklist row `STR-012`

use sha2::{Digest, Sha256};
use thiserror::Error;

const BASE58_ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const BECH32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const BECH32_CHECKSUM: u32 = 1;
const BECH32M_CHECKSUM: u32 = 0x2bc8_30a3;
const MAX_ADDRESS_LEN: usize = 90;

/// Bitcoin network parameters used by payout-address rendering and validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitcoinNetwork {
    Mainnet,
    Testnet,
    Regtest,
}

impl BitcoinNetwork {
    fn p2pkh_version(self) -> u8 {
        match self {
            Self::Mainnet => 0x00,
            Self::Testnet | Self::Regtest => 0x6f,
        }
    }

    fn p2sh_version(self) -> u8 {
        match self {
            Self::Mainnet => 0x05,
            Self::Testnet | Self::Regtest => 0xc4,
        }
    }

    fn human_readable_part(self) -> &'static str {
        match self {
            Self::Mainnet => "bc",
            Self::Testnet => "tb",
            Self::Regtest => "bcrt",
        }
    }
}

/// Address/script shape recognized by the payout codec.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayoutAddressKind {
    P2pkh,
    P2sh,
    P2wpkh,
    P2wsh,
    P2tr,
    Witness(u8),
}

/// Canonically decoded payout address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedPayoutAddress {
    /// Network parameters used to validate the address.
    pub network: BitcoinNetwork,
    /// Standard or future witness address kind.
    pub kind: PayoutAddressKind,
    /// Hash or witness program committed by the address.
    pub payload: Vec<u8>,
}

/// Closed payout-address failure categories.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PayoutAddressError {
    #[error("invalid Base58 encoding")]
    InvalidBase58,
    #[error("invalid Base58Check checksum")]
    InvalidBase58Checksum,
    #[error("invalid Bech32 encoding")]
    InvalidBech32,
    #[error("invalid SegWit witness program")]
    InvalidWitnessProgram,
    #[error("payout address belongs to another Bitcoin network")]
    NetworkMismatch,
    #[error("unsupported payout output script")]
    UnsupportedScript,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Bech32Encoding {
    Bech32,
    Bech32m,
}

/// Encodes a version byte and payload using canonical Base58Check.
#[must_use]
pub fn encode_base58_check(version: u8, payload: &[u8]) -> String {
    let mut checked = Vec::with_capacity(payload.len() + 5);
    checked.push(version);
    checked.extend_from_slice(payload);
    checked.extend_from_slice(&double_sha256(&checked)[..4]);
    encode_base58(&checked)
}

/// Decodes and verifies one canonical Base58Check value.
pub fn decode_base58_check(value: &str) -> Result<(u8, Vec<u8>), PayoutAddressError> {
    let decoded = decode_base58_check_bytes(value)?;
    Ok((decoded[0], decoded[1..].to_vec()))
}

/// Decodes and verifies a canonical Base58Check value while retaining all
/// version bytes for protocols that use a multi-byte prefix.
pub fn decode_base58_check_bytes(value: &str) -> Result<Vec<u8>, PayoutAddressError> {
    let decoded = decode_base58(value)?;
    if decoded.len() < 5 {
        return Err(PayoutAddressError::InvalidBase58);
    }
    let payload_end = decoded.len() - 4;
    let expected = double_sha256(&decoded[..payload_end]);
    if decoded[payload_end..] != expected[..4] {
        return Err(PayoutAddressError::InvalidBase58Checksum);
    }
    Ok(decoded[..payload_end].to_vec())
}

/// Encodes a valid SegWit witness program for the selected Bitcoin network.
pub fn encode_segwit_address(
    network: BitcoinNetwork,
    witness_version: u8,
    program: &[u8],
) -> Result<String, PayoutAddressError> {
    validate_witness_program(witness_version, program)?;
    let encoding = witness_encoding(witness_version);
    let mut data = Vec::with_capacity(1 + (program.len() * 8).div_ceil(5));
    data.push(witness_version);
    data.extend(convert_bits(program, 8, 5, true)?);
    encode_bech32(network.human_readable_part(), &data, encoding)
}

/// Decodes and validates one payout address for an expected Bitcoin network.
pub fn decode_payout_address(
    network: BitcoinNetwork,
    value: &str,
) -> Result<DecodedPayoutAddress, PayoutAddressError> {
    if looks_like_segwit(value) {
        return decode_segwit_address(network, value);
    }
    let (version, payload) = decode_base58_check(value)?;
    if payload.len() != 20 {
        return Err(PayoutAddressError::InvalidBase58);
    }
    let kind = if version == network.p2pkh_version() {
        PayoutAddressKind::P2pkh
    } else if version == network.p2sh_version() {
        PayoutAddressKind::P2sh
    } else {
        return Err(PayoutAddressError::NetworkMismatch);
    };
    Ok(DecodedPayoutAddress {
        network,
        kind,
        payload,
    })
}

/// Renders one supported standard output script as a network-specific address.
pub fn render_standard_script_address(
    network: BitcoinNetwork,
    script: &[u8],
) -> Result<String, PayoutAddressError> {
    let (kind, payload) = classify_standard_script(script)?;
    match kind {
        PayoutAddressKind::P2pkh => Ok(encode_base58_check(network.p2pkh_version(), payload)),
        PayoutAddressKind::P2sh => Ok(encode_base58_check(network.p2sh_version(), payload)),
        PayoutAddressKind::P2wpkh | PayoutAddressKind::P2wsh => {
            encode_segwit_address(network, 0, payload)
        }
        PayoutAddressKind::P2tr => encode_segwit_address(network, 1, payload),
        PayoutAddressKind::Witness(version) => encode_segwit_address(network, version, payload),
    }
}

/// Returns whether a canonical payout address commits to the exact standard script.
pub fn payout_address_matches_script(
    network: BitcoinNetwork,
    address: &str,
    script: &[u8],
) -> Result<bool, PayoutAddressError> {
    let decoded = decode_payout_address(network, address)?;
    let (script_kind, script_payload) = classify_standard_script(script)?;
    Ok(decoded.kind == script_kind && decoded.payload == script_payload)
}

fn classify_standard_script(
    script: &[u8],
) -> Result<(PayoutAddressKind, &[u8]), PayoutAddressError> {
    match script {
        [0x76, 0xa9, 0x14, payload @ .., 0x88, 0xac] if payload.len() == 20 => {
            Ok((PayoutAddressKind::P2pkh, payload))
        }
        [0xa9, 0x14, payload @ .., 0x87] if payload.len() == 20 => {
            Ok((PayoutAddressKind::P2sh, payload))
        }
        [0x00, 0x14, payload @ ..] if payload.len() == 20 => {
            Ok((PayoutAddressKind::P2wpkh, payload))
        }
        [0x00, 0x20, payload @ ..] if payload.len() == 32 => {
            Ok((PayoutAddressKind::P2wsh, payload))
        }
        [0x51, 0x20, payload @ ..] if payload.len() == 32 => Ok((PayoutAddressKind::P2tr, payload)),
        _ => Err(PayoutAddressError::UnsupportedScript),
    }
}

fn encode_base58(bytes: &[u8]) -> String {
    let zero_count = bytes.iter().take_while(|byte| **byte == 0).count();
    if zero_count == bytes.len() {
        return std::iter::repeat_n('1', zero_count).collect();
    }
    let mut digits = vec![0_u8];
    for byte in &bytes[zero_count..] {
        let mut carry = u32::from(*byte);
        for digit in &mut digits {
            carry += u32::from(*digit) * 256;
            *digit = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let mut output = String::with_capacity(zero_count + digits.len());
    output.extend(std::iter::repeat_n('1', zero_count));
    for digit in digits.iter().rev() {
        output.push(char::from(BASE58_ALPHABET[usize::from(*digit)]));
    }
    output
}

fn decode_base58(value: &str) -> Result<Vec<u8>, PayoutAddressError> {
    if value.is_empty() || value.len() > MAX_ADDRESS_LEN || !value.is_ascii() {
        return Err(PayoutAddressError::InvalidBase58);
    }
    let zero_count = value.bytes().take_while(|byte| *byte == b'1').count();
    let mut bytes = vec![0_u8];
    for encoded in value.bytes().skip(zero_count) {
        let digit = BASE58_ALPHABET
            .iter()
            .position(|candidate| *candidate == encoded)
            .ok_or(PayoutAddressError::InvalidBase58)?;
        let mut carry = digit as u32;
        for byte in &mut bytes {
            carry += u32::from(*byte) * 58;
            *byte = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    while bytes.len() > 1 && bytes.last() == Some(&0) {
        bytes.pop();
    }
    let mut decoded = vec![0; zero_count];
    if value.len() > zero_count {
        decoded.extend(bytes.iter().rev());
    }
    if encode_base58(&decoded) != value {
        return Err(PayoutAddressError::InvalidBase58);
    }
    Ok(decoded)
}

fn decode_segwit_address(
    network: BitcoinNetwork,
    value: &str,
) -> Result<DecodedPayoutAddress, PayoutAddressError> {
    let (human_readable_part, data, encoding) = decode_bech32(value)?;
    if human_readable_part != network.human_readable_part() {
        return Err(PayoutAddressError::NetworkMismatch);
    }
    let Some((&witness_version, encoded_program)) = data.split_first() else {
        return Err(PayoutAddressError::InvalidWitnessProgram);
    };
    if witness_version > 16 || witness_encoding(witness_version) != encoding {
        return Err(PayoutAddressError::InvalidWitnessProgram);
    }
    let program = convert_bits(encoded_program, 5, 8, false)?;
    validate_witness_program(witness_version, &program)?;
    let kind = match (witness_version, program.len()) {
        (0, 20) => PayoutAddressKind::P2wpkh,
        (0, 32) => PayoutAddressKind::P2wsh,
        (1, 32) => PayoutAddressKind::P2tr,
        (version, _) => PayoutAddressKind::Witness(version),
    };
    Ok(DecodedPayoutAddress {
        network,
        kind,
        payload: program,
    })
}

fn validate_witness_program(version: u8, program: &[u8]) -> Result<(), PayoutAddressError> {
    if version > 16 || !(2..=40).contains(&program.len()) {
        return Err(PayoutAddressError::InvalidWitnessProgram);
    }
    if version == 0 && program.len() != 20 && program.len() != 32 {
        return Err(PayoutAddressError::InvalidWitnessProgram);
    }
    Ok(())
}

fn witness_encoding(version: u8) -> Bech32Encoding {
    if version == 0 {
        Bech32Encoding::Bech32
    } else {
        Bech32Encoding::Bech32m
    }
}

fn looks_like_segwit(value: &str) -> bool {
    let lowercase = value.to_ascii_lowercase();
    lowercase.starts_with("bc1") || lowercase.starts_with("tb1") || lowercase.starts_with("bcrt1")
}

fn encode_bech32(
    human_readable_part: &str,
    data: &[u8],
    encoding: Bech32Encoding,
) -> Result<String, PayoutAddressError> {
    if human_readable_part.is_empty()
        || !human_readable_part
            .bytes()
            .all(|byte| (33..=126).contains(&byte) && !byte.is_ascii_uppercase())
        || data.iter().any(|value| *value >= 32)
        || human_readable_part.len() + data.len() + 7 > MAX_ADDRESS_LEN
    {
        return Err(PayoutAddressError::InvalidBech32);
    }
    let mut checksum_input = expand_human_readable_part(human_readable_part);
    checksum_input.extend_from_slice(data);
    checksum_input.extend_from_slice(&[0; 6]);
    let constant = match encoding {
        Bech32Encoding::Bech32 => BECH32_CHECKSUM,
        Bech32Encoding::Bech32m => BECH32M_CHECKSUM,
    };
    let checksum = polymod(&checksum_input) ^ constant;
    let mut output = String::with_capacity(human_readable_part.len() + data.len() + 7);
    output.push_str(human_readable_part);
    output.push('1');
    for value in data {
        output.push(char::from(BECH32_ALPHABET[usize::from(*value)]));
    }
    for index in 0..6 {
        let shift = 5 * (5 - index);
        output.push(char::from(
            BECH32_ALPHABET[((checksum >> shift) & 31) as usize],
        ));
    }
    Ok(output)
}

fn decode_bech32(value: &str) -> Result<(String, Vec<u8>, Bech32Encoding), PayoutAddressError> {
    if value.len() < 8 || value.len() > MAX_ADDRESS_LEN || !value.is_ascii() {
        return Err(PayoutAddressError::InvalidBech32);
    }
    let has_lowercase = value.bytes().any(|byte| byte.is_ascii_lowercase());
    let has_uppercase = value.bytes().any(|byte| byte.is_ascii_uppercase());
    if has_lowercase && has_uppercase {
        return Err(PayoutAddressError::InvalidBech32);
    }
    let normalized = value.to_ascii_lowercase();
    let separator = normalized
        .rfind('1')
        .ok_or(PayoutAddressError::InvalidBech32)?;
    if separator == 0 || normalized.len() - separator - 1 < 6 {
        return Err(PayoutAddressError::InvalidBech32);
    }
    let human_readable_part = &normalized[..separator];
    if !human_readable_part
        .bytes()
        .all(|byte| (33..=126).contains(&byte))
    {
        return Err(PayoutAddressError::InvalidBech32);
    }
    let data = normalized[separator + 1..]
        .bytes()
        .map(|encoded| {
            BECH32_ALPHABET
                .iter()
                .position(|candidate| *candidate == encoded)
                .map(|index| index as u8)
                .ok_or(PayoutAddressError::InvalidBech32)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut checksum_input = expand_human_readable_part(human_readable_part);
    checksum_input.extend_from_slice(&data);
    let encoding = match polymod(&checksum_input) {
        BECH32_CHECKSUM => Bech32Encoding::Bech32,
        BECH32M_CHECKSUM => Bech32Encoding::Bech32m,
        _ => return Err(PayoutAddressError::InvalidBech32),
    };
    Ok((
        human_readable_part.to_owned(),
        data[..data.len() - 6].to_vec(),
        encoding,
    ))
}

fn expand_human_readable_part(value: &str) -> Vec<u8> {
    let mut expanded = Vec::with_capacity(value.len() * 2 + 1);
    expanded.extend(value.bytes().map(|byte| byte >> 5));
    expanded.push(0);
    expanded.extend(value.bytes().map(|byte| byte & 31));
    expanded
}

fn polymod(values: &[u8]) -> u32 {
    const GENERATORS: [u32; 5] = [
        0x3b6a_57b2,
        0x2650_8e6d,
        0x1ea1_19fa,
        0x3d42_33dd,
        0x2a14_62b3,
    ];
    let mut checksum = 1_u32;
    for value in values {
        let top = checksum >> 25;
        checksum = ((checksum & 0x01ff_ffff) << 5) ^ u32::from(*value);
        for (index, generator) in GENERATORS.iter().enumerate() {
            if ((top >> index) & 1) != 0 {
                checksum ^= generator;
            }
        }
    }
    checksum
}

fn convert_bits(
    values: &[u8],
    from_bits: u32,
    to_bits: u32,
    pad: bool,
) -> Result<Vec<u8>, PayoutAddressError> {
    let mut accumulator = 0_u32;
    let mut bit_count = 0_u32;
    let max_output = (1_u32 << to_bits) - 1;
    let max_accumulator = (1_u32 << (from_bits + to_bits - 1)) - 1;
    let mut converted = Vec::new();
    for value in values {
        if (u32::from(*value) >> from_bits) != 0 {
            return Err(PayoutAddressError::InvalidWitnessProgram);
        }
        accumulator = ((accumulator << from_bits) | u32::from(*value)) & max_accumulator;
        bit_count += from_bits;
        while bit_count >= to_bits {
            bit_count -= to_bits;
            converted.push(((accumulator >> bit_count) & max_output) as u8);
        }
    }
    if pad {
        if bit_count > 0 {
            converted.push(((accumulator << (to_bits - bit_count)) & max_output) as u8);
        }
    } else if bit_count >= from_bits || ((accumulator << (to_bits - bit_count)) & max_output) != 0 {
        return Err(PayoutAddressError::InvalidWitnessProgram);
    }
    Ok(converted)
}

fn double_sha256(bytes: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(bytes);
    Sha256::digest(first).into()
}

#[cfg(test)]
mod tests;
