// SPDX-License-Identifier: MIT
// Failure-only provenance. No allocator interposition, stack walk, strings, or logs.
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <limits.h>
#ifdef BITAXE_FAULT_HOST_TEST
#define IRAM_ATTR
#define COREDUMP_DRAM_ATTR
#define DRAM_ATTR
extern uint32_t fault_test_core(void);
extern uint32_t fault_test_task(void);
extern uint32_t fault_test_cycles(void);
extern uint32_t fault_test_isr(void);
#define LOCAL_MASK() 0U
#define LOCAL_RESTORE(mask) ((void)(mask))
#define CORE() fault_test_core()
#define TASK() fault_test_task()
#define CYCLES() fault_test_cycles()
#define IN_ISR() fault_test_isr()
#define BARRIER() __atomic_thread_fence(__ATOMIC_SEQ_CST)
#define PHASE_SITE() 0x40001000U
#else
#include "bitaxe_fault_provenance.h"
#include "esp_attr.h"
#include "esp_core_dump.h"
#include "esp_cpu.h"
#include "esp_cpu_utils.h"
#include "xt_utils.h"
#if !XCHAL_HAVE_S32C1I
#error "Allocation provenance requires native internal-DRAM S32C1I"
#endif
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#define LOCAL_MASK() portSET_INTERRUPT_MASK_FROM_ISR()
#define LOCAL_RESTORE(mask) portCLEAR_INTERRUPT_MASK_FROM_ISR(mask)
#define CORE() ((uint32_t)esp_cpu_get_core_id())
#define TASK() ((uint32_t)(uintptr_t)xTaskGetCurrentTaskHandle())
#define CYCLES() esp_cpu_get_cycle_count()
#define IN_ISR() ((uint32_t)xPortInIsrContext())
#define BARRIER() __asm__ volatile("memw" ::: "memory")
#define PHASE_SITE() esp_cpu_process_stack_pc((uint32_t)(uintptr_t)__builtin_return_address(0))
#endif

#define HEADER_WORDS 16U
#define RECORD_WORDS 20U
#define CORE_WORDS 184U
#define CORE_COUNT 2U
#define SLOTS 8U
#define OWNER_SLOTS 8U
#define READY 0xa110ca7eU
COREDUMP_DRAM_ATTR volatile uint32_t BITAXE_ALLOCATION_HISTORY[384];
static DRAM_ATTR volatile uint32_t *legacy_allocation;
static DRAM_ATTR volatile uint32_t *legacy_context;
static DRAM_ATTR uint32_t legacy_source_lo, legacy_source_hi;
static DRAM_ATTR uint32_t legacy_claim;
struct owner { uint32_t task, tag, command, phase, site; };
static DRAM_ATTR struct owner owners[OWNER_SLOTS];
_Static_assert(_Alignof(struct owner) >= 4, "Owner CAS alignment");

static inline __attribute__((always_inline)) uint32_t rotate(uint32_t value, uint32_t shift) {
    return (value << shift) | (value >> (32U - shift));
}
static uint32_t IRAM_ATTR checksum(const volatile uint32_t *words, uint32_t start, uint32_t end) {
    uint32_t value = 0x6d5a56a9U;
    for (uint32_t i = start; i < end; ++i) value = rotate(value, 5) ^ words[i];
    return value;
}
// Internal DRAM only. GCC's generic atomic CAS delegates to a FreeRTOS lock.
static inline __attribute__((always_inline)) bool claim_once(uint32_t *address, uint32_t expected, uint32_t desired) {
#ifdef BITAXE_FAULT_HOST_TEST
    return __atomic_compare_exchange_n(address, &expected, desired, false, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE);
#else
    uint32_t mask = LOCAL_MASK();
    BARRIER();
    bool matched = xt_utils_compare_and_set(address, expected, desired);
    BARRIER();
    LOCAL_RESTORE(mask);
    return matched;
#endif
}
static struct owner *IRAM_ATTR current_owner(uint32_t task) {
    if (task == 0) return NULL;
    for (uint32_t i = 0; i < OWNER_SLOTS; ++i)
        if (__atomic_load_n(&owners[i].task, __ATOMIC_ACQUIRE) == task) return &owners[i];
    return NULL;
}

void bitaxe_allocation_set_legacy(void *allocation, void *context, uint32_t source_lo, uint32_t source_hi) {
    // Startup-only; these private DRAM records are intentionally new each boot.
    for (uint32_t i = 0; i < 384; ++i) BITAXE_ALLOCATION_HISTORY[i] = 0;
    for (uint32_t i = 0; i < OWNER_SLOTS; ++i) {
        owners[i].task = 0; owners[i].tag = 0; owners[i].command = 0; owners[i].phase = 0; owners[i].site = 0;
    }
    legacy_allocation = allocation;
    legacy_context = context;
    legacy_source_lo = source_lo;
    legacy_source_hi = source_hi;
    legacy_claim = 0;
    BITAXE_ALLOCATION_HISTORY[0] = 0x42414c48U;
    BITAXE_ALLOCATION_HISTORY[1] = 1;
    BITAXE_ALLOCATION_HISTORY[2] = HEADER_WORDS;
    BITAXE_ALLOCATION_HISTORY[3] = RECORD_WORDS;
    BITAXE_ALLOCATION_HISTORY[4] = CORE_WORDS;
    BITAXE_ALLOCATION_HISTORY[5] = CORE_COUNT;
    BITAXE_ALLOCATION_HISTORY[6] = SLOTS;
    BITAXE_ALLOCATION_HISTORY[7] = source_lo;
    BITAXE_ALLOCATION_HISTORY[8] = source_hi;
    BITAXE_ALLOCATION_HISTORY[11] = 1;
    BITAXE_ALLOCATION_HISTORY[15] = checksum(BITAXE_ALLOCATION_HISTORY, 0, 11);
    BARRIER(); BITAXE_ALLOCATION_HISTORY[12] = READY;
}
void bitaxe_allocation_set_identity(uint32_t source_lo, uint32_t source_hi, uint32_t boot_lo, uint32_t boot_hi) {
    // Startup-only before other owners exist. Earlier entries retain explicit boot=0.
    BITAXE_ALLOCATION_HISTORY[12] = 0;
    BITAXE_ALLOCATION_HISTORY[7] = source_lo; BITAXE_ALLOCATION_HISTORY[8] = source_hi;
    BITAXE_ALLOCATION_HISTORY[9] = boot_lo; BITAXE_ALLOCATION_HISTORY[10] = boot_hi;
    BITAXE_ALLOCATION_HISTORY[15] = checksum(BITAXE_ALLOCATION_HISTORY, 0, 11);
    BARRIER(); BITAXE_ALLOCATION_HISTORY[12] = READY;
}
void IRAM_ATTR bitaxe_allocation_set_stage(uint32_t stage) {
    if (stage >= 1 && stage <= 8) BITAXE_ALLOCATION_HISTORY[11] = stage;
}
void IRAM_ATTR bitaxe_fault_owner_begin(void) {
    uint32_t mask = LOCAL_MASK(), task = TASK();
    if (task) {
        struct owner *found = current_owner(task);
        for (uint32_t i = 0; !found && i < OWNER_SLOTS; ++i) {
            if (claim_once(&owners[i].task, 0, task)) found = &owners[i];
        }
        if (found) { found->tag = 1; found->command = 0; found->phase = 0; found->site = 0; }
        else { uint32_t core = CORE(); if (core < CORE_COUNT) BITAXE_ALLOCATION_HISTORY[HEADER_WORDS + core * CORE_WORDS + 3] = 1; }
    }
    LOCAL_RESTORE(mask);
}
void IRAM_ATTR bitaxe_fault_owner_end(void) {
    uint32_t mask = LOCAL_MASK(); struct owner *found = current_owner(TASK());
    if (found) { found->tag = 0; found->phase = 0; found->site = 0; __atomic_store_n(&found->task, 0, __ATOMIC_RELEASE); }
    LOCAL_RESTORE(mask);
}
void IRAM_ATTR bitaxe_fault_command_begin(void) {
    uint32_t site = PHASE_SITE(), mask = LOCAL_MASK(); struct owner *found = current_owner(TASK());
    if (found) { if (found->command != UINT32_MAX) ++found->command; found->phase = 1; found->site = site; }
    LOCAL_RESTORE(mask);
}
void IRAM_ATTR bitaxe_fault_enter_phase(uint32_t phase) {
    uint32_t site = PHASE_SITE(), mask = LOCAL_MASK(); struct owner *found = current_owner(TASK());
    if (found && phase <= 9) { found->phase = phase; found->site = site; }
    LOCAL_RESTORE(mask);
}
static void IRAM_ATTR commit_record(volatile uint32_t *destination, const uint32_t *record) {
    destination[0] = 0; BARRIER();
    for (uint32_t i = 1; i < RECORD_WORDS; ++i) destination[i] = record[i];
    BARRIER(); destination[0] = record[2];
}
static void IRAM_ATTR record_history(uint32_t size, uint32_t caps, const char *name) {
    uint32_t mask = LOCAL_MASK(), core = CORE(), task = TASK();
    if (core < CORE_COUNT && BITAXE_ALLOCATION_HISTORY[12] == READY) {
        volatile uint32_t *store = &BITAXE_ALLOCATION_HISTORY[HEADER_WORDS + core * CORE_WORDS];
        if (store[0] == UINT32_MAX) store[1] |= 1;
        else {
            uint32_t sequence = store[0] + 1, record[RECORD_WORDS];
            uint32_t in_isr = IN_ISR();
            struct owner *owner = in_isr ? NULL : current_owner(task);
            record[0] = 0; record[1] = 0x414c4c31U; record[2] = sequence; record[3] = core;
            record[4] = BITAXE_ALLOCATION_HISTORY[7]; record[5] = BITAXE_ALLOCATION_HISTORY[8];
            record[6] = BITAXE_ALLOCATION_HISTORY[9]; record[7] = BITAXE_ALLOCATION_HISTORY[10];
            record[8] = BITAXE_ALLOCATION_HISTORY[11]; record[9] = size; record[10] = caps;
            record[11] = task; record[12] = CYCLES(); record[13] = (uint32_t)(uintptr_t)name;
            record[14] = owner ? owner->tag : 0; record[15] = owner ? owner->phase : 0;
            record[16] = owner ? owner->command : 0; record[17] = owner ? owner->site : 0;
            record[18] = (owner ? 1U : 0U) | ((!record[6] && !record[7]) ? 2U : 0U) |
                ((owner && owner->command == UINT32_MAX) ? 4U : 0U) | (in_isr ? 8U : 0U);
            record[19] = checksum(record, 1, 19);
            store[0] = sequence; store[2] = sequence > SLOTS ? sequence - SLOTS : 0;
            if (sequence == 1) commit_record(store + 4, record);
            commit_record(store + 4 + RECORD_WORDS + ((sequence - 1) & (SLOTS - 1)) * RECORD_WORDS, record);
        }
    }
    LOCAL_RESTORE(mask);
}
void IRAM_ATTR bitaxe_allocation_failure_record(size_t size, uint32_t caps, const char *name) {
#if SIZE_MAX > UINT32_MAX
    uint32_t bytes = size > UINT32_MAX ? UINT32_MAX : (uint32_t)size;
#else
    uint32_t bytes = (uint32_t)size;
#endif
    record_history(bytes, caps, name);
    if (!legacy_allocation || !legacy_context || !claim_once(&legacy_claim, 0, 1)) return;
    // Preserve the legacy first-failure RTC ABI, including its independent integrity checks.
    legacy_allocation[0] = 0; legacy_allocation[1] = bytes; legacy_allocation[2] = caps;
    legacy_allocation[3] = rotate(0x42584146U, 7) ^ rotate(bytes, 17) ^ rotate(caps, 23);
    uint32_t stage = BITAXE_ALLOCATION_HISTORY[11];
    legacy_context[0] = legacy_source_lo; legacy_context[1] = legacy_source_hi;
    legacy_context[2] = 0; legacy_context[3] = bytes; legacy_context[4] = caps; legacy_context[5] = stage;
    legacy_context[6] = (legacy_source_lo << 11 | legacy_source_hi >> 21) ^ (0x42584143U << 7) ^ (bytes << 23) ^ (caps >> 25) ^ (stage >> 11);
    legacy_context[7] = (legacy_source_hi << 11 | legacy_source_lo >> 21) ^ (0x42584143U >> 25) ^ (bytes >> 9) ^ (caps << 7) ^ (stage << 21);
    BARRIER(); legacy_context[2] = 0x42584143U; legacy_allocation[0] = 0x42584146U;
}
