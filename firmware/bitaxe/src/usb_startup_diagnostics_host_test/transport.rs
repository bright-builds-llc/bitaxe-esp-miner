//! Deterministic transport seam for production startup writer tests.
use super::*;
pub(crate) use crate::usb_tx_measurement as measurement;
pub(crate) use crate::usb_write_failure::{WriteFailure, WriteObservation, WriteObservationStage};
pub static DELAY_MS: AtomicU32 = AtomicU32::new(0);
pub static PARTIAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub static SINK: Mutex<Option<mpsc::Sender<String>>> = Mutex::new(None);
pub static BLOCK_NEXT: Mutex<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>> = Mutex::new(None);
pub fn write_if(bytes: &[u8], admitted: impl Fn() -> bool) -> anyhow::Result<()> {
    let maybe_block = BLOCK_NEXT.lock().expect("test block").take();
    if let Some((entered, release)) = maybe_block {
        entered.send(())?;
        release.recv_timeout(Duration::from_secs(2))?;
    }
    anyhow::ensure!(admitted(), "serial_output_revoked");
    std::thread::sleep(Duration::from_millis(u64::from(
        DELAY_MS.load(Ordering::Relaxed),
    )));
    anyhow::ensure!(admitted(), "serial_output_revoked");
    let sink = SINK
        .lock()
        .map_err(|_| anyhow::anyhow!("test sink poisoned"))?;
    sink.as_ref()
        .ok_or_else(|| anyhow::anyhow!("test sink missing"))?
        .send(std::str::from_utf8(bytes)?.to_owned())?;
    Ok(())
}
pub fn resynchronize_if(
    _retained: &mut measurement::Retained,
    admitted: impl Fn() -> bool,
) -> anyhow::Result<()> {
    anyhow::ensure!(admitted(), "serial_output_revoked");
    Ok(())
}
pub fn write_observed_if(
    bytes: &[u8],
    admitted: impl Fn() -> bool,
    mut observe: impl FnMut(WriteObservation),
) -> anyhow::Result<()> {
    let result = write_if(bytes, admitted);
    let queued_bytes = if result.is_ok() { bytes.len() } else { 0 };
    observe(WriteObservation {
        stage: if result.is_ok() {
            WriteObservationStage::Completed
        } else {
            WriteObservationStage::Abandoned
        },
        at_ms: crate::runtime_uptime::millis(),
        queued_bytes,
        record_bytes: bytes.len(),
    });
    result
}
pub fn write_measured_if(
    _retained: &mut measurement::Retained,
    bytes: &[u8],
    admitted: impl Fn() -> bool,
    observe: impl FnMut(WriteObservation),
    _category: measurement::Category,
    _kind: measurement::RecordKind,
) -> anyhow::Result<()> {
    write_observed_if(bytes, admitted, observe)
}
pub fn has_partial_output() -> bool {
    PARTIAL.load(Ordering::Acquire)
}
