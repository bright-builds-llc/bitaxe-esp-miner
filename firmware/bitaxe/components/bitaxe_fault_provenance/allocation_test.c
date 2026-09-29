// SPDX-License-Identifier: MIT
// Host tests execute the production recorder; only CPU/task registers are simulated.
#define BITAXE_FAULT_HOST_TEST
#include <stdint.h>
#include <assert.h>
#include <string.h>
#include <stdio.h>
static uint32_t test_core, test_task, test_cycles, test_isr;
#include "allocation.c"
uint32_t fault_test_core(void) { return test_core; }
uint32_t fault_test_task(void) { return test_task; }
uint32_t fault_test_cycles(void) { return test_cycles; }
uint32_t fault_test_isr(void) { return test_isr; }
static uint32_t allocation[4], context[8];
static void reset(void) {
    memset((void *)BITAXE_ALLOCATION_HISTORY, 0, sizeof(BITAXE_ALLOCATION_HISTORY));
    memset(owners, 0, sizeof(owners)); memset(allocation, 0, sizeof(allocation)); memset(context, 0, sizeof(context));
    test_core = 0; test_task = 0x123400; test_cycles = 10; test_isr = 0;
    bitaxe_allocation_set_legacy(allocation, context, 0x12345678, 0x90abcdef);
    bitaxe_allocation_set_identity(0x12345678, 0x90abcdef, 23, 0);
    bitaxe_allocation_set_stage(8);
}
static volatile uint32_t *first(uint32_t core) { return BITAXE_ALLOCATION_HISTORY + HEADER_WORDS + core * CORE_WORDS + 4; }
static volatile uint32_t *recent(uint32_t core, uint32_t sequence) { return first(core) + RECORD_WORDS + ((sequence - 1) & 7) * RECORD_WORDS; }
static bool valid(const volatile uint32_t *record) {
    return record[0] != 0 && record[0] == record[2] && record[1] == 0x414c4c31 && record[19] == checksum(record, 1, 19);
}
static void first_and_latest_are_per_core(void) {
    // Arrange
    reset();
    // Act
    for (uint32_t core = 0; core < 2; ++core) {
        test_core = core;
        for (uint32_t i = 1; i <= 12; ++i) bitaxe_allocation_failure_record(i * 1024, 0x804, (const char *)(uintptr_t)0x3c001000);
    }
    // Assert
    for (uint32_t core = 0; core < 2; ++core) {
        assert(valid(first(core))); assert(first(core)[2] == 1); assert(first(core)[9] == 1024);
        for (uint32_t i = 5; i <= 12; ++i) { assert(valid(recent(core, i))); assert(recent(core, i)[2] == i); }
        assert(BITAXE_ALLOCATION_HISTORY[HEADER_WORDS + core * CORE_WORDS + 2] == 4);
    }
    assert(allocation[1] == 1024); // Legacy receipt remains global first failure.
}
static void owner_phase_never_leaks_to_other_tasks_or_isr(void) {
    // Arrange
    reset(); bitaxe_fault_owner_begin(); bitaxe_fault_command_begin(); bitaxe_fault_enter_phase(5);
    // Act / Assert
    bitaxe_allocation_failure_record(8192, 0x804, NULL);
    assert(first(0)[14] == 1 && first(0)[15] == 5 && first(0)[16] == 1 && first(0)[17] == 0x40001000 && first(0)[18] == 1);
    test_task++; bitaxe_allocation_failure_record(8192, 0x804, NULL);
    assert(recent(0, 2)[14] == 0 && recent(0, 2)[15] == 0 && recent(0, 2)[18] == 0);
    test_task--; test_isr = 1; bitaxe_allocation_failure_record(8192, 0x804, NULL);
    assert(recent(0, 3)[14] == 0 && recent(0, 3)[18] == 8);
    test_isr = 0; bitaxe_fault_owner_end(); bitaxe_allocation_failure_record(8192, 0x804, NULL);
    assert(recent(0, 4)[14] == 0);
}
static void wraps_torn_writes_and_saturation_are_explicit(void) {
    // Arrange
    reset(); test_cycles = UINT32_MAX;
    // Act
    bitaxe_allocation_failure_record(8192, 0x804, NULL); test_cycles = 2;
    bitaxe_allocation_failure_record(8192, 0x804, NULL);
    // Assert: cycles wrap, event order does not.
    assert(first(0)[12] == UINT32_MAX && recent(0, 2)[12] == 2 && recent(0, 2)[2] == 2);
    recent(0, 2)[0] = 0; assert(!valid(recent(0, 2)));
    recent(0, 2)[0] = 2; recent(0, 2)[9] ^= 1; assert(!valid(recent(0, 2)));
    BITAXE_ALLOCATION_HISTORY[HEADER_WORDS] = UINT32_MAX;
    bitaxe_allocation_failure_record(8192, 0x804, NULL);
    assert(BITAXE_ALLOCATION_HISTORY[HEADER_WORDS] == UINT32_MAX && BITAXE_ALLOCATION_HISTORY[HEADER_WORDS + 1] == 1);
}
static void migration_and_registry_exhaustion_do_not_invent_owners(void) {
    // Arrange
    reset(); bitaxe_fault_owner_begin(); bitaxe_fault_command_begin(); bitaxe_fault_enter_phase(6);
    // Act: the same registered task migrates; a different unregistered task never inherits it.
    test_core = 1; bitaxe_fault_command_begin(); bitaxe_fault_enter_phase(7);
    bitaxe_allocation_failure_record(4096, 0x804, NULL);
    // Assert
    assert(first(1)[14] == 1 && first(1)[15] == 7 && first(1)[16] == 2);
    for (uint32_t i = 1; i <= OWNER_SLOTS; ++i) { test_task = 0x123400 + i; bitaxe_fault_owner_begin(); }
    bitaxe_allocation_failure_record(4096, 0x804, NULL);
    assert(BITAXE_ALLOCATION_HISTORY[HEADER_WORDS + CORE_WORDS + 3] == 1);
    assert(recent(1, 2)[14] == 0 && recent(1, 2)[15] == 0);
    test_task = 0x123400; owners[0].command = UINT32_MAX - 1;
    bitaxe_fault_command_begin(); bitaxe_fault_command_begin(); bitaxe_allocation_failure_record(4096, 0x804, NULL);
    assert(recent(1, 3)[16] == UINT32_MAX && (recent(1, 3)[18] & 4) != 0);
}
static uint64_t rol64(uint64_t value, uint32_t shift) { return (value << shift) | (value >> (64 - shift)); }
static void identity_and_legacy_abi_are_exact(void) {
    // Arrange
    reset();
    // Act
    bitaxe_allocation_failure_record(8192, 0x804, NULL);
    // Assert
    assert(BITAXE_ALLOCATION_HISTORY[15] == checksum(BITAXE_ALLOCATION_HISTORY, 0, 11));
    assert(first(0)[4] == 0x12345678 && first(0)[5] == 0x90abcdef && first(0)[6] == 23 && first(0)[8] == 8);
    assert(allocation[0] == 0x42584146 && allocation[3] == (rotate(0x42584146, 7) ^ rotate(8192, 17) ^ rotate(0x804, 23)));
    uint64_t expected = rol64(UINT64_C(0x90abcdef12345678), 11) ^ rol64(0x42584143, 7) ^ rol64(8192, 23) ^ rol64(0x804, 39) ^ rol64(8, 53);
    assert(context[2] == 0x42584143 && context[6] == (uint32_t)expected && context[7] == (uint32_t)(expected >> 32));
    bitaxe_allocation_set_identity(0x12345678, 0x90abcdef, 0, 0); bitaxe_allocation_failure_record(1, 4, NULL);
    assert((recent(0, 2)[18] & 2) != 0); // Unbound early boot is not attributed later.
}
int main(void) {
    first_and_latest_are_per_core(); owner_phase_never_leaks_to_other_tasks_or_isr();
    wraps_torn_writes_and_saturation_are_explicit(); migration_and_registry_exhaustion_do_not_invent_owners(); identity_and_legacy_abi_are_exact();
    puts("allocation_provenance_host_passed"); return 0;
}
