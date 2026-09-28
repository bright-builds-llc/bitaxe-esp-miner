import { preflight } from './preflight.mjs';
import { beginAdmissionVerification } from './runtime-admission.mjs';
import { routePolicy } from './routes.mjs';

import { fstatSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { git, cleanPushed, ignored, missing, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, writeNew } from '../str005-noise-serial/files.mjs';
import { requireNoHolders, processSnapshot } from '../str005-v2-serial/host-resources.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { HARDWARE_ENABLED, CONTRACT, argumentsFor, requireEnabled, requireFrozenSource } from './contract.mjs';
import { createServerOwner } from '../str005-startup-probe/server.mjs';
import { finalize } from './finish.mjs';

async function currentSource(root, effect) {
  if (effect) check(HARDWARE_ENABLED, 'startup_hardware_disabled');
  const source = git(root, ['rev-parse', 'HEAD']); cleanPushed(root, source);
  const tasks = await readFile(resolve(root, 'TASKS.md'), 'utf8');
  const admissionSha256 = effect ? requireEnabled(tasks) : undefined;
  return { source_commit: source, admissionSha256, contractSha256: await fileDigest(resolve(root, CONTRACT)) };
}
export async function main(argv) {
  const { action, options } = argumentsFor(argv);
  // Reject before touching private inputs, authority directories, discovery or subprocesses.
  if (action !== 'finish') check(HARDWARE_ENABLED, 'startup_hardware_disabled');
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  const root = options['--private-root']; ignored(firmwareRoot, root);
  const source = await currentSource(firmwareRoot, action !== 'finish');
  if (action === 'preflight') return preflight(firmwareRoot, root, options, source);
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-share-context-v1' && context.admission === 'qualified-renew-share-v1' && context.source_commit === source.source_commit && context.contractSha256 === source.contractSha256,
    'startup_source_changed');
  if (action === 'finish') return finalize(root, context);
  const initialManifest = await proof(root, 'admission.json');
  check(initialManifest.sha256 === context.admissionSha256, 'share_admission_changed');
  const verifyAdmission = await beginAdmissionVerification(firmwareRoot, initialManifest.value);
  const verify = async () => {
    const current = await currentSource(firmwareRoot, true);
    requireFrozenSource(current, context);
    cleanPushed(context.gate_root, context.gate_commit);
    check(await fileDigest(context.fixture_binary) === context.fixture_sha256 && await fileDigest(context.fixture_receipt_path) === context.fixture_receipt_sha256, 'startup_tool_changed');
    const manifest = await proof(root, 'admission.json');
    check(manifest.sha256 === context.admissionSha256, 'share_admission_changed');
    await verifyAdmission();
  };
  await verify();
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'startup_private_output');
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, 'startup_distinct_output');
  requireNoHolders(context.detector.port); await missing(resolve(root, 'server-owner.json'));
  const assets = {};
  for (const [key, hash] of Object.entries(context.assetHashes)) { const bytes = await readFile(resolve(root, `gate-${key}`)); check(sha256(bytes) === hash, 'startup_asset_changed'); assets[key] = bytes; }
  assets.trust = JSON.parse(assets.trust); assets.pageClient = await readFile(resolve(firmwareRoot, 'scripts/str005-share-probe/page.mjs'));
  assets.coordinator = await readFile(resolve(firmwareRoot, 'scripts/str005-startup-probe/client.mjs'));
  await missing(resolve(root, 'serve-claim.json'));
  const admissionOwner = (await processSnapshot()).find(row => row.pid === process.pid); check(admissionOwner, 'startup_server_owner');
  await writeNew(resolve(root, 'serve-claim.json'), { schema: 'str005-startup-serve-claim-v1', owner: admissionOwner, contextSha256: sha256(JSON.stringify(context)) });
  const extraAssets = new Map();
  for (const [url, file] of [['/shared-page.mjs', 'str005-startup-probe/page.mjs'], ['/share-client.mjs', 'str005-share-probe/client.mjs']])
    extraAssets.set(url, await readFile(resolve(firmwareRoot, 'scripts', file)));
  const server = await createServerOwner({ root, context, assets, authorityDirectory: options['--authority-directory'], verify }, {
    routePolicy: routePolicy(root), extraAssets,
    admitContext: value => check(value.schema === 'str005-share-context-v1' && value.admission === 'qualified-renew-share-v1', 'share_context'),
    finishFixture: fixture => fixture.finish(),
  });
  let maybeClosing;
  const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    server.listen(0, '127.0.0.1'); await once(server, 'listening');
    const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'startup_server_owner');
    await writeNew(resolve(root, 'server-owner.json'), { owner, port: server.address().port });
    process.stdout.write(`startup_url=http://127.0.0.1:${server.address().port}/\n`);
    await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch(error => { process.stdout.write(`${JSON.stringify({ complete: false, blocker: /^(?:startup|share)_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'startup_operation_rejected' })}\n`); process.exitCode = 1; });
