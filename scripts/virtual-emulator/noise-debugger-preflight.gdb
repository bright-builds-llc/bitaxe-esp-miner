set auto-load no
set pagination off
set confirm off
set debuginfod enabled off
set language c
printf "ABI_TCB_BYTES %u\n", (unsigned)sizeof(TCB_t)
printf "ABI_STACK_OFFSET %u\n", (unsigned)&((TCB_t*)0)->pxStack
printf "ABI_END_OFFSET %u\n", (unsigned)&((TCB_t*)0)->pxEndOfStack
printf "ABI_TOP_OFFSET %u\n", (unsigned)&((TCB_t*)0)->pxTopOfStack
printf "ABI_NAME_OFFSET %u\n", (unsigned)&((TCB_t*)0)->pcTaskName
printf "ABI_PANIC_BYTES %u\n", (unsigned)sizeof(panic_info_t)
printf "ABI_FRAME_OFFSET %u\n", (unsigned)&((panic_info_t*)0)->frame
printf "ABI_EXCEPTION_BYTES %u\n", (unsigned)sizeof(XtExcFrame)
printf "ABI_CURRENT_BYTES %u\n", (unsigned)sizeof(pxCurrentTCBs)
info address esp_panic_handler
info address __assert_func
info address bitaxe_virtual_noise_prefix_cutoff
info address bitaxe_virtual_noise_checkpoint_complete
info address bitaxe_virtual_noise_prefix_released
