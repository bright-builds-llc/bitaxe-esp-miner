#[path = "production_mining_session/revocation.rs"]
mod revocation;
#[path = "production_mining_session/shutdown_budget.rs"]
mod shutdown_budget;
#[path = "production_mining_session/transport.rs"]
mod transport;

mod noise_serial_runtime {
    pub(crate) fn install_transport(
        _primary: std::sync::Weak<crate::transport::borrow::NoiseBorrowHandle>,
        _fallback: std::sync::Weak<crate::transport::borrow::NoiseBorrowHandle>,
    ) -> bool {
        true
    }
}

// This host target exercises the real V1/borrow owner. Native V2 is unavailable,
// never a fake successful protocol or resource proof; shared V2 I/O has OS tests.
mod v2_serial_runtime {
    use crate::revocation::WorkPermit;
    use bitaxe_stratum::{
        v1::{
            production_session::ProductionTransportEpoch, production_work::PoolSessionGeneration,
        },
        v2::frame::Frame,
    };
    pub(crate) trait ShareIo {
        fn authenticated(&mut self);
        fn open(&self) -> bool;
        fn next_write(&mut self) -> Result<Option<(Frame, WorkPermit)>, ()>;
        fn frame(&mut self, frame: Frame);
        fn written(&mut self, sequence: u32);
    }
    pub(crate) fn run_share_transport(
        _endpoint: std::net::SocketAddrV4,
        _authority: [u8; 32],
        _permit: WorkPermit,
        _generation: PoolSessionGeneration,
        _epoch: ProductionTransportEpoch,
        _io: &mut dyn ShareIo,
    ) -> Result<(), ()> {
        Err(())
    }
    pub(crate) fn share_completed() {}
    pub(crate) fn capture_failure() {}
}
