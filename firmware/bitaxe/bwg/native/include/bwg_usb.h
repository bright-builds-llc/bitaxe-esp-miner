#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int32_t bwg_usb_install(void);
uint32_t bwg_usb_vendor_write(const uint8_t *bytes, uint32_t length);
uint32_t bwg_usb_evidence_write(const uint8_t *bytes, uint32_t length);
int32_t bwg_usb_restart_bootloader(void);

#ifdef __cplusplus
}
#endif
