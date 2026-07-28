use std::cell::Cell;
use std::rc::Rc;

use bitaxe_asic::bm1366::{
    production::ProductionWorkPayload,
    result::Bm1366ValidJobIds,
    work::{Bm1366JobId, Bm1366WorkFields},
};

use super::*;

struct FakeProductionBackend {
    send_count: Rc<Cell<u32>>,
    read_count: Rc<Cell<u32>>,
    maybe_result: Option<Bm1366NonceResult>,
}

impl FakeProductionBackend {
    fn maybe_execute(
        &self,
        command: Bm1366ProductionCommand,
        _valid_jobs: &Bm1366ValidJobIds,
    ) -> Result<Option<Bm1366NonceResult>, ProductionAsicBlocker> {
        match command {
            Bm1366ProductionCommand::SendProductionWork(_) => {
                self.send_count.set(self.send_count.get() + 1);
                Ok(None)
            }
            Bm1366ProductionCommand::ReadProductionResult => {
                self.read_count.set(self.read_count.get() + 1);
                Ok(self.maybe_result)
            }
        }
    }
}

#[test]
fn send_production_work_increments_dispatch_counter() {
    // Arrange
    let send_count = Rc::new(Cell::new(0));
    let backend = FakeProductionBackend {
        send_count: send_count.clone(),
        read_count: Rc::new(Cell::new(0)),
        maybe_result: None,
    };
    let job_id = Bm1366JobId::new(0x28);
    let payload = ProductionWorkPayload::new(job_id, sample_fields());
    let command = Bm1366ProductionCommand::SendProductionWork(payload);

    // Act
    let _ = backend.maybe_execute(command, &Bm1366ValidJobIds::single(job_id));

    // Assert
    assert_eq!(send_count.get(), 1);
}

#[test]
fn read_production_result_uses_bounded_read_path() {
    // Arrange
    let read_count = Rc::new(Cell::new(0));
    let job_id = Bm1366JobId::new(0x28);
    let backend = FakeProductionBackend {
        send_count: Rc::new(Cell::new(0)),
        read_count: read_count.clone(),
        maybe_result: Some(Bm1366NonceResult {
            job_id,
            nonce: 0x0102_0304,
            asic_index: 0,
            core_id: 0,
            small_core_id: 0,
            version_bits: 0,
        }),
    };

    // Act
    let result = backend
        .maybe_execute(
            Bm1366ProductionCommand::ReadProductionResult,
            &Bm1366ValidJobIds::single(job_id),
        )
        .expect("read should succeed");

    // Assert
    assert_eq!(read_count.get(), 1);
    assert!(result.is_some());
}

#[test]
fn production_executor_module_never_references_diagnostic_work() {
    // Arrange
    let source = include_str!("../production.rs");

    // Assert
    assert!(!source.contains("SendDiagnosticWork"));
}

#[test]
fn apply_negotiated_version_mask_encodes_set_version_mask_frame() {
    // Arrange — encode path used by apply_negotiated_version_mask (no UART).
    let mask = VersionMask::new(0x1fff_e000);

    // Act
    let frame = Bm1366Command::SetVersionMask(mask)
        .frame_bytes()
        .expect("SetVersionMask should encode");

    // Assert — non-empty command frame; helper returns false without UART.
    assert!(!frame.as_ref().is_empty());
    assert!(!apply_negotiated_version_mask(mask));
}

#[test]
fn accepted_state_safe_read_set_is_exact() {
    // Arrange / Act
    let registers = ACCEPTED_STATE_READ_REGISTERS;

    // Assert
    assert_eq!(registers, [0x00, 0x4c, 0x88, 0x89, 0x8a, 0x8b, 0x8c]);
}

#[test]
fn accepted_state_marker_contains_categories_only() {
    // Arrange
    let mut observation = AcceptedStateSnapshotObservation::new(AcceptedStateStage::PostFirstWork);
    observation.observe(Bm1366RegisterRead {
        register: Bm1366Register::TotalCount,
        asic_index: 7,
        asic_address: 8,
        value: 9,
    });

    // Act
    let marker = observation
        .snapshot(PowerDeltaClass::RisingHashing, false, false)
        .marker();

    // Assert
    assert_eq!(marker, "accepted_state_snapshot stage=post_first_work observation=unavailable chip_count_class=unavailable readable_responses=1 error_counter_active=false domain_counter_active=false total_counter_active=true power_delta_class=rising_hashing result_correlated=false submit_observed=false redacted=true");
    assert!(!marker.contains("asic_address"));
    assert!(!marker.contains("value"));
    assert!(!marker.contains("=7"));
    assert!(!marker.contains("=8"));
    assert!(!marker.contains("=9"));
}

fn sample_fields() -> Bm1366WorkFields {
    Bm1366WorkFields {
        starting_nonce: [0; 4],
        nbits: [1, 2, 3, 4],
        ntime: [5, 6, 7, 8],
        merkle_root: [9; 32],
        prev_block_hash: [10; 32],
        version: [11, 12, 13, 14],
    }
}
