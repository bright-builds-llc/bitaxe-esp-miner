//! Aggregate board adapter and deterministic observation journal.
use crate::{
    asic::Bm1366,
    memory::CapabilityHeap,
    peripherals::{Dynamics, Peripherals},
    scheduler::DeterministicScheduler,
    storage::{PersistentStore, ResetReason},
    transport::VirtualTransport,
    ModelError,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoardConfig {
    pub seed: u64,
    pub dynamics: Dynamics,
}
impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            dynamics: Dynamics::default(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardEvent {
    SensorPublicationBlocked(bool),
    FanStalled(bool),
    Button(bool),
    I2cDelay(u64),
    NetworkDisconnect,
}
#[derive(Debug, Clone)]
pub struct VirtualBoard {
    pub scheduler: DeterministicScheduler<BoardEvent>,
    pub peripherals: Peripherals,
    pub heap: CapabilityHeap,
    pub storage: PersistentStore,
    pub asic: Bm1366,
    pub control: VirtualTransport,
    pub network: VirtualTransport,
    pub journal: Vec<(u64, BoardEvent)>,
    i2c_busy_until_ms: u64,
}
impl VirtualBoard {
    pub fn new(config: BoardConfig) -> Result<Self, ModelError> {
        Ok(Self {
            scheduler: DeterministicScheduler::new(config.seed),
            peripherals: Peripherals::new(config.dynamics)?,
            heap: CapabilityHeap::healthy(),
            storage: PersistentStore::default(),
            asic: Bm1366::default(),
            control: VirtualTransport::new(65536),
            network: VirtualTransport::new(65536),
            journal: Vec::new(),
            i2c_busy_until_ms: 0,
        })
    }
    pub fn now_ms(&self) -> u64 {
        self.scheduler.now_ms()
    }
    pub fn advance_to(&mut self, deadline_ms: u64) -> Result<(), ModelError> {
        let events = self.scheduler.advance_to(deadline_ms)?;
        for (at_ms, event) in events {
            self.peripherals
                .advance_to(at_ms, self.asic.maybe_last_payload.is_some())?;
            match event {
                BoardEvent::SensorPublicationBlocked(blocked) => {
                    self.peripherals.sensor_publication_blocked = blocked
                }
                BoardEvent::FanStalled(stalled) => self.peripherals.fan_stalled = stalled,
                BoardEvent::Button(pressed) => self.peripherals.button_pressed = pressed,
                BoardEvent::I2cDelay(delay) => {
                    self.i2c_busy_until_ms = at_ms
                        .checked_add(delay)
                        .ok_or(ModelError::Invalid("I2C time overflow"))?
                }
                BoardEvent::NetworkDisconnect => self.network.connected = false,
            }
            self.journal.push((at_ms, event));
        }
        self.peripherals
            .advance_to(deadline_ms, self.asic.maybe_last_payload.is_some())?;
        Ok(())
    }
    fn claim_i2c(&mut self) -> Result<(), ModelError> {
        if self.now_ms() < self.i2c_busy_until_ms {
            return Err(ModelError::Busy {
                until_ms: self.i2c_busy_until_ms,
            });
        }
        self.i2c_busy_until_ms = self
            .now_ms()
            .checked_add(self.peripherals.dynamics.i2c_transaction_ms)
            .ok_or(ModelError::Invalid("I2C time overflow"))?;
        Ok(())
    }
    pub fn i2c_read(
        &mut self,
        address: u8,
        register: u8,
        length: usize,
    ) -> Result<Vec<u8>, ModelError> {
        self.claim_i2c()?;
        self.peripherals.read_register(address, register, length)
    }
    pub fn i2c_write(&mut self, address: u8, register: u8, value: &[u8]) -> Result<(), ModelError> {
        self.claim_i2c()?;
        self.peripherals.write_register(address, register, value)
    }
    pub fn uart_exchange(&mut self, frame: &[u8]) -> Result<Vec<u8>, ModelError> {
        self.asic.power_enabled = self.peripherals.power_enabled;
        self.asic.reset_asserted = self.peripherals.reset_asserted;
        self.asic.exchange(frame)
    }
    pub fn reset(&mut self, reason: ResetReason) -> Result<(), ModelError> {
        self.storage.reset(reason)?;
        self.peripherals.reset_boot(self.storage.boot_ordinal);
        self.heap
            .reset_boot()
            .map_err(|_| ModelError::Invalid("heap boot epoch overflow"))?;
        self.i2c_busy_until_ms = self.now_ms();
        self.asic.reset();
        self.asic.power_enabled = false;
        self.asic.reset_asserted = true;
        self.asic.host_baud = 115200;
        self.control.reset_boot();
        self.network.reset_boot();
        Ok(())
    }
}
