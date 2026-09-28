import { admittedImage } from './admission.mjs';
import { sealed } from '../str005-startup-probe/capture.mjs';
import { STARTUP_SEAL } from './contract.mjs';
import { check } from '../str005-v2-serial/values.mjs';

/** Re-read every producer byte and pinned inventory; never inspect current USB ownership. */
export async function verifyPinnedInventories(pins) {
  for (const [root, expected] of pins) check(await sealed(root) === expected, 'share_producer_changed');
}

/** Full admission runs once before ownership, then returns only immutable-evidence verification. */
export async function beginAdmissionVerification(firmwareRoot, input, operations = {}) {
  const manifest = structuredClone(input);
  await (operations.admit ?? admittedImage)(firmwareRoot, manifest);
  const pins = [
    [manifest.installationRoot, manifest.installationSeal], [manifest.startupRoot, STARTUP_SEAL],
    [manifest.captureBindings.captureRoot, manifest.captureSeal], [manifest.clearRoot, manifest.clearSeal],
    [manifest.recoveryRoot, manifest.recoverySeal],
  ];
  // captureEvidence has already proved archive, decoder and recovery inputs live inside this one capture root.
  return () => (operations.verifyInventories ?? verifyPinnedInventories)(pins);
}
