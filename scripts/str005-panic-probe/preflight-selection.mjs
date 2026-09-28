import { installedPredecessor } from './installed-predecessor.mjs';
import { verifyCorePreservation } from './core-preservation.mjs';
import { packageSnapshot } from '../fixed-usb-qualification/contract.mjs';
import { recoveryPredecessor, retainedPackage, beforeRecovery } from './recovery-predecessor.mjs';

/** Separate current host provenance from a retained installed image in both no-write modes. */
export async function resolvePreflightSources(options, firmwareRoot, commit, operations = { packageSnapshot, recoveryPredecessor, retainedPackage, beforeRecovery, verifyCorePreservation, installedPredecessor }) {
  const recoveryOnly = Boolean(options['--recover-install-root'] || options['--recover-installed-root']);
  const captureExisting = Boolean(options['--capture-recovery-root']);
  const beforeRoot = options['--capture-recovery-root'] ?? options['--before-recovery-root'];
  const before = beforeRoot ? await operations.beforeRecovery(beforeRoot, firmwareRoot) : undefined;
  const failedInstall = options['--recover-install-root'] ? await operations.recoveryPredecessor(options['--recover-install-root'], firmwareRoot) : captureExisting && before.kind !== 'verified_installation' ? before.predecessor : undefined;
  const installed = options['--recover-installed-root'] ? await operations.installedPredecessor(options['--recover-installed-root'], firmwareRoot) : captureExisting && before.kind === 'verified_installation' ? before.predecessor : undefined;
  const retainedSource = installed ?? failedInstall;
  const retained = retainedSource ? await operations.retainedPackage(options['--retained-manifest'], retainedSource, firmwareRoot) : undefined;
  const packaged = retained?.packaged ?? await operations.packageSnapshot(firmwareRoot, options['--manifest'], commit);
  const corePreservation = captureExisting ? await operations.verifyCorePreservation(before.root, before.context) : undefined;
  return { corePreservation, recoveryOnly, captureExisting, before, failedInstall, installed, retainedSource, retained, packaged };
}
