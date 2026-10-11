// Builds the pinned Gate acceptance page and bundle, and the no-mining context that serves them.
import { execFile } from 'node:child_process';
import { copyFile, mkdir, readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { BUNDLE, digest, PAGE, REQUIRED_CYCLES } from '../../fixed-usb-qualification/contract.mjs';
import { NO_MINING_SCHEMA } from '../../fixed-usb-qualification/no-mining-context.mjs';

const run = promisify(execFile);
const SUPERVISOR_CLIENT = resolve(dirname(fileURLToPath(import.meta.url)), '../../fixed-usb-qualification/no-mining-client.mjs');
/** The device's firmware identity (`crates/bitaxe-simulation/src/controller.rs`). */
export const DEVICE_IDENTITY = Object.freeze({ firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64) });

/** The Gate source commit `MODULE.bazel` pins for `bwg_gate_contract`. */
export async function pinnedGateCommit(moduleBazel) {
  const match = /bitaxe-turnstile-system-([0-9a-f]{40})/u.exec(await readFile(moduleBazel, 'utf8'));
  if (!match) throw Error('gate_pin_missing');
  return match[1];
}

/**
 * Builds the page bundle like the Gate's own `build-worker-acceptance.ts`, except the
 * source commit comes from the pin, because the Bazel archive is not a git checkout.
 */
export async function buildGateTree({ gateSource, pagesSource, commit, gateRoot }) {
  const outdir = resolve(gateRoot, dirname(BUNDLE));
  await mkdir(outdir, { recursive: true, mode: 0o700 });
  // An exact output file: under Bazel the entry is a runfiles copy whose output name would differ.
  await run('bun', ['build', resolve(gateSource, 'web/worker-serial-acceptance.ts'), '--target=browser', '--format=esm',
    `--outfile=${resolve(gateRoot, BUNDLE)}`, '--define', `BWG_GATE_SOURCE_COMMIT=${JSON.stringify(commit)}`], { timeout: 120000 });
  const bundle = await readFile(resolve(gateRoot, BUNDLE));
  if (!bundle.includes(commit)) throw Error('gate_bundle_stale');
  await mkdir(resolve(gateRoot, dirname(PAGE)), { recursive: true, mode: 0o700 });
  await copyFile(resolve(pagesSource, 'acceptance.html'), resolve(gateRoot, PAGE));
}

/** A firmware tree holding only the deployment trust the supervisor publishes. */
export async function buildFirmwareTree({ trust, firmwareRoot }) {
  const target = resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json');
  await mkdir(dirname(target), { recursive: true, mode: 0o700 });
  await copyFile(trust, target);
}

/**
 * A no-mining supervisor context for the virtual device. Package fields describe no
 * physical image and are marked synthetic; the asset digests are real.
 */
export async function virtualNoMiningContext({ gateRoot, firmwareRoot, commit }) {
  const read = path => readFile(path);
  return {
    schema: NO_MINING_SCHEMA,
    manifest_sha256: 'synthetic-virtual-device',
    app_elf_sha256: DEVICE_IDENTITY.app_elf_sha256,
    reference_commit: 'synthetic-virtual-device',
    artifacts: [],
    update_segments: [],
    firmware_commit: DEVICE_IDENTITY.firmware_commit,
    gate_commit: commit,
    gate_bundle_sha256: digest(await read(resolve(gateRoot, BUNDLE))),
    gate_page_relative_path: PAGE,
    gate_page_sha256: digest(await read(resolve(gateRoot, PAGE))),
    trust_sha256: digest(await read(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json'))),
    supervisor_client_sha256: digest(await read(SUPERVISOR_CLIENT)),
    required_no_mining_cycles: REQUIRED_CYCLES,
    mining_authorized: false,
    firmware_root: firmwareRoot,
    gate_root: gateRoot,
    manifest: 'synthetic-virtual-device',
  };
}
