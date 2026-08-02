use crate::error::StratumV1Error;
use crate::v1::coinbase::hex_decode;
use crate::v1::messages::{MiningNotify, MAX_EXTRANONCE_2_LEN};

const BIP110_SIGNAL_BIT: u32 = 4;
const BIP110_SIGNAL_EXPIRY_BLOCK: u32 = 965_664;
const COINBASE_PREVIOUS_OUTPUT_LEN: usize = 36;

/// Maximum decoded outputs retained for the operator projection.
pub const MAX_COINBASE_OUTPUTS: usize = 6;

/// Standard script shape recognized without performing address encoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoinbaseScriptKind {
    /// Pay to public key hash.
    P2pkh,
    /// Pay to script hash.
    P2sh,
    /// Version-zero pay to witness public key hash.
    P2wpkh,
    /// Version-zero pay to witness script hash.
    P2wsh,
    /// Version-one pay to Taproot output key.
    P2tr,
    /// Provably unspendable data output.
    OpReturn,
    /// Script shape outside the recognized reference set.
    Unknown,
}

/// Decoded value and raw script for one retained transaction output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoinbaseOutput {
    /// Output value in satoshis.
    pub value_satoshis: u64,
    /// Raw script bytes; address rendering is owned by `STR-012`.
    pub script_pubkey: Vec<u8>,
    /// Structural script classification without an encoded address.
    pub script_kind: CoinbaseScriptKind,
}

/// Deterministic, effect-free projection of a decoded coinbase notification.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodedCoinbase {
    /// Network difficulty derived from the notification's compact target.
    pub network_difficulty: f64,
    /// BIP-34 block height extracted from the input ScriptSig.
    pub block_height: u32,
    /// Printable pool tag with both extranonce segments removed.
    pub maybe_scriptsig: Option<String>,
    /// At most [`MAX_COINBASE_OUTPUTS`] retained outputs.
    pub outputs: Vec<CoinbaseOutput>,
    /// Sum of every decoded output, including outputs beyond the retention cap.
    pub total_value_satoshis: u64,
    /// Whether detailed output and signaling projection was requested.
    pub decode_transaction: bool,
    /// Whether the decoded fields satisfy the reference BIP-54 signal rule.
    pub bip54_signaling: bool,
    /// Whether the decoded fields satisfy the reference BIP-110 signal rule.
    pub bip110_signaling: bool,
}

/// Decodes one Bitcoin CompactSize value and advances `offset` on success.
pub fn decode_compact_size(bytes: &[u8], offset: &mut usize) -> Result<u64, StratumV1Error> {
    let prefix = read_exact(bytes, offset, 1, "compact_size")?[0];
    match prefix {
        0x00..=0xfc => Ok(u64::from(prefix)),
        0xfd => Ok(u64::from(read_u16(bytes, offset, "compact_size")?)),
        0xfe => Ok(u64::from(read_u32(bytes, offset, "compact_size")?)),
        0xff => read_u64(bytes, offset, "compact_size"),
    }
}

/// Reassembles and safely decodes a split Stratum v1 coinbase transaction.
pub fn decode_coinbase_notification(
    notification: &MiningNotify,
    extranonce1: &str,
    extranonce2_len: usize,
    decode_transaction: bool,
) -> Result<DecodedCoinbase, StratumV1Error> {
    if extranonce2_len > usize::from(MAX_EXTRANONCE_2_LEN) {
        return Err(invalid(
            "extranonce2_len",
            "exceeds MAX_EXTRANONCE_2_LEN 32",
        ));
    }

    let coinbase_1 = hex_decode(&notification.coinbase_1, "coinbase_1")?;
    let coinbase_2 = hex_decode(&notification.coinbase_2, "coinbase_2")?;
    let extranonce1 = hex_decode(extranonce1, "extranonce1")?;
    let (transaction, extranonce_range) =
        assemble_transaction(&coinbase_1, &extranonce1, extranonce2_len, &coinbase_2)?;
    let network_difficulty = network_difficulty(notification.nbits)?;

    decode_transaction_bytes(
        &transaction,
        extranonce_range,
        notification.version,
        network_difficulty,
        decode_transaction,
    )
}

fn assemble_transaction(
    coinbase_1: &[u8],
    extranonce1: &[u8],
    extranonce2_len: usize,
    coinbase_2: &[u8],
) -> Result<(Vec<u8>, std::ops::Range<usize>), StratumV1Error> {
    let extranonce_len = extranonce1
        .len()
        .checked_add(extranonce2_len)
        .ok_or_else(|| invalid("coinbase", "extranonce length overflow"))?;
    let total_len = coinbase_1
        .len()
        .checked_add(extranonce_len)
        .and_then(|len| len.checked_add(coinbase_2.len()))
        .ok_or_else(|| invalid("coinbase", "transaction length overflow"))?;
    let extranonce_start = coinbase_1.len();
    let extranonce_end = extranonce_start
        .checked_add(extranonce_len)
        .ok_or_else(|| invalid("coinbase", "extranonce range overflow"))?;

    let mut transaction = Vec::with_capacity(total_len);
    transaction.extend_from_slice(coinbase_1);
    transaction.extend_from_slice(extranonce1);
    transaction.resize(extranonce_end, 0);
    transaction.extend_from_slice(coinbase_2);

    Ok((transaction, extranonce_start..extranonce_end))
}

fn decode_transaction_bytes(
    transaction: &[u8],
    extranonce_range: std::ops::Range<usize>,
    block_version: u32,
    network_difficulty: f64,
    decode_transaction: bool,
) -> Result<DecodedCoinbase, StratumV1Error> {
    let mut offset = 0;
    let _transaction_version = read_u32(transaction, &mut offset, "transaction_version")?;
    let input_count = decode_compact_size(transaction, &mut offset)?;
    if input_count != 1 {
        return Err(invalid("coinbase_input_count", "expected one input"));
    }

    read_exact(
        transaction,
        &mut offset,
        COINBASE_PREVIOUS_OUTPUT_LEN,
        "coinbase_previous_output",
    )?;
    let script_len = compact_size_as_usize(transaction, &mut offset, "scriptsig_len")?;
    let script_start = offset;
    let script = read_exact(transaction, &mut offset, script_len, "scriptsig")?;
    let script_end = offset;
    let (block_height, height_prefix_len) = decode_block_height(script)?;
    let maybe_scriptsig = scriptsig_text(
        transaction,
        script_start..script_end,
        extranonce_range,
        height_prefix_len,
    )?;
    let sequence = read_u32(transaction, &mut offset, "sequence")?;
    let output_count = decode_compact_size(transaction, &mut offset)?;

    let mut outputs = Vec::with_capacity(MAX_COINBASE_OUTPUTS);
    let mut total_value_satoshis = 0_u64;
    for output_index in 0..output_count {
        let value_satoshis = read_u64(transaction, &mut offset, "output_value")?;
        total_value_satoshis = total_value_satoshis
            .checked_add(value_satoshis)
            .ok_or_else(|| invalid("output_value", "total value overflow"))?;
        let script_len = compact_size_as_usize(transaction, &mut offset, "script_pubkey_len")?;
        let script_pubkey = read_exact(transaction, &mut offset, script_len, "script_pubkey")?;

        if decode_transaction && output_index < MAX_COINBASE_OUTPUTS as u64 {
            outputs.push(CoinbaseOutput {
                value_satoshis,
                script_pubkey: script_pubkey.to_vec(),
                script_kind: classify_script(script_pubkey),
            });
        }
    }

    let lock_time = read_u32(transaction, &mut offset, "lock_time")?;
    if offset != transaction.len() {
        return Err(invalid("coinbase", "unexpected trailing transaction bytes"));
    }

    let bip54_signaling = decode_transaction
        && block_height.checked_sub(1) == Some(lock_time)
        && sequence != u32::MAX;
    let bip110_signaling = decode_transaction
        && block_height < BIP110_SIGNAL_EXPIRY_BLOCK
        && block_version & (1 << BIP110_SIGNAL_BIT) != 0;

    Ok(DecodedCoinbase {
        network_difficulty,
        block_height,
        maybe_scriptsig,
        outputs,
        total_value_satoshis,
        decode_transaction,
        bip54_signaling,
        bip110_signaling,
    })
}

fn decode_block_height(script: &[u8]) -> Result<(u32, usize), StratumV1Error> {
    let Some((&height_len, height_bytes)) = script.split_first() else {
        return Err(invalid("block_height", "missing BIP-34 height"));
    };
    if !(1..=4).contains(&height_len) {
        return Err(invalid("block_height", "expected 1 to 4 bytes"));
    }

    let height_len = usize::from(height_len);
    let Some(height_bytes) = height_bytes.get(..height_len) else {
        return Err(invalid("block_height", "truncated BIP-34 height"));
    };
    let mut padded = [0_u8; 4];
    padded[..height_len].copy_from_slice(height_bytes);
    Ok((u32::from_le_bytes(padded), height_len + 1))
}

fn scriptsig_text(
    transaction: &[u8],
    script_range: std::ops::Range<usize>,
    extranonce_range: std::ops::Range<usize>,
    height_prefix_len: usize,
) -> Result<Option<String>, StratumV1Error> {
    if extranonce_range.start < script_range.start
        || extranonce_range.end > script_range.end
        || extranonce_range.start > extranonce_range.end
    {
        return Err(invalid(
            "coinbase_split",
            "extranonce must be contained in ScriptSig",
        ));
    }

    let tag_start = script_range
        .start
        .checked_add(height_prefix_len)
        .ok_or_else(|| invalid("scriptsig", "tag range overflow"))?;
    if tag_start > extranonce_range.start {
        return Err(invalid(
            "coinbase_split",
            "extranonce overlaps BIP-34 height",
        ));
    }

    let before_extranonce = &transaction[tag_start..extranonce_range.start];
    let after_extranonce = &transaction[extranonce_range.end..script_range.end];
    if before_extranonce.is_empty() && after_extranonce.is_empty() {
        return Ok(None);
    }

    let tag = before_extranonce
        .iter()
        .chain(after_extranonce)
        .map(|byte| {
            if byte.is_ascii_graphic() || *byte == b' ' {
                char::from(*byte)
            } else {
                '.'
            }
        })
        .collect();
    Ok(Some(tag))
}

fn network_difficulty(nbits: u32) -> Result<f64, StratumV1Error> {
    let mantissa = nbits & 0x007f_ffff;
    if mantissa == 0 {
        return Err(invalid("nbits", "zero compact target mantissa"));
    }
    let exponent = ((nbits >> 24) & 0xff) as i32;
    let target = f64::from(mantissa) * 256_f64.powi(exponent - 3);
    let difficulty = (2_f64.powi(208) * 65_535_f64) / target;
    if !difficulty.is_finite() {
        return Err(invalid("nbits", "non-finite network difficulty"));
    }
    Ok(difficulty)
}

fn compact_size_as_usize(
    bytes: &[u8],
    offset: &mut usize,
    field: &'static str,
) -> Result<usize, StratumV1Error> {
    usize::try_from(decode_compact_size(bytes, offset)?)
        .map_err(|_| invalid(field, "value does not fit usize"))
}

fn read_u16(bytes: &[u8], offset: &mut usize, field: &'static str) -> Result<u16, StratumV1Error> {
    let value = read_exact(bytes, offset, 2, field)?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: &mut usize, field: &'static str) -> Result<u32, StratumV1Error> {
    let value = read_exact(bytes, offset, 4, field)?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn read_u64(bytes: &[u8], offset: &mut usize, field: &'static str) -> Result<u64, StratumV1Error> {
    let value = read_exact(bytes, offset, 8, field)?;
    Ok(u64::from_le_bytes([
        value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7],
    ]))
}

fn read_exact<'a>(
    bytes: &'a [u8],
    offset: &mut usize,
    len: usize,
    field: &'static str,
) -> Result<&'a [u8], StratumV1Error> {
    let end = offset
        .checked_add(len)
        .ok_or_else(|| invalid(field, "offset overflow"))?;
    let Some(value) = bytes.get(*offset..end) else {
        return Err(invalid(field, "truncated value"));
    };
    *offset = end;
    Ok(value)
}

fn classify_script(script: &[u8]) -> CoinbaseScriptKind {
    if script.len() == 25
        && script.starts_with(&[0x76, 0xa9, 0x14])
        && script.ends_with(&[0x88, 0xac])
    {
        return CoinbaseScriptKind::P2pkh;
    }
    if script.len() == 23 && script.starts_with(&[0xa9, 0x14]) && script.ends_with(&[0x87]) {
        return CoinbaseScriptKind::P2sh;
    }
    if script.len() == 22 && script.starts_with(&[0x00, 0x14]) {
        return CoinbaseScriptKind::P2wpkh;
    }
    if script.len() == 34 && script.starts_with(&[0x00, 0x20]) {
        return CoinbaseScriptKind::P2wsh;
    }
    if script.len() == 34 && script.starts_with(&[0x51, 0x20]) {
        return CoinbaseScriptKind::P2tr;
    }
    if script.first() == Some(&0x6a) {
        return CoinbaseScriptKind::OpReturn;
    }
    CoinbaseScriptKind::Unknown
}

fn invalid(field: &'static str, reason: &'static str) -> StratumV1Error {
    StratumV1Error::InvalidField { field, reason }
}
