#include "bitaxe_fault_provenance.h"
#include "panic_model.h"
#include <stdbool.h>
#include <stddef.h>
#include "esp_attr.h"
#include "esp_memory_utils.h"
#include "esp_private/panic_internal.h"
#include "esp_core_dump_port.h"
#include "esp_core_dump_types.h"
#include "soc/soc.h"
#include "xtensa_context.h"

#if !CONFIG_IDF_TARGET_ESP32S3
#error "Fault provenance requires the reviewed ESP32-S3 ABI"
#endif
#if CONFIG_ESP_SYSTEM_ALLOW_RTC_FAST_MEM_AS_HEAP
#define BITAXE_RTC_FLAG 1u
#else
#define BITAXE_RTC_FLAG 0u
#endif
#if CONFIG_FREERTOS_TASK_CREATE_ALLOW_EXT_MEM
#define BITAXE_EXT_FLAG 2u
#else
#define BITAXE_EXT_FLAG 0u
#endif
#define BITAXE_SDK_CONFIG (BITAXE_RTC_FLAG | BITAXE_EXT_FLAG)
#define ABI_OFFSET(type, field, expected) _Static_assert(offsetof(type, field) == expected, "SDK field offset changed")
_Static_assert(sizeof(void *) == 4 && sizeof(panic_info_t) == 36, "SDK panic ABI changed");
_Static_assert(sizeof(XtExcFrame) == 112 && sizeof(core_dump_task_header_t) == 12, "SDK frame/header ABI changed");
_Static_assert(COREDUMP_MAX_TASK_STACK_SIZE == 65536, "SDK stack limit changed");
ABI_OFFSET(panic_info_t, core, 0); ABI_OFFSET(panic_info_t, exception, 4);
ABI_OFFSET(panic_info_t, reason, 8); ABI_OFFSET(panic_info_t, description, 12);
ABI_OFFSET(panic_info_t, addr, 24); ABI_OFFSET(panic_info_t, frame, 28); ABI_OFFSET(panic_info_t, pseudo_excause, 32);
ABI_OFFSET(XtExcFrame, exit, 0); ABI_OFFSET(XtExcFrame, pc, 4); ABI_OFFSET(XtExcFrame, ps, 8);
ABI_OFFSET(XtExcFrame, a0, 12); ABI_OFFSET(XtExcFrame, a1, 16);
ABI_OFFSET(XtExcFrame, exccause, 80); ABI_OFFSET(XtExcFrame, excvaddr, 84);
ABI_OFFSET(core_dump_task_header_t, tcb_addr, 0); ABI_OFFSET(core_dump_task_header_t, stack_start, 4);
ABI_OFFSET(core_dump_task_header_t, stack_end, 8);

const uint32_t BITAXE_FAULT_PROVENANCE_ABI[BITAXE_FAULT_ABI_WORDS] __attribute__((used)) = {
    BITAXE_FAULT_ABI_MAGIC, 1, BITAXE_PANIC_FRAME_WORDS, sizeof(panic_info_t), sizeof(XtExcFrame), sizeof(core_dump_task_header_t),
    offsetof(panic_info_t, core), offsetof(panic_info_t, exception), offsetof(panic_info_t, reason), offsetof(panic_info_t, description),
    offsetof(panic_info_t, addr), offsetof(panic_info_t, frame), offsetof(panic_info_t, pseudo_excause),
    offsetof(XtExcFrame, exit), offsetof(XtExcFrame, pc), offsetof(XtExcFrame, ps), offsetof(XtExcFrame, a0), offsetof(XtExcFrame, a1),
    offsetof(XtExcFrame, exccause), offsetof(XtExcFrame, excvaddr), offsetof(core_dump_task_header_t, tcb_addr),
    offsetof(core_dump_task_header_t, stack_start), offsetof(core_dump_task_header_t, stack_end),
    SOC_DRAM_LOW, SOC_DRAM_HIGH, SOC_RTC_DRAM_LOW, SOC_RTC_DRAM_HIGH, BITAXE_SDK_CONFIG,
    COREDUMP_MAX_TASK_STACK_SIZE, sizeof(XtExcFrame), 0, BITAXE_PANIC_FRAME_MAGIC,
};
volatile uint32_t BITAXE_PANIC_FRAME_RECORD[BITAXE_PANIC_FRAME_WORDS] COREDUMP_DRAM_ATTR;
static DRAM_ATTR volatile uint32_t identity[4];
/* Only the SDK's crashed-task callback may copy this transient header into retained evidence. */
static DRAM_ATTR volatile uint32_t snapshot[12];
static DRAM_ATTR volatile uint32_t checks;
static DRAM_ATTR volatile uint32_t capture_active;

static inline __attribute__((always_inline)) void barrier(void) { __asm__ volatile("memw" ::: "memory"); }
static inline __attribute__((always_inline)) uint32_t cycle_count(void)
{
    uint32_t value; __asm__ volatile("rsr.ccount %0" : "=a"(value)); return value;
}
static inline __attribute__((always_inline)) bool readable(const void *ptr, uint32_t size)
{
    return bitaxe_readable_span((uint32_t)ptr, size, SOC_DRAM_LOW, SOC_DRAM_HIGH);
}
static IRAM_ATTR __attribute__((noinline)) void commit_record(void)
{
    BITAXE_PANIC_FRAME_RECORD[46] = bitaxe_panic_checksum(BITAXE_PANIC_FRAME_RECORD);
    BITAXE_PANIC_FRAME_RECORD[47] = ~BITAXE_PANIC_FRAME_RECORD[46];
    barrier(); BITAXE_PANIC_FRAME_RECORD[0] = BITAXE_PANIC_FRAME_MAGIC;
}
void bitaxe_fault_set_identity(uint32_t source_lo, uint32_t source_hi, uint32_t boot_lo, uint32_t boot_hi)
{
    identity[0] = source_lo; identity[1] = source_hi; identity[2] = boot_lo; identity[3] = boot_hi;
}
void IRAM_ATTR __attribute__((noinline)) bitaxe_capture_original_panic(const void *opaque)
{
    volatile uint32_t *record = BITAXE_PANIC_FRAME_RECORD;
    if (capture_active) {
        record[0] = 0; barrier(); record[3] |= 32u; commit_record(); return;
    }
    capture_active = 1;
    record[0] = 0; barrier();
    record[1] = 1; record[2] = BITAXE_PANIC_FRAME_WORDS; record[3] = 4;
    record[4] = identity[0]; record[5] = identity[1]; record[6] = identity[2]; record[7] = identity[3];
    record[9] = cycle_count(); record[10] = (uint32_t)opaque;
    record[35] = BITAXE_SDK_CONFIG; record[36] = SOC_DRAM_LOW; record[37] = SOC_DRAM_HIGH;
    record[38] = SOC_RTC_DRAM_LOW; record[39] = SOC_RTC_DRAM_HIGH;
    record[40] = COREDUMP_MAX_TASK_STACK_SIZE; record[41] = sizeof(panic_info_t); record[42] = sizeof(XtExcFrame);
    if (readable(opaque, sizeof(panic_info_t))) {
        const volatile panic_info_t *info = opaque;
        record[3] |= 1u; record[8] = (uint32_t)info->core; record[11] = info->exception;
        record[12] = (uint32_t)info->reason; record[13] = (uint32_t)info->description;
        record[14] = (uint32_t)info->addr; const void *frame = info->frame; record[15] = (uint32_t)frame;
        /* Read the underlying byte: a corrupt C bool must not create invalid Rust/C boolean values. */
        record[16] = *((const volatile uint8_t *)opaque + offsetof(panic_info_t, pseudo_excause));
        if (readable(frame, sizeof(XtExcFrame))) {
            const volatile XtExcFrame *native = frame;
            record[3] |= 2u; record[17] = (uint32_t)native->exit; record[18] = (uint32_t)native->pc;
            record[19] = (uint32_t)native->ps; record[20] = (uint32_t)native->a0; record[21] = (uint32_t)native->a1;
            record[22] = (uint32_t)native->exccause; record[23] = (uint32_t)native->excvaddr;
        }
    }
    commit_record();
}
extern bool __real_esp_core_dump_check_task(core_dump_task_header_t *task);
extern void __real_esp_core_dump_port_set_crashed_tcb(uint32_t handle);

bool IRAM_ATTR __attribute__((noinline)) __wrap_esp_core_dump_check_task(core_dump_task_header_t *task)
{
    snapshot[10] = 0;
    const bool valid = capture_active && readable(task, sizeof(*task));
    if (valid) {
        const volatile core_dump_task_header_t *header = task;
        const uint32_t start = header->stack_start, end = header->stack_end;
        snapshot[0] = (uint32_t)header->tcb_addr; snapshot[1] = start; snapshot[2] = end;
        bool end_sane = esp_ptr_in_dram((void *)end);
#if CONFIG_FREERTOS_TASK_CREATE_ALLOW_EXT_MEM
        end_sane = end_sane || esp_stack_ptr_in_extram(end);
#endif
#if CONFIG_ESP_SYSTEM_ALLOW_RTC_FAST_MEM_AS_HEAP
        end_sane = end_sane || esp_ptr_in_rtc_dram_fast((void *)end);
#endif
        const bool start_sane = esp_stack_ptr_is_sane(start);
        snapshot[7] = bitaxe_stack_rejections(start, end, start_sane, end_sane, COREDUMP_MAX_TASK_STACK_SIZE);
        snapshot[9] = start_sane ? 1u : 0u; snapshot[11] = end_sane ? 1u : 0u;
    }
    const bool result = __real_esp_core_dump_check_task(task);
    if (valid) {
        const volatile core_dump_task_header_t *header = task;
        snapshot[3] = header->stack_start; snapshot[4] = header->stack_end; snapshot[5] = result ? 1u : 0u;
        snapshot[6] = snapshot[3] >= UINT32_C(0x20000000) && snapshot[3] < UINT32_C(0x30000000);
        snapshot[8] = ++checks; barrier(); snapshot[10] = 1;
    }
    return result;
}
void IRAM_ATTR __attribute__((noinline)) __wrap_esp_core_dump_port_set_crashed_tcb(uint32_t handle)
{
    volatile uint32_t *record = BITAXE_PANIC_FRAME_RECORD;
    if (capture_active && bitaxe_snapshot_matches(handle, snapshot[0], snapshot[10] == 1)) {
        record[0] = 0; barrier();
        if (record[33] == 0) {
            record[24] = handle; record[25] = snapshot[1]; record[26] = snapshot[2];
            record[27] = snapshot[3]; record[28] = snapshot[4]; record[29] = snapshot[5];
            record[30] = snapshot[7]; record[31] = snapshot[6]; record[32] = snapshot[8];
            record[43] = record[15] == snapshot[1]; record[44] = snapshot[9]; record[45] = snapshot[11];
        } else if (record[24] != handle || record[25] != snapshot[1] || record[26] != snapshot[2] ||
                   record[27] != snapshot[3] || record[28] != snapshot[4] || record[30] != snapshot[7] ||
                   record[31] != snapshot[6] || record[29] != snapshot[5]) {
            record[34] = 1;
        }
        record[33] += 1; record[3] |= 24u; commit_record();
    }
    __real_esp_core_dump_port_set_crashed_tcb(handle);
}
