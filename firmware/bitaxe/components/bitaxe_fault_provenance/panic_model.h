#pragma once
#include <stdbool.h>
#include <stdint.h>
#include "include/bitaxe_fault_provenance.h"
/* Scalar-only helpers shared by native capture and host tests. */
static inline __attribute__((always_inline)) bool bitaxe_readable_span(uint32_t address, uint32_t length, uint32_t low, uint32_t high)
{
    return length != 0 && high > low && length <= high - low &&
           address >= low && address <= high - length && (address & 3u) == 0;
}
static inline __attribute__((always_inline)) uint32_t bitaxe_stack_rejections(uint32_t start, uint32_t end,
                                                                              bool start_sane, bool end_sane, uint32_t maximum)
{
    return (!start_sane ? 1u : 0u) | (!end_sane ? 2u : 0u) |
           (start >= end ? 4u : 0u) | ((uint32_t)(end - start) > maximum ? 8u : 0u);
}
static inline __attribute__((always_inline)) bool bitaxe_snapshot_matches(uint32_t requested, uint32_t observed, bool readable)
{
    return readable && requested != 0 && requested == observed;
}
static inline __attribute__((always_inline)) uint32_t bitaxe_panic_checksum(const volatile uint32_t *words)
{
    uint32_t hash = UINT32_C(2166136261);
    for (unsigned i = 1; i <= 45; ++i) {
        uint32_t word = words[i];
        for (unsigned shift = 0; shift < 32; shift += 8) {
            hash = (hash ^ ((word >> shift) & 255u)) * UINT32_C(16777619);
        }
    }
    return hash;
}
static inline __attribute__((always_inline)) bool bitaxe_panic_record_valid(const volatile uint32_t *words)
{
    return words[0] == BITAXE_PANIC_FRAME_MAGIC && words[1] == 1 && words[2] == BITAXE_PANIC_FRAME_WORDS &&
           words[46] == bitaxe_panic_checksum(words) && words[47] == ~words[46];
}
