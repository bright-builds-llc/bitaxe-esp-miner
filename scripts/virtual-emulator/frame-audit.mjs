/** Protect the released receive frame; this is not a complete native call-closure audit. */
export function auditGuestFrames(disassembly, sdkconfig) {
  const frames = new Map();
  const expression = /^([0-9a-f]+) <(.+)>:\n[^\n]*?\bentry\s+a1,\s*(0x[0-9a-f]+|[0-9]+)/gm;
  for (const match of disassembly.matchAll(expression)) frames.set(match[2], Number(match[3]));
  const dispatch = frames.get('bitaxe_virtual_firmware::guest::run');
  const receive = frames.get('bitaxe_virtual_firmware::guest::read_command');
  if (!Number.isInteger(dispatch) || dispatch > 2048 || !Number.isInteger(receive) || receive < 4096 || receive > 4608 ||
      !sdkconfig.split('\n').includes('CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384')) throw Error('guest_receive_frame_not_released');
  return { schema: 'bitaxe-virtual-guest-frame-audit-v1', dispatch_frame_bytes: dispatch,
    receive_frame_bytes: receive, main_stack_bytes: 16384, receive_frame_separate: true,
    complete_call_closure_audited: false };
}
