use anyhow::{anyhow, Context};
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use esp_idf_svc::sys;
use serde_json::json;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

/// Exercises actual target memory, FreeRTOS pthread tasks and NVS independently of model budgets.
pub fn run() -> anyhow::Result<()> {
    let partition = EspDefaultNvsPartition::take().context("virtual_nvs_partition")?;
    let storage = EspNvs::new(partition, "virtual_u205", true)?;
    let boot = storage
        .get_u32("boot")?
        .unwrap_or(0)
        .checked_add(1)
        .context("boot_overflow")?;
    storage.set_u32("boot", boot)?;
    emit(
        json!({"schema":"bitaxe-virtual-guest-v1","event":"boot","execution_profile":"virtual-ultra205",
        "adc_calibration":"synthetic_uncalibrated_offset_2048","boot":boot,"compiled_source_sha256":env!("BITAXE_VIRTUAL_SOURCE_DIGEST"),"model":bitaxe_virtual_board::MODEL_VERSION,
        "heartbeat_cutoff_ms":bitaxe_runtime::revocation::HEARTBEAT_CUTOFF_MS}),
    )?;
    let joined = std::thread::Builder::new()
        .name("virtual-probe".into())
        .stack_size(8192)
        .spawn(|| unsafe {
            (
                i32::from(esp_idf_svc::hal::cpu::core()),
                sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()),
            )
        })
        .context("task_spawn")?
        .join()
        .map_err(|_| anyhow!("task_join"))?;
    emit(
        json!({"event":"task","core":joined.0,"minimum_stack_free_bytes":joined.1,"stack_bytes":8192,"joined":true}),
    )?;
    bitaxe_simulation::set_handshake_stack_observer(record_handshake_stack);
    let installed = unsafe { sys::uart_driver_install(0, 4096, 0, 0, std::ptr::null_mut(), 0) };
    if installed != sys::ESP_OK {
        return Err(anyhow!("virtual_uart_driver_install"));
    }
    unsafe { sys::uart_vfs_dev_use_driver(0) };
    let input = std::io::stdin();
    let mut reader = input.lock();
    loop {
        let value = read_command(&mut reader)?;
        match value.get("command").and_then(serde_json::Value::as_str) {
            Some("scenario") => {
                let name = value
                    .get("scenario")
                    .and_then(serde_json::Value::as_str)
                    .context("scenario_name")?;
                let seed = value
                    .get("seed")
                    .and_then(serde_json::Value::as_u64)
                    .context("scenario_seed")?;
                let result = if value
                    .get("diagnose_heap")
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
                {
                    bitaxe_simulation::run_scenario_with_observer(name, seed, &mut |phase| {
                        crate::checkpoint::bitaxe_virtual_heap_checkpoint(phase as u32);
                    })
                } else {
                    bitaxe_simulation::run_scenario(name, seed)
                }
                .map_err(|error| anyhow!("scenario_execution: {error}"))?;
                emit(json!({"event":"scenario","result":result}))?;
                let margin = unsafe { sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };
                let handshake = HANDSHAKE_STACK_FREE.load(Ordering::Acquire);
                let heap_integrity = unsafe { sys::heap_caps_check_integrity_all(false) };
                emit(
                    json!({"event":"scenario_margin","minimum_main_stack_free_bytes":margin,
                    "required_margin_bytes":2048,"configured_main_stack_bytes":16384,
                    "handshake_minimum_stack_free_bytes":(handshake != u32::MAX).then_some(handshake),
                    "handshake_configured_stack_bytes":bitaxe_simulation::HANDSHAKE_STACK_BYTES,
                    "heap_integrity":heap_integrity}),
                )?;
            }
            Some("noise_prefix") => crate::noise_probe::run_prefix_and_emit(&value)?,
            Some("noise_probe") => crate::noise_probe::run_and_emit(&value)?,
            Some("status") => emit(json!({"event":"status","boot":boot,
                "internal_free": unsafe { sys::heap_caps_get_free_size(sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT) },
                "internal_largest": unsafe { sys::heap_caps_get_largest_free_block(sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT) },
                "psram_free": unsafe { sys::heap_caps_get_free_size(sys::MALLOC_CAP_SPIRAM | sys::MALLOC_CAP_8BIT) }}))?,
            Some("allocation") => allocation_probe()?,
            Some("restart") => {
                emit(json!({"event":"restart","boot":boot}))?;
                std::thread::sleep(Duration::from_millis(50));
                unsafe { sys::esp_restart() }
            }
            Some("panic") => {
                emit(json!({"event":"panic_requested","boot":boot}))?;
                unsafe { sys::abort() }
            }
            _ => emit(json!({"event":"unsupported","category":"virtual_command_unsupported"}))?,
        }
    }
}

static HANDSHAKE_STACK_FREE: AtomicU32 = AtomicU32::new(u32::MAX);

/// Runs on the handshake helper just before it exits; keeps the lowest high-water mark.
fn record_handshake_stack() {
    let free = unsafe { sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };
    // Helpers are joined one at a time, so a load and store keep the minimum. This
    // also avoids `fetch_min`, which the pinned Xtensa backend fails to assemble.
    if free < HANDSHAKE_STACK_FREE.load(Ordering::Acquire) {
        HANDSHAKE_STACK_FREE.store(free, Ordering::Release);
    }
}

fn allocation_probe() -> anyhow::Result<()> {
    let caps = sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT;
    let before = unsafe { sys::heap_caps_get_free_size(caps) };
    let memory = unsafe { sys::heap_caps_malloc(8192, caps) };
    if memory.is_null() {
        return Err(anyhow!("target_internal_allocation_rejected"));
    }
    let during = unsafe { sys::heap_caps_get_free_size(caps) };
    unsafe { sys::heap_caps_free(memory) };
    let after = unsafe { sys::heap_caps_get_free_size(caps) };
    emit(
        json!({"event":"allocation","capability":"internal_8bit","bytes":8192,
        "before":before,"during":during,"after":after,"released":after == before}),
    )
}

pub(super) fn emit(value: serde_json::Value) -> anyhow::Result<()> {
    let mut output = std::io::stdout().lock();
    writeln!(output, "VIRTUAL_U205 {}", serde_json::to_string(&value)?)?;
    output.flush()?;
    Ok(())
}

/// The pinned QEMU SAR ADC cannot complete the SDK constructor's ground conversion.
/// This explicit guest-only I/O adapter supplies an uncalibrated functional offset;
/// physical ADC calibration remains unsupported and no production symbol is changed.
#[no_mangle]
pub extern "C" fn __wrap_adc_hal_self_calibration(
    _unit: u32,
    _attenuation: u32,
    _internal_ground: bool,
) -> u32 {
    2048
}

// Release the bounded receive frame before scenario/control serialization begins.
#[inline(never)]
fn read_command(reader: &mut impl Read) -> anyhow::Result<serde_json::Value> {
    let mut buffer = [0_u8; 4096];
    let mut length = 0;
    loop {
        let mut byte = [0_u8; 1];
        if reader.read(&mut byte)? == 0 {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        if byte[0] == b'\r' {
            continue;
        }
        if byte[0] == b'\n' {
            if length == 0 {
                continue;
            }
            return Ok(serde_json::from_slice(&buffer[..length])?);
        }
        if length == buffer.len() {
            return Err(anyhow!("virtual_request_bound"));
        }
        buffer[length] = byte[0];
        length += 1;
    }
}
