// Device Noise stack audit: the same frame parser and crypto descent as the emulator auditor.
import { parseNoiseFrames, measureNoisePath, nestedCryptoPath } from './virtual-emulator/noise-stack-audit.mjs';

export const WORKER_STACK_BYTES = 12 * 1024;
export const HELPER_STACK_BYTES = 16 * 1024;
export const MARGIN_BYTES = 2048;
const COMPLETE = 'bitaxe_stratum::v2::noise::completion::<impl bitaxe_stratum::v2::noise::NoiseInitiator>::complete_diagnostic_into';
const STEP_TWO = 'noise_sv2::initiator::Initiator::step_2_with_now';
const VERIFY = 'rustsecp256k1_v0_9_2_schnorrsig_verify';
const HELPER = 'bitaxe_firmware::noise_completion_stack::complete_in_helper';
const DIAGNOSTIC = 'bitaxe_stratum::v2::noise::diagnostic::run';
const SHARE = ['bitaxe_firmware::production_mining_session::transport::run_worker',
  'bitaxe_firmware::production_mining_session::transport::v2::run',
  'bitaxe_firmware::v2_serial_runtime::run_share_transport', 'bitaxe_firmware::v2_serial_runtime::observer::run_share'];
const CHANNEL = ['bitaxe_v2_channel_owner_entry', 'bitaxe_firmware::v2_serial_runtime::observer::run_channel'];
const SPAWN = 'bitaxe_firmware::noise_completion_stack::authenticate_on_psram_stack';
const REQUIRED = [
  { id: 'helper_completion', stack: HELPER_STACK_BYTES, names: [HELPER, COMPLETE], crypto: STEP_TWO },
  { id: 'helper_certificate', stack: HELPER_STACK_BYTES, names: [HELPER, COMPLETE, STEP_TWO], crypto: VERIFY },
  { id: 'worker_share_crypto', stack: WORKER_STACK_BYTES, names: SHARE, crypto: DIAGNOSTIC },
  { id: 'worker_channel_crypto', stack: WORKER_STACK_BYTES, names: CHANNEL, crypto: DIAGNOSTIC },
  { id: 'worker_share_helper_spawn', stack: WORKER_STACK_BYTES, names: [...SHARE, DIAGNOSTIC, SPAWN], crypto: null },
];
// Source contracts the measured budgets depend on.
const SOURCE_CONTRACTS = [
  ['helper', 'pub(crate) const COMPLETION_STACK_BYTES: usize = 16 * 1024;'],
  ['helper', 'config.stack_alloc_caps = sys::MALLOC_CAP_SPIRAM | sys::MALLOC_CAP_8BIT;'],
  ['transport', 'const WORKER_STACK_BYTES: usize = 12 * 1024;'],
];

/** Every direct caller of completion crypto must be the helper entry; nothing else may host it. */
export function completionOnlyOnHelper(frames) {
  const targets = new Set([COMPLETE, STEP_TWO].flatMap(name => [...(frames.byName.get(name) ?? [])]));
  if (!targets.size || !frames.byName.has(HELPER)) throw Error('device_noise_required_symbol_missing');
  const allowed = new Set([...frames.byName.get(HELPER), ...(frames.byName.get(COMPLETE) ?? [])]);
  for (const [address, frame] of frames.byAddress) {
    if (frame.calls.some(call => targets.has(call)) && !allowed.has(address)) return false;
  }
  return true;
}

/** Named paths plus the longest resolved crypto descent; never a global upper bound. */
export function auditDeviceNoiseStack(disassembly, sources) {
  for (const [file, line] of SOURCE_CONTRACTS) {
    if (!sources[file]?.includes(line)) throw Error('device_noise_stack_contract');
  }
  const frames = parseNoiseFrames(disassembly);
  const paths = REQUIRED.map(({ id, stack, names, crypto }) => {
    const nested = crypto ? nestedCryptoPath(frames, crypto) : { names: [], outside_crypto_family_edges: null };
    const measured = measureNoisePath(frames, [...names, ...nested.names]);
    const budget = stack - MARGIN_BYTES;
    return { id, stack_bytes: stack, budget_bytes: budget, frame_bytes: measured.frame_bytes,
      fit: measured.frame_bytes <= budget, native_symbols: measured.native_symbols,
      unresolved_call_edges: measured.unresolved_call_edges, outside_crypto_family_edges: nested.outside_crypto_family_edges };
  });
  const isolated = completionOnlyOnHelper(frames);
  return { schema: 'bitaxe-device-noise-stack-audit-v1', completion_only_on_helper: isolated,
    helper_stack_placement: 'psram', helper_entry_frames_bounded: false, complete_callgraph_bound: false,
    paths, result: isolated && paths.every(path => path.fit) ? 'selected_path_with_headroom' : 'blocked' };
}
