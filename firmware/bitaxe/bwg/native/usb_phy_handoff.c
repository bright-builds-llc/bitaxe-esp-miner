#include "usb_phy_handoff.h"

#include "esp_private/periph_ctrl.h"
#include "esp_system.h"
#include "soc/periph_defs.h"
#include "soc/rtc_cntl_reg.h"
#include "soc/soc.h"
#include "soc/usb_serial_jtag_reg.h"
#include "tinyusb.h"

static void IRAM_ATTR bitaxe_usb_force_download_shutdown(void)
{
    REG_WRITE(RTC_CNTL_OPTION1_REG, RTC_CNTL_FORCE_DOWNLOAD_BOOT);
}

static void bitaxe_usb_switch_to_serial_jtag(void)
{
    /*
     * Minimal ESP32-S3 half of Espressif's usb_switch_to_cdc_jtag(), pinned at
     * arduino-esp32 bb0bb3ec57fbcf7efb8409f727fb792e3d28fe79:
     * cores/esp32/esp32-hal-tinyusb.c. esp_tinyusb deletes the OTG PHY during
     * uninstall; this Adapter retains module reset and internal-PHY routing.
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

int32_t bitaxe_usb_restart_bootloader(void)
{
    int32_t result = esp_register_shutdown_handler(bitaxe_usb_force_download_shutdown);
    if (result != ESP_OK) {
        return result;
    }
    result = tinyusb_driver_uninstall();
    if (result != ESP_OK) {
        (void)esp_unregister_shutdown_handler(bitaxe_usb_force_download_shutdown);
        return result;
    }
    bitaxe_usb_switch_to_serial_jtag();
    esp_restart();
    return ESP_FAIL;
}
