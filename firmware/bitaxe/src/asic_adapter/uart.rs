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
        self.driver.wait_tx_done(ticks(timeout_ms))?;
        Ok(())
    }

    pub fn write_frame(&mut self, frame: &[u8]) -> Result<()> {
        let written = self.driver.write(frame)?;
        ensure!(written == frame.len(), "partial BM1366 UART frame write");
        Ok(())
    }

    pub fn read_exact(&mut self, len: usize, timeout_ms: u32) -> Result<Vec<u8>> {
        debug_assert_eq!(CHIP_DETECT_READ_LEN, 11);
        debug_assert_eq!(CHIP_DETECT_TIMEOUT_MS, 1_000);
        debug_assert_eq!(RESULT_WORK_TIMEOUT_MS, 10_000);

        let mut buf = vec![0; len];
        let read = self.driver.read(&mut buf, ticks(timeout_ms))?;
        if read != len {
            self.clear_rx()?;
            anyhow::bail!("partial BM1366 UART read: expected {len} bytes, read {read}");
        }

        Ok(buf)
    }

    pub fn clear_rx(&mut self) -> Result<()> {
        self.driver.clear_rx()?;
        Ok(())
    }
}

fn ticks(timeout_ms: u32) -> esp_idf_svc::sys::TickType_t {
    TickType::new_millis(u64::from(timeout_ms)).ticks()
}
