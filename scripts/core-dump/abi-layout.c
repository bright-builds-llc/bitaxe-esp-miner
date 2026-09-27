/* Native layout facts from IDF v5.5.4 core_dump_elf.c; no device data. */
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

struct version_info {
    uint32_t version;
    uint8_t app_elf_sha256[66];
};

_Static_assert(sizeof(struct version_info) == 72, "native descriptor size");
_Static_assert(offsetof(struct version_info, app_elf_sha256) == 4, "native SHA offset");

int main(void) {
    return printf("72 4 66\n") < 0;
}
