#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "bwg_usb.h"
#include "FreeRTOS.h"
#include "task.h"
#include "tinyusb.h"
#include "tusb.h"
#include "esp_system.h"
#include "esp_private/periph_ctrl.h"
#include "soc/soc.h"
#include "soc/periph_defs.h"
#include "soc/rtc_cntl_reg.h"
#include "soc/usb_serial_jtag_reg.h"

extern void bwg_worker_usb_attached(void);
extern void bwg_worker_usb_detached(void);
extern void bwg_worker_usb_vendor_received(const uint8_t *bytes, uint32_t length);
extern void usb_runtime_line_coding(uint32_t bit_rate);
extern void usb_runtime_line_state(bool dtr, bool rts);

enum {
    BWG_INTERFACE_VENDOR = 0,
    BWG_INTERFACE_CDC = 1,
    BWG_INTERFACE_CDC_DATA = 2,
    BWG_INTERFACE_COUNT = 3,
};

enum {
    BWG_STRING_MANUFACTURER = 1,
    BWG_STRING_PRODUCT = 2,
    BWG_STRING_CONTROL = 3,
    BWG_STRING_EVIDENCE = 4,
};

#define BWG_CONFIG_TOTAL_LENGTH (TUD_CONFIG_DESC_LEN + TUD_VENDOR_DESC_LEN + TUD_CDC_DESC_LEN)

static const tusb_desc_device_t BWG_DEVICE_DESCRIPTOR = {
    .bLength = sizeof(tusb_desc_device_t),
    .bDescriptorType = TUSB_DESC_DEVICE,
    .bcdUSB = 0x0200,
    .bDeviceClass = TUSB_CLASS_MISC,
    .bDeviceSubClass = MISC_SUBCLASS_COMMON,
    .bDeviceProtocol = MISC_PROTOCOL_IAD,
    .bMaxPacketSize0 = CFG_TUD_ENDPOINT0_SIZE,
    .idVendor = 0x1209,
    .idProduct = 0xb17a,
    .bcdDevice = 0x0001,
    .iManufacturer = BWG_STRING_MANUFACTURER,
    .iProduct = BWG_STRING_PRODUCT,
    .iSerialNumber = 0,
    .bNumConfigurations = 1,
};

static const uint8_t BWG_CONFIGURATION_DESCRIPTOR[] = {
    TUD_CONFIG_DESCRIPTOR(1, BWG_INTERFACE_COUNT, 0, BWG_CONFIG_TOTAL_LENGTH, 0, 100),

    9, TUSB_DESC_INTERFACE, BWG_INTERFACE_VENDOR, 0, 2,
    TUSB_CLASS_VENDOR_SPECIFIC, 0x42, 0x01, BWG_STRING_CONTROL,
    7, TUSB_DESC_ENDPOINT, 0x01, TUSB_XFER_BULK, U16_TO_U8S_LE(64), 0,
    7, TUSB_DESC_ENDPOINT, 0x81, TUSB_XFER_BULK, U16_TO_U8S_LE(64), 0,

    TUD_CDC_DESCRIPTOR(BWG_INTERFACE_CDC, BWG_STRING_EVIDENCE, 0x82, 8, 0x03, 0x83, 64),
};

static const char *BWG_STRING_DESCRIPTORS[] = {
    (const char[]){0x09, 0x04},
    "Bright Builds",
    "Bitaxe Ultra 205 BWG Worker",
    "BWG Worker Control",
    "BWG Worker Evidence",
};

int32_t bwg_usb_install(void)
{
    const tinyusb_config_t config = {
        .device_descriptor = &BWG_DEVICE_DESCRIPTOR,
        .string_descriptor = BWG_STRING_DESCRIPTORS,
        .string_descriptor_count = sizeof(BWG_STRING_DESCRIPTORS) / sizeof(BWG_STRING_DESCRIPTORS[0]),
        .external_phy = false,
        .configuration_descriptor = BWG_CONFIGURATION_DESCRIPTOR,
        .self_powered = false,
        .vbus_monitor_io = -1,
    };
    return tinyusb_driver_install(&config);
}

void tud_mount_cb(void)
{
    bwg_worker_usb_attached();
}

void tud_umount_cb(void)
{
    bwg_worker_usb_detached();
}

uint32_t bwg_usb_vendor_write(const uint8_t *bytes, uint32_t length)
{
    if (bytes == NULL || length == 0 || !tud_vendor_mounted()) {
        return 0;
    }
    uint32_t total = 0;
    uint32_t waits = 0;
    while (total < length && tud_vendor_mounted()) {
        uint32_t available = tud_vendor_write_available();
        if (available == 0) {
            tud_vendor_write_flush();
            vTaskDelay(pdMS_TO_TICKS(1));
            if (++waits > 2000) {
                tud_vendor_write_clear();
                return 0;
            }
            continue;
        }
        uint32_t remaining = length - total;
        uint32_t chunk = remaining < available ? remaining : available;
        uint32_t written = tud_vendor_write(bytes + total, chunk);
        if (written == 0) {
            continue;
        }
        total += written;
        tud_vendor_write_flush();
    }
    return total;
}

uint32_t bwg_usb_evidence_write(const uint8_t *bytes, uint32_t length)
{
    if (bytes == NULL || length == 0 || !tud_cdc_connected()) {
        return 0;
    }
    uint32_t written = tud_cdc_write(bytes, length);
    if (written != length) {
        tud_cdc_write_clear();
        return 0;
    }
    tud_cdc_write_flush();
    return written;
}

void tud_vendor_rx_cb(uint8_t interface, const uint8_t *buffer, uint16_t length)
{
    if (interface != 0) {
        return;
    }
    if (buffer != NULL && length > 0) {
        bwg_worker_usb_vendor_received(buffer, length);
        return;
    }
    uint8_t chunk[256];
    while (tud_vendor_available() > 0) {
        uint32_t received = tud_vendor_read(chunk, sizeof(chunk));
        if (received == 0) {
            return;
        }
        bwg_worker_usb_vendor_received(chunk, received);
    }
}

void tud_cdc_rx_cb(uint8_t interface)
{
    uint8_t discarded[64];
    while (tud_cdc_n_available(interface) > 0) {
        (void)tud_cdc_n_read(interface, discarded, sizeof(discarded));
    }
}

void tud_cdc_line_coding_cb(uint8_t interface, cdc_line_coding_t const *coding)
{
    if (interface == 0 && coding != NULL) {
        usb_runtime_line_coding(coding->bit_rate);
    }
}

void tud_cdc_line_state_cb(uint8_t interface, bool dtr, bool rts)
{
    if (interface == 0) {
        usb_runtime_line_state(dtr, rts);
    }
}

static void IRAM_ATTR bwg_usb_force_download_shutdown(void)
{
    REG_WRITE(RTC_CNTL_OPTION1_REG, RTC_CNTL_FORCE_DOWNLOAD_BOOT);
}

static void bwg_usb_switch_to_serial_jtag(void)
{
    /*
     * Minimal ESP32-S3 half of Espressif's usb_switch_to_cdc_jtag(), pinned at
     * arduino-esp32 bb0bb3ec57fbcf7efb8409f727fb792e3d28fe79:
     * cores/esp32/esp32-hal-tinyusb.c. esp_tinyusb already deletes the OTG PHY
     * during uninstall, so retain only module reset and internal-PHY routing.
     */
    periph_module_reset(PERIPH_USB_MODULE);
    periph_module_disable(PERIPH_USB_MODULE);
    CLEAR_PERI_REG_MASK(
        RTC_CNTL_USB_CONF_REG,
        RTC_CNTL_SW_HW_USB_PHY_SEL | RTC_CNTL_SW_USB_PHY_SEL | RTC_CNTL_USB_PAD_ENABLE);
    CLEAR_PERI_REG_MASK(USB_SERIAL_JTAG_CONF0_REG, USB_SERIAL_JTAG_PHY_SEL);
    CLEAR_PERI_REG_MASK(USB_SERIAL_JTAG_CONF0_REG, USB_SERIAL_JTAG_USB_PAD_ENABLE);
    SET_PERI_REG_MASK(USB_SERIAL_JTAG_CONF0_REG, USB_SERIAL_JTAG_USB_PAD_ENABLE);
}

int32_t bwg_usb_restart_bootloader(void)
{
    int32_t result = esp_register_shutdown_handler(bwg_usb_force_download_shutdown);
    if (result != ESP_OK) {
        return result;
    }
    result = tinyusb_driver_uninstall();
    if (result != ESP_OK) {
        (void)esp_unregister_shutdown_handler(bwg_usb_force_download_shutdown);
        return result;
    }
    bwg_usb_switch_to_serial_jtag();
    esp_restart();
    return ESP_FAIL;
}
