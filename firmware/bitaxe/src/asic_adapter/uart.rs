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
const UART_BUF_SIZE: usize = 1024;
const UART_RX_BUFFER_BYTES: usize = UART_BUF_SIZE * 2;
const READ_CHUNK_MAX: usize = 64;

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
        self.read_accumulate(len, timeout_ms)
    }

    pub fn read_accumulate(&mut self, len: usize, timeout_ms: u32) -> Result<Vec<u8>> {
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
            let read = self
                .driver
                .read(&mut scratch[..chunk_cap], ticks(remaining_ms))?;
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
            if uart_trace_enabled() && !buf.is_empty() {
                log::info!(
                    "asic_uart_trace=partial_frame hex={} expected_len={len} actual_len={}",
                    hex_bytes(&buf),
                    buf.len()
                );
            }
            self.clear_rx()?;
            anyhow::bail!("partial BM1366 UART read: expected {len} bytes, read {}", buf.len());
        }

        if uart_trace_enabled() {
            log::info!(
                "asic_uart_trace=rx_complete read_count={read_index} hex={}",
                hex_bytes(&buf)
            );
        }

        Ok(buf)
    }

    pub fn clear_rx(&mut self) -> Result<()> {
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

fn ticks(timeout_ms: u32) -> esp_idf_svc::sys::TickType_t {
    TickType::new_millis(u64::from(timeout_ms)).ticks()
}
