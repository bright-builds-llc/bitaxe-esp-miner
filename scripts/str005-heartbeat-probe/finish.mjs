import { validateFailure } from '../str005-startup-probe/failure.mjs';
import { readFile, readdir, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { proof, writeNew, inventory } from '../str005-noise-serial/files.mjs';
import { requireGone, requireNoHolders, requireLsofAbsent, signerExitProofs } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { main as inspect } from './inspect.mjs';
export async function optionalProof(root, name) {
  try { return (await proof(root, name)).value; } catch (error) { if (error.code !== 'ENOENT') throw error; return undefined; }
}
/** Missing proof makes a partial result; it never manufactures release or loses the first failure. */
export async function finalize(root, context, operations = {}) {
  const blockers = [], parts = { recovery: {}, hostReleased: false };
  parts.before = await optionalProof(root, 'before.json'); parts.run = await optionalProof(root, 'run.json');
  const server = await optionalProof(root, 'server-owner.json'), claim = await optionalProof(root, 'serve-claim.json');
  const fixtureClaim = await optionalProof(root, 'fixture-start.claim.json'), fixtureOwner = await optionalProof(root, 'fixture-owner.json');
  const released = await optionalProof(root, 'fixture-release.json'), clear = await optionalProof(root, 'clear-launch.json');
  let ownersGone = true, serialReleased = true;
  try {
    const owners = [];
    if (server?.owner) owners.push(server.owner); else if (claim?.owner) owners.push(claim.owner);
    if (fixtureOwner?.owner) owners.push(fixtureOwner.owner);
    if (owners.length) await (operations.requireGone ?? requireGone)(owners);
    if (server) (operations.requireLsofAbsent ?? requireLsofAbsent)(['-nP', `-iTCP:${server.port}`, '-sTCP:LISTEN', '-t']);
    await signerExitProofs(root, context);
  } catch { ownersGone = false; blockers.push('startup_host_ownership_unproven'); }
  const ports = new Set([context.detector.port]); if (clear) ports.add(clear.selectedPort);
  try {
    if (clear || parts.run?.startInvokedAt !== null && parts.run?.startInvokedAt !== undefined) {
      const path = resolve(dirname(root), 'final-detector.stdout.log');
      const detector = parseDetector(await readFile(path, 'utf8'), context.physical, Date.now() - (await stat(path)).mtimeMs); ports.add(detector.port);
    }
    for (const port of ports) (operations.requireNoHolders ?? requireNoHolders)(port);
  } catch { serialReleased = false; blockers.push('startup_serial_release_unproven'); }
  if (fixtureClaim) {
    const exit = await optionalProof(root, 'fixture-exit.json'), reap = await optionalProof(root, 'fixture-reap.json');
    if (!fixtureOwner || !exit || !reap || released?.complete !== true) blockers.push('startup_fixture_release_unproven');
  }
  parts.hostReleased = ownersGone && serialReleased && (!fixtureClaim || (released?.complete === true && blockers.length === 0));
  const rounds = (await readdir(root)).filter(name => /^recovery-[1-4]-session\.json$/u.test(name)).sort();
  const round = rounds.at(-1)?.split('-')[1];
  if (round) for (const stage of ['state', 'ledger', 'original_budget', 'status', 'diagnostics', 'closed', 'finished']) {
    const value = await optionalProof(root, `recovery-${round}-${stage}.json`); if (value !== undefined) parts.recovery[stage] = value;
  }
  let result;
  try { result = await inspect(['inspect', '--private-root', root]); result.blockers = []; }
  catch (error) { result = { schema: 'str005-heartbeat-result-v1', complete: false, blockers: [/^heartbeat_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'heartbeat_evidence_unverified'], parityPromotion: false }; } result.blockers.push(...blockers); result.complete &&= blockers.length === 0;
  const ownerFailure = await optionalProof(root, 'first-failure.json');
  result.owner_first_failure = ownerFailure ? validateFailure(ownerFailure) : null;
  if (ownerFailure) { result.blockers.push('startup_owner_failed'); result.complete = false; }
  result.first_failure = parts.run?.firstFailure ?? (await optionalProof(root, 'clear-failure.json'))?.firstFailure ?? null;
  if (!ownersGone) { await writeNew(resolve(root, `finish-blocker-${Date.now()}.json`), result); return { ...result, sealed: false }; }
  await writeNew(resolve(root, 'result.json'), result);
  await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) });
  return { ...result, sealed: true };
}
