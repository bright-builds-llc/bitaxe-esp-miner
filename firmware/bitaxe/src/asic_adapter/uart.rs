use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{ensure, Result};
use esp_idf_svc::hal::{
    delay::TickType,
    gpio::{InputPin, OutputPin},
    uart::{config, Uart, UartDriver},
    units::Hertz,
};

pub const UART_INITIAL_BAUD: u32 = 115_200;
pub const UART_TX_PIN: i32 = 17;
pub const UART_RX_PIN: i32 = 18;
pub const CHIP_DETECT_READ_LEN: usize = 11;
pub const CHIP_DETECT_TIMEOUT_MS: u32 = 1_000;
pub const RESULT_WORK_TIMEOUT_MS: u32 = 10_000;
pub const WAIT_TX_DONE_TIMEOUT_MS: u32 = 1_000;
/// Emit compact flood-safe RX summary every N production result polls.
pub const RX_ACQUISITION_SUMMARY_EVERY_N_POLLS: u32 = 50;
const UART_BUF_SIZE: usize = 1024;
const UART_RX_BUFFER_BYTES: usize = UART_BUF_SIZE * 2;
const READ_CHUNK_MAX: usize = 64;

static RX_ACQ_IDLE: AtomicU32 = AtomicU32::new(0);
static RX_ACQ_PARTIAL: AtomicU32 = AtomicU32::new(0);
static RX_ACQ_CLEAR: AtomicU32 = AtomicU32::new(0);
static RX_ACQ_COMPLETE: AtomicU32 = AtomicU32::new(0);
static RX_ACQ_POLL_TICKS: AtomicU32 = AtomicU32::new(0);

/// Snapshot of flood-safe RX-acquisition counters (integers only; no hex).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RxAcquisitionCounts {
    pub idle: u32,
    pub partial: u32,
    pub clear: u32,
    pub complete: u32,
}

/// Read current RX-acquisition counters without resetting them.
#[must_use]
pub fn rx_acquisition_counts() -> RxAcquisitionCounts {
    RxAcquisitionCounts {
        idle: RX_ACQ_IDLE.load(Ordering::Relaxed),
        partial: RX_ACQ_PARTIAL.load(Ordering::Relaxed),
        clear: RX_ACQ_CLEAR.load(Ordering::Relaxed),
        complete: RX_ACQ_COMPLETE.load(Ordering::Relaxed),
    }
}

/// Record one production result-poll tick; emit compact summary every N polls.
///
/// Marker shape (no hex): `asic_rx_acquisition_summary idle=N partial=N clear=N complete=N`
pub fn note_result_poll_and_maybe_emit_summary() {
    let ticks = RX_ACQ_POLL_TICKS
        .fetch_add(1, Ordering::Relaxed)
        .saturating_add(1);
    if ticks % RX_ACQUISITION_SUMMARY_EVERY_N_POLLS != 0 {
        return;
    }
    emit_rx_acquisition_summary();
}

fn emit_rx_acquisition_summary() {
    let counts = rx_acquisition_counts();
    log::info!(
        "asic_rx_acquisition_summary idle={} partial={} clear={} complete={}",
        counts.idle,
        counts.partial,
        counts.clear,
        counts.complete
    );
}

fn record_rx_idle() {
    RX_ACQ_IDLE.fetch_add(1, Ordering::Relaxed);
}

fn record_rx_partial() {
    RX_ACQ_PARTIAL.fetch_add(1, Ordering::Relaxed);
}

fn record_rx_clear() {
    RX_ACQ_CLEAR.fetch_add(1, Ordering::Relaxed);
}

fn record_rx_complete() {
    RX_ACQ_COMPLETE.fetch_add(1, Ordering::Relaxed);
}

enum ReadAccumulateOutcome {
    Complete(Vec<u8>),
    Idle,
}

pub struct AsicUart<'d> {
    driver: UartDriver<'d>,
}

impl<'d> AsicUart<'d> {
    pub fn new<UART, TX, RX>(uart: UART, tx: TX, rx: RX) -> Result<Self>
    where
        UART: Uart + 'd,
        TX: OutputPin + 'd,
        RX: InputPin + 'd,
    {
        debug_assert_eq!(UART_TX_PIN, 17);
        debug_assert_eq!(UART_RX_PIN, 18);

        let config = config::Config::new()
            .baudrate(Hertz(UART_INITIAL_BAUD))
            .data_bits(config::DataBits::DataBits8)
            .parity_none()
            .stop_bits(config::StopBits::STOP1)
            .flow_control(config::FlowControl::None)
            .rx_fifo_size(UART_RX_BUFFER_BYTES);
        let driver = UartDriver::new(
            uart,
            tx,
            rx,
            Option::<RX>::None,
            Option::<TX>::None,
            &config,
        )?;

        Ok(Self { driver })
    }

    pub fn change_baud(&mut self, baud: u32) -> Result<()> {
        self.wait_tx_done(WAIT_TX_DONE_TIMEOUT_MS)?;
        self.driver.change_baudrate(Hertz(baud))?;
        Ok(())
    }

    pub fn wait_tx_done(&self, timeout_ms: u32) -> Result<()> {
        let started = std::time::Instant::now();
        let outcome = self.driver.wait_tx_done(ticks(timeout_ms));
        if uart_trace_enabled() {
            log::info!(
                "asic_uart_trace=wait_tx_done outcome={outcome:?} elapsed_ms={}",
                started.elapsed().as_millis()
            );
        }
        outcome?;
        Ok(())
    }

    pub fn write_frame(&mut self, frame: &[u8]) -> Result<()> {
        if uart_trace_enabled() {
            log::info!(
                "asic_uart_trace=tx len={} hex={}",
                frame.len(),
                hex_bytes(frame)
            );
        }
        let written = self.driver.write(frame)?;
        ensure!(written == frame.len(), "partial BM1366 UART frame write");
        Ok(())
    }

    pub fn read_exact(&mut self, len: usize, timeout_ms: u32) -> Result<Vec<u8>> {
        match self.read_accumulate_inner(len, timeout_ms)? {
            ReadAccumulateOutcome::Complete(frame) => Ok(frame),
            ReadAccumulateOutcome::Idle => {
                anyhow::bail!("partial BM1366 UART read: expected {len} bytes, read 0")
            }
        }
    }

    pub fn read_accumulate(&mut self, len: usize, timeout_ms: u32) -> Result<Vec<u8>> {
        self.read_exact(len, timeout_ms)
    }

    pub fn try_read_exact(&mut self, len: usize, timeout_ms: u32) -> Result<Option<Vec<u8>>> {
        match self.read_accumulate_inner(len, timeout_ms)? {
            ReadAccumulateOutcome::Complete(frame) => Ok(Some(frame)),
            ReadAccumulateOutcome::Idle => Ok(None),
        }
    }

    fn read_accumulate_inner(
        &mut self,
        len: usize,
        timeout_ms: u32,
    ) -> Result<ReadAccumulateOutcome> {
        debug_assert_eq!(CHIP_DETECT_READ_LEN, 11);
        debug_assert_eq!(CHIP_DETECT_TIMEOUT_MS, 1_000);
        debug_assert_eq!(RESULT_WORK_TIMEOUT_MS, 10_000);

        let started = std::time::Instant::now();
        let deadline = started + std::time::Duration::from_millis(u64::from(timeout_ms));
        let mut buf = Vec::with_capacity(len);
        let mut scratch = [0_u8; READ_CHUNK_MAX];
        let mut read_index = 0_u32;

        while buf.len() < len {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            let remaining_ms = remaining.as_millis().min(u128::from(u32::MAX)) as u32;
            let chunk_cap = (len - buf.len()).min(scratch.len());
            // Upstream SERIAL_rx returns 0 on idle timeout. esp-idf UartDriver::read
            // often surfaces ESP_ERR_TIMEOUT instead of Ok(0); treat empty-buffer
            // timeout as zero bytes so drain-until-idle can exit cleanly.
            let read = match self
                .driver
                .read(&mut scratch[..chunk_cap], ticks(remaining_ms))
            {
                Ok(n) => n,
                Err(error) if is_uart_timeout_error(&error) && buf.is_empty() => 0,
                Err(error) => return Err(error.into()),
            };
            read_index = read_index.saturating_add(1);

            if uart_trace_enabled() {
                log::info!(
                    "asic_uart_trace=rx read_index={read_index} chunk_bytes={read} total_bytes={} remaining_ms={remaining_ms}",
                    buf.len().saturating_add(read)
                );
                if read > 0 {
                    log::info!(
                        "asic_uart_trace=rx_chunk hex={}",
                        hex_bytes(&scratch[..read])
                    );
                }
            }

            if read == 0 {
                continue;
            }

            buf.extend_from_slice(&scratch[..read]);
        }

        if buf.len() != len {
            if buf.is_empty() {
                record_rx_idle();
                if uart_trace_enabled() {
                    log::info!("asic_uart_trace=rx_idle timeout_ms={timeout_ms}");
                }
                return Ok(ReadAccumulateOutcome::Idle);
            }
            record_rx_partial();
            if uart_trace_enabled() {
                log::info!(
                    "asic_uart_trace=partial_frame hex={} expected_len={len} actual_len={}",
                    hex_bytes(&buf),
                    buf.len()
                );
            }
            self.clear_rx()?;
            anyhow::bail!(
                "partial BM1366 UART read: expected {len} bytes, read {}",
                buf.len()
            );
        }

        record_rx_complete();
        if uart_trace_enabled() {
            log::info!(
                "asic_uart_trace=rx_complete read_count={read_index} hex={}",
                hex_bytes(&buf)
            );
        }

        Ok(ReadAccumulateOutcome::Complete(buf))
    }

    pub fn clear_rx(&mut self) -> Result<()> {
        record_rx_clear();
        if uart_trace_enabled() {
            log::info!("asic_uart_trace=clear_rx");
        }
        self.driver.clear_rx()?;
        Ok(())
    }
}

fn uart_trace_enabled() -> bool {
    crate::mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge()
        || option_env!("BITAXE_ASIC_UART_TRACE") == Some("1")
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_uart_timeout_error(error: &esp_idf_svc::sys::EspError) -> bool {
    error.code() == esp_idf_svc::sys::ESP_ERR_TIMEOUT
}

fn ticks(timeout_ms: u32) -> esp_idf_svc::sys::TickType_t {
    TickType::new_millis(u64::from(timeout_ms)).ticks()
}
