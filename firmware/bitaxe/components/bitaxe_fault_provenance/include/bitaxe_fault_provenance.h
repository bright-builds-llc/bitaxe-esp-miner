#pragma once
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
#define BITAXE_PANIC_FRAME_WORDS 48u
#define BITAXE_PANIC_FRAME_MAGIC UINT32_C(0x50465231)
#define BITAXE_FAULT_ABI_WORDS 32u
#define BITAXE_FAULT_ABI_MAGIC UINT32_C(0x50464131)
/* Private numeric memory evidence; never format these records on the controller transport. */
extern volatile uint32_t BITAXE_PANIC_FRAME_RECORD[BITAXE_PANIC_FRAME_WORDS];
extern const uint32_t BITAXE_FAULT_PROVENANCE_ABI[BITAXE_FAULT_ABI_WORDS];
void bitaxe_fault_set_identity(uint32_t source_lo, uint32_t source_hi, uint32_t boot_lo, uint32_t boot_hi);
/* Called only after the native cutoff/revocation receipt has committed. */
void bitaxe_capture_original_panic(const void *info);
void bitaxe_allocation_set_identity(uint32_t source_lo, uint32_t source_hi, uint32_t boot_lo, uint32_t boot_hi);
void bitaxe_allocation_set_legacy(void *allocation, void *context, uint32_t source_lo, uint32_t source_hi);
void bitaxe_allocation_set_stage(uint32_t stage);
void bitaxe_allocation_failure_record(size_t size, uint32_t caps, const char *function_name);
void bitaxe_fault_owner_begin(void);
void bitaxe_fault_owner_end(void);
void bitaxe_fault_command_begin(void);
void bitaxe_fault_enter_phase(uint32_t phase);
#ifdef __cplusplus
}
#endif
