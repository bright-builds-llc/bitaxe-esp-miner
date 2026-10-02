//! Allocation-free private Noise checkpoints; result formatting happens after release.
use crate::checkpoint::HeapCheckpoint;
use bitaxe_simulation::noise_probe::{NoiseFault, NoisePhase, NoiseProbeResult};
use esp_idf_svc::sys;
use std::sync::atomic::{AtomicBool, Ordering};

const CONFIGURED_STACK_BYTES: u32 = 16384;
const EMPTY: HeapCheckpoint = HeapCheckpoint {
    magic: 0,
    phase: 0,
    stage: 0,
    integrity: 0,
    core: 0,
    stack_base: 0,
    stack_bytes: 0,
    stack_pointer: 0,
    internal_free: 0,
    internal_largest: 0,
    stack_low_water: 0,
    stack_pointer_inside: 0,
};
static USED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
#[used]
#[link_section = ".dram1.coredump"]
pub static mut BITAXE_VIRTUAL_NOISE_CHECKPOINTS: [HeapCheckpoint; 16] = [EMPTY; 16];
#[no_mangle]
#[used]
#[link_section = ".dram1.coredump"]
pub static mut BITAXE_VIRTUAL_NOISE_CHECKPOINT_COUNT: u32 = 0;

/// One fresh process, one bounded probe. The span is configured, not allocator-derived.
pub fn run(value: &serde_json::Value) -> anyhow::Result<NoiseProbeResult> {
    let seed = value
        .get("seed")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow::anyhow!("noise_probe_seed"))?;
    if seed != 1 {
        return Err(anyhow::anyhow!("noise_probe_seed_bound"));
    }
    let mode = value
        .get("mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("noise_probe_mode"))?;
    let fault = match mode {
        "valid" => NoiseFault::None,
        "wrong_authority" => NoiseFault::WrongAuthority,
        "tampered_act_two" => NoiseFault::TamperedActTwo,
        "truncated_act_two" => NoiseFault::TruncatedActTwo,
        "tampered_frame" => NoiseFault::TamperedFrame,
        "truncated_frame" => NoiseFault::TruncatedFrame,
        _ => return Err(anyhow::anyhow!("noise_probe_mode_unsupported")),
    };
    if USED.swap(true, Ordering::AcqRel) {
        return Err(anyhow::anyhow!("noise_probe_reused"));
    }
    bitaxe_simulation::noise_probe::run(seed, fault, &mut |phase| {
        bitaxe_virtual_noise_checkpoint(phase as u32);
    })
    .map_err(anyhow::Error::new)
}

/// Preserve stack facts before heap checks that may fault on existing corruption.
#[no_mangle]
#[inline(never)]
pub extern "C" fn bitaxe_virtual_noise_checkpoint(phase: u32) {
    unsafe {
        let index =
            std::ptr::read_volatile(std::ptr::addr_of!(BITAXE_VIRTUAL_NOISE_CHECKPOINT_COUNT));
        if index >= 16 || !(NoisePhase::Entry as u32..=NoisePhase::Released as u32).contains(&phase)
        {
            return;
        }
        let destination = std::ptr::addr_of_mut!(BITAXE_VIRTUAL_NOISE_CHECKPOINTS)
            .cast::<HeapCheckpoint>()
            .add(index as usize);
        let mut record = EMPTY;
        record.magic = 0x564e5031;
        record.phase = phase;
        record.stage = 1;
        let task = sys::xTaskGetCurrentTaskHandle();
        record.stack_base = sys::pxTaskGetStackStart(task) as u32;
        record.stack_bytes = CONFIGURED_STACK_BYTES;
        core::arch::asm!("mov {0}, a1", out(reg) record.stack_pointer);
        record.core = i32::from(esp_idf_svc::hal::cpu::core()) as u32;
        record.stack_low_water = sys::uxTaskGetStackHighWaterMark(task);
        record.stack_pointer_inside = u32::from(
            record.stack_pointer >= record.stack_base
                && record.stack_pointer < record.stack_base.saturating_add(record.stack_bytes),
        );
        std::ptr::write_volatile(destination, record);
        std::ptr::write_volatile(
            std::ptr::addr_of_mut!(BITAXE_VIRTUAL_NOISE_CHECKPOINT_COUNT),
            index + 1,
        );
        record.integrity = u32::from(sys::heap_caps_check_integrity_all(false));
        record.internal_free =
            sys::heap_caps_get_free_size(sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT) as u32;
        record.internal_largest =
            sys::heap_caps_get_largest_free_block(sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT)
                as u32;
        record.stage = 2;
        std::sync::atomic::compiler_fence(Ordering::Release);
        std::ptr::write_volatile(destination, record);
        bitaxe_virtual_noise_checkpoint_complete(phase);
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn bitaxe_virtual_noise_checkpoint_complete(phase: u32) {
    std::hint::black_box(phase);
}

/// Keep post-probe serialization scratch out of the live cryptographic caller frame.
#[inline(never)]
pub fn run_and_emit(value: &serde_json::Value) -> anyhow::Result<()> {
    let outcome = run(value);
    emit_outcome(outcome)
}

#[inline(never)]
fn emit_outcome(outcome: anyhow::Result<NoiseProbeResult>) -> anyhow::Result<()> {
    let margin = unsafe { sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };
    let count = unsafe {
        std::ptr::read_volatile(std::ptr::addr_of!(BITAXE_VIRTUAL_NOISE_CHECKPOINT_COUNT))
    };
    if count > 16 {
        return Err(anyhow::anyhow!("noise_checkpoint_count"));
    }
    let mut checkpoints = Vec::new();
    for index in 0..count {
        let record = unsafe {
            std::ptr::read_volatile(
                std::ptr::addr_of!(BITAXE_VIRTUAL_NOISE_CHECKPOINTS)
                    .cast::<HeapCheckpoint>()
                    .add(index as usize),
            )
        };
        if record.magic != 0x564e5031
            || ![1, 2].contains(&record.stage)
            || !(101..=114).contains(&record.phase)
        {
            return Err(anyhow::anyhow!("noise_checkpoint_format"));
        }
        let complete = record.stage == 2;
        checkpoints.push(serde_json::json!({"phase":record.phase,"stage":record.stage,
            "integrity":if complete { Some(record.integrity == 1) } else { None },
            "stack_low_water":record.stack_low_water,"stack_pointer_inside":record.stack_pointer_inside == 1,
            "stack_span_kind":"configured","configured_main_stack_bytes":record.stack_bytes,
            "internal_free":if complete { Some(record.internal_free) } else { None },
            "internal_largest":if complete { Some(record.internal_largest) } else { None }}));
    }
    let mut event = serde_json::json!({"minimum_main_stack_free_bytes":margin,"required_margin_bytes":2048,
        "configured_main_stack_bytes":16384,"checkpoint_stack_span":"configured","checkpoints":checkpoints});
    match outcome {
        Ok(result) => {
            event["event"] = "noise_probe".into();
            event["result"] = serde_json::to_value(result)?;
        }
        Err(error) => {
            event["event"] = "noise_probe_rejected".into();
            event["category"] = error.to_string().into();
        }
    }
    crate::guest::emit(event)
}
