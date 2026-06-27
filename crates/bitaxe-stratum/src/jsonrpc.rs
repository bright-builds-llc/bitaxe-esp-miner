/// JSON-RPC request identifier used by Stratum v1 client messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StratumRequestId(u64);

impl StratumRequestId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
