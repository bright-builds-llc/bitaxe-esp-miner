set auto-load no
set pagination off
set confirm off
set print frame-arguments none
set debuginfod enabled off
set language c
set remotetimeout 5
set $snapshots = 0
define capture-noise
 if $snapshots == 0
@CAPTURE_0@
 else
@CAPTURE_1@
 end
 set $snapshots = $snapshots + 1
end
target remote 127.0.0.1:@PORT@
break esp_panic_handler
commands
 silent
 set $capture_kind = 1
 capture-noise
 if $snapshots >= 2
  detach
  quit
 end
 continue
end
break __assert_func
commands
 silent
 set $capture_kind = 2
 capture-noise
 if $snapshots >= 2
  detach
  quit
 end
 continue
end
break bitaxe_virtual_noise_prefix_cutoff if $a2 == @CUTOFF@ && *(unsigned*)&BITAXE_VIRTUAL_NOISE_PREFIX_REACHED == @CUTOFF@ && *(unsigned*)&BITAXE_VIRTUAL_NOISE_PREFIX_RELEASED == 0
commands
 silent
 set $capture_kind = 3
 capture-noise
 if $snapshots >= 2
  detach
  quit
 end
 continue
end
break bitaxe_virtual_noise_prefix_released if $a2 == @CUTOFF@ && *(unsigned*)&BITAXE_VIRTUAL_NOISE_PREFIX_REACHED == @CUTOFF@ && *(unsigned*)&BITAXE_VIRTUAL_NOISE_PREFIX_RELEASED == 1
commands
 silent
 set $capture_kind = 4
 capture-noise
 detach
 quit
end
continue
