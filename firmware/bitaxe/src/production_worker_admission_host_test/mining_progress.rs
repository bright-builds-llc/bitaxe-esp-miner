pub(crate) static LAST: std::sync::Mutex<Option<(u32, [u64; 2])>> = std::sync::Mutex::new(None);
pub(crate) fn capture(
    generation: u32,
    _snapshot: &bitaxe_stratum::v1::production_session::ProductionSessionSnapshot,
    counts: [u64; 2],
) {
    *LAST.lock().expect("test capture") = Some((generation, counts));
}
