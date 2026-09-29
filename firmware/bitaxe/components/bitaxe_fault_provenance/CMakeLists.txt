idf_component_register(SRCS "panic.c" "allocation.c"
                       INCLUDE_DIRS "include"
                       PRIV_REQUIRES esp_system esp_hw_support freertos espcoredump heap soc
                                     esp_app_format bootloader_support hal esp_rom log)
# The SDK owns the ABI; compile against its pinned private declarations rather than duplicating layouts.
idf_build_get_property(idf_path IDF_PATH)
target_include_directories(${COMPONENT_LIB} PRIVATE "${idf_path}/components/espcoredump/include_core_dump")
target_compile_options(${COMPONENT_LIB} PRIVATE -fno-jump-tables -fno-tree-switch-conversion)
