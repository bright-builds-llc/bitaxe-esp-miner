import { verifyCorePreservation } from './core-preservation.mjs';
import { packageSnapshot } from '../fixed-usb-qualification/contract.mjs';
import { recoveryPredecessor, retainedPackage, beforeRecovery } from './recovery-predecessor.mjs';

/** Separate current host provenance from a retained installed image in both no-write modes. */
export async function resolvePreflightSources(options, firmwareRoot, commit, operations = { packageSnapshot, recoveryPredecessor, retainedPackage, beforeRecovery, verifyCorePreservation }) {
  const recoveryOnly = Boolean(options['--recover-install-root']);
  const captureExisting = Boolean(options['--capture-recovery-root']);
  const beforeRoot = options['--capture-recovery-root'] ?? options['--before-recovery-root'];
  const before = beforeRoot ? await operations.beforeRecovery(beforeRoot, firmwareRoot) : undefined;
  const failedInstall = recoveryOnly ? await operations.recoveryPredecessor(options['--recover-install-root'], firmwareRoot) : captureExisting ? before.predecessor : undefined;
  const retained = failedInstall ? await operations.retainedPackage(options['--retained-manifest'], failedInstall, firmwareRoot) : undefined;
  const packaged = retained?.packaged ?? await operations.packageSnapshot(firmwareRoot, options['--manifest'], commit);
  const corePreservation = captureExisting ? await operations.verifyCorePreservation(before.root, before.context) : undefined;
  return { corePreservation, recoveryOnly, captureExisting, before, failedInstall, retained, packaged };
}
