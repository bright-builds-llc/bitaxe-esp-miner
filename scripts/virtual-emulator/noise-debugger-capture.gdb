printf "SNAPSHOT_@INDEX@_KIND %u\n", $capture_kind
printf "SNAPSHOT_@INDEX@_PC %u\n", $pc
printf "SNAPSHOT_@INDEX@_SP %u\n", $a1
printf "SNAPSHOT_@INDEX@_ARG %u\n", $a2
dump binary memory "@ROOT@/snapshot@INDEX@-journal.raw" (char*)&BITAXE_VIRTUAL_NOISE_CHECKPOINTS (char*)&BITAXE_VIRTUAL_NOISE_CHECKPOINTS+768
dump binary memory "@ROOT@/snapshot@INDEX@-count.raw" (char*)&BITAXE_VIRTUAL_NOISE_CHECKPOINT_COUNT (char*)&BITAXE_VIRTUAL_NOISE_CHECKPOINT_COUNT+4
dump binary memory "@ROOT@/snapshot@INDEX@-current-tcbs.raw" (char*)&pxCurrentTCBs (char*)&pxCurrentTCBs+8
dump binary memory "@ROOT@/snapshot@INDEX@-prefix-reached.raw" (char*)&BITAXE_VIRTUAL_NOISE_PREFIX_REACHED (char*)&BITAXE_VIRTUAL_NOISE_PREFIX_REACHED+4
dump binary memory "@ROOT@/snapshot@INDEX@-prefix-released.raw" (char*)&BITAXE_VIRTUAL_NOISE_PREFIX_RELEASED (char*)&BITAXE_VIRTUAL_NOISE_PREFIX_RELEASED+4
set $tcb0 = (TCB_t*)pxCurrentTCBs[0]
set $tcb1 = (TCB_t*)pxCurrentTCBs[1]
if (unsigned)$tcb0 >= 0x3fc88000 && (unsigned)$tcb0 <= 0x3fd00000-340
 dump binary memory "@ROOT@/snapshot@INDEX@-tcb0.raw" (char*)$tcb0 (char*)$tcb0+340
end
if (unsigned)$tcb1 >= 0x3fc88000 && (unsigned)$tcb1 <= 0x3fd00000-340
 dump binary memory "@ROOT@/snapshot@INDEX@-tcb1.raw" (char*)$tcb1 (char*)$tcb1+340
end
if $capture_kind == 1 && (unsigned)$a2 >= 0x3fc88000 && (unsigned)$a2 <= 0x3fd00000-36
 dump binary memory "@ROOT@/snapshot@INDEX@-panic-info.raw" (char*)$a2 (char*)$a2+36
 set $exception_frame = ((panic_info_t*)$a2)->frame
 if (unsigned)$exception_frame >= 0x3fc88000 && (unsigned)$exception_frame <= 0x3fd00000-112
  dump binary memory "@ROOT@/snapshot@INDEX@-exception-frame.raw" (char*)$exception_frame (char*)$exception_frame+112
 end
end
bt 40
