import { readFile, stat, mkdir } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { PAGE, BUNDLE, cleanPushed, git, ignored, missing, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, writeNew, retain } from '../str005-noise-serial/files.mjs';
import { processSnapshot, sameProcess, requireNoHolders, requireLsofAbsent } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { CLOSED_FILE, ENDPOINT_FILE, createEndpointServer } from './server.mjs';

const GATE_PORT = 48765;
const ACTIONS = { preflight: ['--private-root', '--gate-root', '--package-manifest', '--detector-output'],
  serve: ['--private-root', '--detector-output'], finish: ['--private-root', '--detector-output'] };

/** `bazel run` executes in the runfiles tree, so every operator path resolves against the workspace. */
export function workspacePath(firmwareRoot, value) { return resolve(firmwareRoot, value); }

function argumentsFor(argv) {
  const [action, ...rest] = argv; check(Object.hasOwn(ACTIONS, action), 'endpoint_action');
  const options = {};
  for (let index = 0; index < rest.length; index += 2) {
    check(ACTIONS[action].includes(rest[index]) && typeof rest[index + 1] === 'string' && !Object.hasOwn(options, rest[index]), 'endpoint_option');
    options[rest[index]] = rest[index + 1];
  }
  check(ACTIONS[action].every(key => Object.hasOwn(options, key)), 'endpoint_option');
  return { action, options };
}

/** A detector output written within the last minute, admitted for the expected (or first-seen) physical device. */
async function detect(path, maybePhysical) {
  await protectedPath(path); const text = await readFile(path, 'utf8');
  const physical = maybePhysical ?? /^physical_identity_sha256: ([0-9a-f]{64})$/mu.exec(text)?.[1];
  return parseDetector(text, physical, Date.now() - (await stat(path)).mtimeMs);
}

async function preflight(firmwareRoot, root, options) {
  await missing(root); await protectedPath(dirname(root), true); ignored(firmwareRoot, root);
  const pins = [...(await readFile(resolve(firmwareRoot, 'MODULE.bazel'), 'utf8')).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(pins.length === 1, 'endpoint_gate_pin'); const gateCommit = pins[0][1];
  const gateRoot = workspacePath(firmwareRoot, options['--gate-root']); cleanPushed(gateRoot, gateCommit);
  const manifest = JSON.parse(await readFile(workspacePath(firmwareRoot, options['--package-manifest']), 'utf8'));
  check(/^[0-9a-f]{40}$/u.test(manifest.source_commit) && /^[0-9a-f]{64}$/u.test(manifest.app_elf_sha256) && manifest.build_identity?.source_dirty === false, 'endpoint_package');
  const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
    trust: await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json')) };
  check(assets.bundle.includes(gateCommit) && assets.bundle.includes('observeStationEndpoint'), 'endpoint_gate_bundle');
  const detected = await detect(workspacePath(firmwareRoot, options['--detector-output']));
  const context = { schema: 'otawww-endpoint-context-v1', gate_commit: gateCommit, firmware_commit: manifest.source_commit,
    app_elf_sha256: manifest.app_elf_sha256, physical: detected.physical,
    assetHashes: Object.fromEntries(Object.entries(assets).map(([key, bytes]) => [key, sha256(bytes)])) };
  await mkdir(root, { mode: 0o700 }); await writeNew(resolve(root, 'context.json'), context);
  for (const [key, bytes] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), bytes);
  return { preflight: 'passed', device_effects: false };
}

async function serve(firmwareRoot, root, context, options) {
  await missing(resolve(root, 'server-owner.json'));
  const selected = await detect(workspacePath(firmwareRoot, options['--detector-output']), context.physical); requireNoHolders(selected.port);
  const assets = {};
  for (const [key, hash] of Object.entries(context.assetHashes)) {
    const bytes = await readFile(resolve(root, `gate-${key}`)); check(sha256(bytes) === hash, 'endpoint_asset_changed'); assets[key] = bytes;
  }
  assets.trust = JSON.parse(assets.trust);
  assets.pageModule = await readFile(resolve(firmwareRoot, 'scripts/otawww-endpoint/page.mjs'));
  const server = createEndpointServer({ root, context, assets }); let maybeClosing;
  const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    server.listen(GATE_PORT, '127.0.0.1'); await once(server, 'listening');
    const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'endpoint_owner');
    await writeNew(resolve(root, 'server-owner.json'), { owner, port: server.address().port, serialPort: selected.port });
    process.stdout.write(`endpoint_handoff_url=http://127.0.0.1:${server.address().port}/\n`);
    await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true };
}

async function finish(firmwareRoot, root, context, options) {
  const owner = (await proof(root, 'server-owner.json')).value;
  check(!(await processSnapshot()).some(row => sameProcess(row, owner.owner) || row.ppid === owner.owner.pid), 'endpoint_owner_live');
  requireLsofAbsent(['-nP', `-iTCP:${owner.port}`, '-sTCP:LISTEN', '-t']);
  const selected = await detect(workspacePath(firmwareRoot, options['--detector-output']), context.physical);
  for (const port of new Set([owner.serialPort, selected.port])) requireNoHolders(port);
  const closed = (await proof(root, CLOSED_FILE)).value; await protectedPath(resolve(root, ENDPOINT_FILE));
  const result = { schema: 'otawww-endpoint-result-v1', endpoint_recorded: closed.endpoint_saved === true, page_closed: true, host_released: true };
  await writeNew(resolve(root, 'result.json'), result);
  check(result.endpoint_recorded, 'endpoint_not_recorded');
  return result;
}

export async function main(argv) {
  const { action, options } = argumentsFor(argv);
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  const root = workspacePath(firmwareRoot, options['--private-root']);
  if (action === 'preflight') return preflight(firmwareRoot, root, options);
  await privateRoot(root);
  const context = (await proof(root, 'context.json')).value; check(context.schema === 'otawww-endpoint-context-v1', 'endpoint_context');
  return action === 'serve' ? serve(firmwareRoot, root, context, options) : finish(firmwareRoot, root, context, options);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch(error => { process.stdout.write(`${JSON.stringify({ error: typeof error.code === 'string' ? error.code : 'endpoint_failed' })}\n`); process.exitCode = 1; });
