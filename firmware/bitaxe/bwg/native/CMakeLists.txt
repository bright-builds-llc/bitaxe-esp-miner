idf_component_register(
    SRCS "usb_phy_handoff.c"
    INCLUDE_DIRS "include"
    PRIV_REQUIRES espressif__esp_tinyusb esp_driver_gpio esp_hw_support
)
