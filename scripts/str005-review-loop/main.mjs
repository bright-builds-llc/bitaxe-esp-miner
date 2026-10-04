import { fstatSync } from 'node:fs';
import { mkdir, readFile, readdir, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { BUNDLE, PAGE, cleanPushed, git, ignored, missing, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { inventory, privateRoot, proof, retain, writeNew } from '../str005-noise-serial/files.mjs';
import { processSnapshot, requireLsofAbsent, requireNoHolders, sameProcess } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { requireEnabled } from '../str005-share-recovery/contract.mjs';
import { CONTROL_DIAGNOSTIC_RECOVERY, loadInstallPredecessor } from '../str005-panic-recovery/control-diagnostic.mjs';
import { createLoopServer } from './server.mjs';

export const ENABLED = true;
export const PROFILE = Object.freeze({ ...CONTROL_DIAGNOSTIC_RECOVERY, enabled: ENABLED, lines: ['Control review loop hardware: enabled.'] });
// The origin that holds the Ultra 205 Web Serial grant; another port would show the chooser.
export const PORT = 48765;

export function argumentsFor(argv, enabled = ENABLED) {
  const [action, ...args] = argv;
  check(['preflight', 'serve', 'finish'].includes(action) && args.length % 2 === 0, 'review_loop_arguments');
  if (action !== 'finish') check(enabled, 'review_loop_disabled');
  const allowed = action === 'preflight' ? ['--private-root', '--predecessor-root', '--gate-root'] : ['--private-root'], options = {};
  for (let i = 0; i < args.length; i += 2) {
    check(allowed.includes(args[i]) && !options[args[i]] && typeof args[i + 1] === 'string' && resolve(args[i + 1]) === args[i + 1], 'review_loop_arguments');
    options[args[i]] = args[i + 1];
  }
  check(allowed.every(key => options[key]), 'review_loop_arguments'); return { action, options };
}
async function source(root, effect = true) {
  const source_commit = git(root, ['rev-parse', 'HEAD']); cleanPushed(root, source_commit);
  if (effect) requireEnabled(await readFile(resolve(root, 'TASKS.md'), 'utf8'), PROFILE.enabled, PROFILE);
  return { source_commit, contractSha256: sha256(await readFile(resolve(root, PROFILE.contract))) };
}
async function detect(root, physical, final = false) {
  const path = resolve(dirname(root), `${final ? 'final-' : ''}detector.stdout.log`); await protectedPath(path);
  return parseDetector(await readFile(path, 'utf8'), physical, Date.now() - (await stat(path)).mtimeMs);
}

export async function main(argv) {
  const { action, options } = argumentsFor(argv);
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  const root = options['--private-root']; ignored(firmwareRoot, root);
  const current = await source(firmwareRoot, action !== 'finish');
  if (action === 'preflight') {
    await missing(root); await privateRoot(dirname(root)); ignored(firmwareRoot, options['--predecessor-root']);
    const prior = (await loadInstallPredecessor(firmwareRoot, options['--predecessor-root'])).context;
    const gateRoot = options['--gate-root']; cleanPushed(gateRoot, prior.gate_commit);
    const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
      trust: await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json')) };
    for (const [key, bytes] of Object.entries(assets)) check(sha256(bytes) === prior.assetHashes[key], 'review_loop_asset');
    const context = { schema: 'str005-review-loop-context-v1', ...current, firmware_root: firmwareRoot, gate_root: gateRoot,
      installRoot: options['--predecessor-root'], scope: 'share', ...prior };
    await mkdir(root, { mode: 0o700 }); await writeNew(resolve(root, 'context.json'), context);
    for (const [key, bytes] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), bytes);
    return { preflight: 'passed', device_effects: false };
  }
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-review-loop-context-v1' && context.source_commit === current.source_commit &&
    context.contractSha256 === current.contractSha256, 'review_loop_source_changed');
  if (action === 'finish') return finish(root, context);
  const verify = async () => { const live = await source(firmwareRoot); check(live.source_commit === context.source_commit, 'review_loop_source_changed');
    cleanPushed(context.gate_root, context.gate_commit); };
  await verify(); const selected = await detect(root, context.physical); requireNoHolders(selected.port);
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'review_loop_private_output');
  await missing(resolve(root, 'server-owner.json'));
  const assets = { modules: {} };
  for (const [key, hash] of Object.entries(context.assetHashes)) { const bytes = await readFile(resolve(root, `gate-${key}`)); check(sha256(bytes) === hash, 'review_loop_asset'); assets[key] = bytes; }
  assets.trust = JSON.parse(assets.trust);
  for (const name of ['loop-page.mjs', 'loop.mjs']) assets.modules[`/${name}`] = await readFile(resolve(firmwareRoot, 'scripts/str005-review-loop', name === 'loop-page.mjs' ? 'page.mjs' : name));
  const server = createLoopServer({ root, context, assets, verify }); let maybeClosing;
  const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    requireLsofAbsent(['-nP', `-iTCP:${PORT}`, '-sTCP:LISTEN', '-t']);
    server.listen(PORT, '127.0.0.1'); await once(server, 'listening');
    const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'review_loop_owner');
    await writeNew(resolve(root, 'server-owner.json'), { owner, port: PORT, serialPort: selected.port });
    process.stdout.write(`review_loop_url=http://127.0.0.1:${PORT}/\n`);
    await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true };
}
/** Seals the rows with proven host release; a device panic is judged later by recovery, not here. */
export async function finish(root, context) {
  const owner = (await proof(root, 'server-owner.json')).value;
  check(!(await processSnapshot()).some(row => sameProcess(row, owner.owner)), 'review_loop_owner_live');
  requireLsofAbsent(['-nP', `-iTCP:${owner.port}`, '-sTCP:LISTEN', '-t']);
  const selected = await detect(root, context.physical, true);
  for (const port of new Set([owner.serialPort, selected.port])) requireNoHolders(port);
  const names = (await readdir(root)).filter(name => /^loop-row-\d{3}\.json$/u.test(name)).sort();
  const rows = []; for (const name of names) rows.push((await proof(root, name)).value);
  const last = rows.at(-1) ?? null, complete = rows.filter(row => row.kind === 'complete');
  const result = { schema: 'str005-review-loop-result-v2', rows: rows.length, batches_completed: complete.length,
    rounds_completed: complete.reduce((sum, row) => sum + row.completed, 0) + (last?.kind === 'failure' ? last.completed : 0),
    terminal: last?.kind ?? 'none',
    failure: last?.kind === 'failure' ? { batch: last.batch, iteration: last.iteration, operation: last.operation, category: last.category } : null,
    host_released: true, device_panic_judged_by_recovery: true };
  await writeNew(resolve(root, 'result.json'), result);
  await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch(error => { process.stdout.write(`${JSON.stringify({ error: /^review_loop_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'review_loop_failed' })}\n`); process.exitCode = 1; });
