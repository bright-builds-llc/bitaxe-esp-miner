import { fstatSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { once } from 'node:events';
import { fileURLToPath } from 'node:url';
import { git, cleanPushed, ignored, missing, protectedPath, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, writeNew } from '../str005-noise-serial/files.mjs';
import { processSnapshot, requireLsofAbsent, requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { createServerOwner } from '../str005-startup-probe/server.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { preflight } from '../str005-step5-diagnostic/preflight.mjs';
import { installation, previousStart, restartAfter } from '../str005-step5-diagnostic/lineage.mjs';
import { validateRun } from '../str005-heartbeat-probe/evidence.mjs';
import { createHeartbeatOwner } from '../str005-heartbeat-probe/owner.mjs';
import { finalize } from '../str005-heartbeat-probe/finish.mjs';
import { observerContext } from './observer-context.mjs';
import { ADMISSION, PINS, SCHEMA, argumentsFor, source } from './contract.mjs';

// The origin that holds the Ultra 205 Web Serial grant; another port would show the chooser.
export const PORT = 48765;
const PROFILE = Object.freeze({ pins: PINS, schema: SCHEMA, admission: ADMISSION, bundleMarkers: ['suppressHeartbeats'] });

/** Re-verifies the frozen install -> previous Start -> restart chain and the frozen tools. */
async function verifyLineage(root, context, live) {
  check(context.schema === SCHEMA && context.admission === ADMISSION &&
    context.source_commit === live.commit && context.contractSha256 === live.contractSha256, 'share_source_changed');
  const installed = await installation(context.anchors.installation.root, PINS);
  const previous = await previousStart(context.anchors.previousStart.root, installed, PINS);
  const restarted = await restartAfter(context.anchors.restart.root, previous, PINS);
  check(installed.seal === context.anchors.installation.seal && previous.seal === context.anchors.previousStart.seal &&
    restarted.seal === context.anchors.restart.seal && installed.identity.firmware_commit === context.firmware_commit &&
    installed.identity.app_elf_sha256 === context.app_elf_sha256 && installed.physical === context.physical &&
    restarted.expectedBootOrdinal === context.expectedBootOrdinal, 'share_lineage_changed');
  cleanPushed(context.gate_root, context.gate_commit);
  check(await fileDigest(context.fixture_binary) === context.fixture_sha256 &&
    await fileDigest(context.fixture_receipt_path) === context.fixture_receipt_sha256 &&
    await fileDigest(context.candidateElf) === context.app_elf_sha256, 'share_tools_changed');
  for (const [name, hash] of Object.entries(context.auditSha256)) {
    const audit = await proof(root, `${name}-audit.json`);
    check(audit.sha256 === hash && audit.value.elf_sha256 === context.app_elf_sha256, 'share_audit_changed');
  }
  check((await proof(root, 'gate-compatibility.json')).sha256 === context.gateCompatibilitySha256, 'share_gate_compatibility_changed');
  check(await fileDigest(context.cadence_observer.path) === context.cadence_observer.sha256 &&
    await fileDigest(context.observer_receipt_path) === context.observer_build_receipt_sha256, 'heartbeat_observer_changed');
}

export async function main(argv) {
  const { action, options } = argumentsFor(argv), root = options['--private-root'];
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  ignored(repo, root);
  const current = await source(repo);
  if (action === 'preflight') return preflight(repo, root, options, current, { extraContext: observerContext }, PROFILE);
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  const verify = async () => verifyLineage(root, context, await source(repo));
  await verify();
  if (action === 'finish') return finalize(root, context);
  check(action === 'serve', 'share_action');
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'share_private_output');
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, 'share_distinct_output');
  requireNoHolders(context.detector.port); await missing(resolve(root, 'server-owner.json'));
  const assets = {};
  for (const [name, hash] of Object.entries(context.assetHashes)) {
    const path = resolve(root, `gate-${name}`); await protectedPath(path);
    const bytes = await readFile(path); check(sha256(bytes) === hash, 'share_assets_changed'); assets[name] = bytes;
  }
  assets.trust = JSON.parse(assets.trust);
  assets.pageClient = await readFile(resolve(repo, 'scripts/str005-heartbeat-probe/page.mjs'));
  assets.coordinator = await readFile(resolve(repo, 'scripts/str005-startup-probe/client.mjs'));
  const extraAssets = new Map();
  for (const [url, file] of [['/shared-page.mjs', 'str005-startup-probe/page.mjs'], ['/heartbeat-client.mjs', 'str005-heartbeat-probe/client.mjs'],
    ['/limits.mjs', 'str005-heartbeat-probe/limits.mjs'], ['/heartbeat-page-options.mjs', 'str005-heartbeat-probe/page-options.mjs']]) {
    const text = await readFile(resolve(repo, 'scripts', file));
    extraAssets.set(url, file.endsWith('page-options.mjs') ? Buffer.from(text.toString('utf8').replace("'./client.mjs'", "'/heartbeat-client.mjs'")) : text);
  }
  const owner = (await processSnapshot()).find(row => row.pid === process.pid);
  check(owner, 'share_owner');
  await writeNew(resolve(root, 'serve-claim.json'), { schema: 'str005-startup-serve-claim-v1', owner,
    contextSha256: sha256(JSON.stringify(context)) });
  const heartbeat = createHeartbeatOwner(root, context, { verify });
  // Zero renewals and the default normal signing; no share is required, so the fixture is only released.
  const server = await createServerOwner({ root, context, assets, authorityDirectory: options['--authority-directory'], verify }, {
    routePolicy: { extraRoute: heartbeat.extraRoute, validateRun }, extraAssets, release: heartbeat.release,
    admitContext: value => check(value.schema === SCHEMA && value.admission === ADMISSION, 'share_context'),
  });
  let maybeClosing;
  const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    requireLsofAbsent(['-nP', `-iTCP:${PORT}`, '-sTCP:LISTEN', '-t']);
    server.listen(PORT, '127.0.0.1'); await once(server, 'listening');
    const running = (await processSnapshot()).find(row => row.pid === process.pid);
    check(running, 'share_owner');
    await writeNew(resolve(root, 'server-owner.json'), { owner: running, port: server.address().port });
    process.stdout.write(`heartbeat_url=http://127.0.0.1:${server.address().port}/\n`);
    await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value =>
  process.stdout.write(`${JSON.stringify(value)}\n`)).catch(error => {
  process.stdout.write(`${JSON.stringify({ error: /^(?:heartbeat|share|startup|step5)_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'heartbeat_failed' })}\n`);
  process.exitCode = 1;
});
