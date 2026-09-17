use sha2::{Digest, Sha256};

pub(super) const VERSION: u32 = 0x2000_0000;
pub(super) const MASK: u32 = 0x1fff_e000;
pub(super) const NBITS: u32 = 0x207f_ffff;
pub(super) const TARGET: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xc0, 0xff, 0x3f, 0, 0,
    0, 0, 0,
];
pub(super) fn sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
pub(super) fn header(
    version: u32,
    previous: [u8; 32],
    merkle: [u8; 32],
    ntime: u32,
    nbits: u32,
    nonce: u32,
) -> [u8; 80] {
    let mut value = [0; 80];
    value[..4].copy_from_slice(&version.to_le_bytes());
    value[4..36].copy_from_slice(&previous);
    value[36..68].copy_from_slice(&merkle);
    value[68..72].copy_from_slice(&ntime.to_le_bytes());
    value[72..76].copy_from_slice(&nbits.to_le_bytes());
    value[76..].copy_from_slice(&nonce.to_le_bytes());
    value
}
pub(super) fn hash(header: &[u8; 80]) -> [u8; 32] {
    Sha256::digest(Sha256::digest(header)).into()
}
pub(super) fn meets_target(hash: &[u8; 32], target: &[u8; 32]) -> bool {
    hash.iter().rev().cmp(target.iter().rev()) != std::cmp::Ordering::Greater
}
pub(super) fn valid_version(version: u32) -> bool {
    version & !MASK == VERSION
}
