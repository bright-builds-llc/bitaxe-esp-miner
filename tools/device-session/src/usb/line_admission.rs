/// Establishes a trusted line boundary for a newly opened receive-only reader.
pub(super) struct ReceiveLineAdmission {
    boundary_observed: bool,
}

impl ReceiveLineAdmission {
    pub(super) const fn new() -> Self {
        Self {
            boundary_observed: false,
        }
    }

    pub(super) fn reset(&mut self) {
        self.boundary_observed = false;
    }

    pub(super) fn admit<'a>(&mut self, chunk: &'a [u8]) -> Option<&'a [u8]> {
        if self.boundary_observed {
            return (!chunk.is_empty()).then_some(chunk);
        }
        let boundary = chunk.iter().position(|byte| *byte == b'\n')?;
        self.boundary_observed = true;
        let admitted = &chunk[boundary + 1..];
        (!admitted.is_empty()).then_some(admitted)
    }
}
