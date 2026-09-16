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
