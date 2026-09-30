//! Resource model checks explicitly separated from actual allocator injection.
use crate::{CheckStatus, Failure, ProfileCheck, ScenarioResult, SCENARIO_VERSION};
use bitaxe_virtual_board::memory::{AllocationError, Capability, CapabilityHeap, FailureRule};

pub(crate) fn run(name: &str, seed: u64) -> ScenarioResult {
    let mut heap = if name == "heap-fragmented" {
        CapabilityHeap::observed_pressure()
    } else {
        CapabilityHeap::healthy()
    };
    if name == "internal-allocation-failure" {
        heap.failure_rules.push(FailureRule {
            maybe_attempt: None,
            maybe_bytes: Some(8192),
            maybe_capability: Some(Capability::Internal),
            maybe_phase: Some("runtime_stack".into()),
        });
    }
    let before = heap.snapshot(Capability::Internal);
    let allocation = heap.allocate(8192, 8, Capability::Internal, "runtime_stack");
    let rejected = matches!(
        allocation,
        Err(AllocationError::Exhausted | AllocationError::Injected)
    );
    let category = match allocation {
        Err(AllocationError::Exhausted) => "allocation_exhausted",
        Err(AllocationError::Injected) => "allocation_injected",
        _ => "unexpected_allocation",
    };
    let after = heap.snapshot(Capability::Internal);
    ScenarioResult {
        schema: SCENARIO_VERSION.into(),
        scenario: name.into(),
        seed,
        checks: vec![
            ProfileCheck {
                id: "capability_budget".into(),
                status: if rejected && before == after && heap.live_allocations() == 0 {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                },
                detail: category.into(),
            },
            ProfileCheck {
                id: "actual_allocator_fault_injection".into(),
                status: CheckStatus::Unsupported,
                detail: "production_allocation_interception_not_integrated".into(),
            },
        ],
        expected_outcome: if name == "heap-fragmented" {
            "allocation_exhausted"
        } else {
            "allocation_injected"
        }
        .into(),
        actual_outcome: category.into(),
        journal: vec![],
        maybe_earliest_failure: Some(Failure {
            phase: "runtime_stack".into(),
            category: category.into(),
        }),
        cleanup_failures: vec![],
        virtual_time_ms: 0,
        modeled_memory: true,
        hardware_qualified: false,
    }
}
