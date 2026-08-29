#include "usb_phy_handoff.h"

#include "driver/gpio.h"
#include "esp_private/periph_ctrl.h"
#include "esp_rom_sys.h"
#include "esp_system.h"
#include "hal/usb_serial_jtag_ll.h"
#include "soc/periph_defs.h"
#include "soc/rtc_cntl_reg.h"
#include "soc/soc.h"
#include "soc/usb_pins.h"
#include "soc/usb_serial_jtag_reg.h"
#include "tinyusb.h"

#define BUS_RESET_TIMEOUT_US 1000000U
#define BUS_RESET_POLL_US 1000U

static void IRAM_ATTR bitaxe_usb_force_download_shutdown(void)
{
    REG_WRITE(RTC_CNTL_OPTION1_REG, RTC_CNTL_FORCE_DOWNLOAD_BOOT);
}

static int32_t bitaxe_usb_switch_to_serial_jtag(void)
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

    const gpio_config_t disconnect = {
        .pin_bit_mask = (1ULL << USBPHY_DM_NUM) | (1ULL << USBPHY_DP_NUM),
        .mode = GPIO_MODE_OUTPUT_OD,
        .pull_up_en = GPIO_PULLUP_DISABLE,
        .pull_down_en = GPIO_PULLDOWN_DISABLE,
        .intr_type = GPIO_INTR_DISABLE,
    };
    int32_t result = gpio_config(&disconnect);
    if (result != ESP_OK) {
        return result;
    }
    result = gpio_set_level(USBPHY_DM_NUM, 0);
    if (result != ESP_OK) {
        return result;
    }
    result = gpio_set_level(USBPHY_DP_NUM, 0);
    if (result != ESP_OK) {
        return result;
    }

    usb_serial_jtag_ll_disable_intr_mask(USB_SERIAL_JTAG_LL_INTR_MASK);
    usb_serial_jtag_ll_clr_intsts_mask(USB_SERIAL_JTAG_LL_INTR_MASK);
    usb_serial_jtag_ll_ena_intr_mask(USB_SERIAL_JTAG_INTR_BUS_RESET);
    SET_PERI_REG_MASK(USB_SERIAL_JTAG_CONF0_REG, USB_SERIAL_JTAG_USB_PAD_ENABLE);

    uint32_t elapsed_us = 0;
    while (elapsed_us < BUS_RESET_TIMEOUT_US) {
        if (usb_serial_jtag_ll_get_intraw_mask() & USB_SERIAL_JTAG_INTR_BUS_RESET) {
            usb_serial_jtag_ll_disable_intr_mask(USB_SERIAL_JTAG_LL_INTR_MASK);
            usb_serial_jtag_ll_clr_intsts_mask(USB_SERIAL_JTAG_LL_INTR_MASK);
            return ESP_OK;
        }
        esp_rom_delay_us(BUS_RESET_POLL_US);
        elapsed_us += BUS_RESET_POLL_US;
    }
    usb_serial_jtag_ll_disable_intr_mask(USB_SERIAL_JTAG_LL_INTR_MASK);
    usb_serial_jtag_ll_clr_intsts_mask(USB_SERIAL_JTAG_LL_INTR_MASK);
    return ESP_ERR_TIMEOUT;
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
    result = bitaxe_usb_switch_to_serial_jtag();
    if (result != ESP_OK) {
        (void)esp_unregister_shutdown_handler(bitaxe_usb_force_download_shutdown);
        return result;
    }
    esp_restart();
    return ESP_FAIL;
}
