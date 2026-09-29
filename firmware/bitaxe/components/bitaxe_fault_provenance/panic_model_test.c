#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "panic_model.h"

static void invalid_spans_never_require_pointer_access(void)
{
    /* Arrange: numeric addresses only, including wrap and partial final frame. */
    const uint32_t low = UINT32_C(0x3fc88000), high = UINT32_C(0x3fd00000);
    /* Act / Assert */
    assert(bitaxe_readable_span(low, 112, low, high));
    assert(bitaxe_readable_span(high - 112, 112, low, high));
    assert(!bitaxe_readable_span(0, 112, low, high));
    assert(!bitaxe_readable_span(low + 1, 112, low, high));
    assert(!bitaxe_readable_span(high - 108, 112, low, high));
    assert(!bitaxe_readable_span(UINT32_MAX - 3, 112, low, high));
    assert(!bitaxe_readable_span(low, UINT32_MAX, low, high));
}
static void stack_predicates_preserve_each_sdk_reason(void)
{
    /* Arrange */
    const uint32_t start = 0x1000, end = 0x2000, maximum = 65536;
    /* Act / Assert */
    assert(bitaxe_stack_rejections(start, end, true, true, maximum) == 0);
    assert(bitaxe_stack_rejections(start, end, false, true, maximum) == 1);
    assert(bitaxe_stack_rejections(start, end, true, false, maximum) == 2);
    assert(bitaxe_stack_rejections(start, start, true, true, maximum) == 4);
    assert(bitaxe_stack_rejections(start, start + maximum + 1, true, true, maximum) == 8);
    assert(bitaxe_stack_rejections(end, start, true, true, maximum) == 12);
}
static void wrong_snapshot_cannot_be_current_task_evidence(void)
{
    assert(bitaxe_snapshot_matches(32, 32, true));
    assert(!bitaxe_snapshot_matches(32, 36, true));
    assert(!bitaxe_snapshot_matches(32, 32, false));
    assert(!bitaxe_snapshot_matches(0, 0, true));
}
static void torn_or_modified_receipts_are_not_committed(void)
{
    /* Arrange */
    volatile uint32_t words[BITAXE_PANIC_FRAME_WORDS] = {0};
    words[1] = 1; words[2] = BITAXE_PANIC_FRAME_WORDS; words[3] = 4;
    words[46] = bitaxe_panic_checksum(words); words[47] = ~words[46];
    /* Act / Assert */
    assert(!bitaxe_panic_record_valid(words));
    words[0] = BITAXE_PANIC_FRAME_MAGIC;
    assert(bitaxe_panic_record_valid(words));
    words[25] ^= 4;
    assert(!bitaxe_panic_record_valid(words));
    words[25] ^= 4; words[47] ^= 1;
    assert(!bitaxe_panic_record_valid(words));
}
int main(void)
{
    invalid_spans_never_require_pointer_access();
    stack_predicates_preserve_each_sdk_reason();
    wrong_snapshot_cannot_be_current_task_evidence();
    torn_or_modified_receipts_are_not_committed();
    return puts("panic_model_tests=passed") < 0;
}
