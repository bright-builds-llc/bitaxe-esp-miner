import { fstatSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { once } from 'node:events';
import { fileURLToPath } from 'node:url';
import { git, cleanPushed, ignored, missing, protectedPath, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, writeNew } from '../str005-noise-serial/files.mjs';
import { processSnapshot, requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { createServerOwner } from '../str005-startup-probe/server.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { argumentsFor, source } from './contract.mjs';
import { preflight, lineage } from './preflight.mjs';
import { capturePackage } from '../str005-share-diagnostic/prepare.mjs';
import { routePolicy } from './policy.mjs';
import { finalize } from './finish.mjs';

export async function main(argv) {
  const { action, options } = argumentsFor(argv), root = options['--private-root'];
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  ignored(repo, root);
  const current = await source(repo);
  if (action === 'preflight') return preflight(repo, root, options, current);
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  const verify = async () => {
    const live = await source(repo);
    check(context.schema === 'str005-status-repro-context-v1' && context.admission === 'diagnostic-status-v1' &&
      context.source_commit === live.commit && context.contractSha256 === live.contractSha256, 'status_repro_source_changed');
    const anchors = context.anchors;
    const chain = await lineage(repo, { '--installation-root': anchors.installation.root, '--capture-root': anchors.capture.root,
      '--clear-root': anchors.clear.root, '--recovery-root': anchors.recovery.root }, live);
    check(chain.installed.seal === anchors.installation.seal && chain.captured.seal === anchors.capture.seal &&
      chain.cleared.seal === anchors.clear.seal && chain.recovered.seal === anchors.recovery.seal &&
      chain.identity.firmware_commit === context.firmware_commit && chain.identity.app_elf_sha256 === context.app_elf_sha256 &&
      chain.physical === context.physical && chain.status.observation.bootOrdinal === context.expectedBootOrdinal,
    'status_repro_lineage_changed');
    cleanPushed(context.gate_root, context.gate_commit);
    const retained = await capturePackage(context.retainedManifest, chain.installed, repo);
    check(retained.candidateElf === context.candidateElf &&
      retained.retainedManifestSha256 === context.retainedManifestSha256, 'status_repro_package_changed');
    check(await fileDigest(context.fixture_binary) === context.fixture_sha256 &&
      await fileDigest(context.fixture_receipt_path) === context.fixture_receipt_sha256 &&
      await fileDigest(context.candidateElf) === context.app_elf_sha256, 'status_repro_tools_changed');
    for (const [name, hash] of Object.entries(context.auditSha256)) {
      const audit = await proof(root, `${name}-audit.json`);
      check(audit.sha256 === hash && audit.value.elf_sha256 === context.app_elf_sha256, 'status_repro_audit_changed');
    }
    check((await proof(root, 'gate-compatibility.json')).sha256 === context.gateCompatibilitySha256,
      'status_repro_gate_compatibility_changed');
  };
  await verify();
  if (action === 'finish') return finalize(root, context);
  check(action === 'serve', 'status_repro_action');
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'status_repro_private_output');
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, 'status_repro_distinct_output');
  requireNoHolders(context.detector.port); await missing(resolve(root, 'server-owner.json'));
  const assets = {};
  for (const [name, hash] of Object.entries(context.assetHashes)) {
    const path = resolve(root, `gate-${name}`); await protectedPath(path);
    const bytes = await readFile(path); check(sha256(bytes) === hash, 'status_repro_assets_changed'); assets[name] = bytes;
  }
  assets.trust = JSON.parse(assets.trust);
  assets.pageClient = await readFile(resolve(repo, 'scripts/str005-status-repro/page.mjs'));
  assets.coordinator = await readFile(resolve(repo, 'scripts/str005-status-repro/client.mjs'));
  const extraAssets = new Map();
  for (const [url, file] of [['/base-startup-page.mjs','str005-startup-probe/page.mjs'],
    ['/base-client.mjs','str005-startup-probe/client.mjs'], ['/client-core.mjs','str005-status-repro/client-core.mjs']])
    extraAssets.set(url, await readFile(resolve(repo, 'scripts', file)));
  const owner = (await processSnapshot()).find(row => row.pid === process.pid);
  check(owner, 'status_repro_owner');
  await writeNew(resolve(root, 'serve-claim.json'), { schema: 'str005-startup-serve-claim-v1', owner,
    contextSha256: sha256(JSON.stringify(context)) });
  const server = await createServerOwner({ root, context, assets, authorityDirectory: options['--authority-directory'], verify }, {
    routePolicy: routePolicy(), extraAssets, finishFixture: fixture => fixture.finish(),
    admitContext: value => check(value.schema === 'str005-status-repro-context-v1' && value.admission === 'diagnostic-status-v1', 'status_repro_context'),
  });
  let maybeClosing;
  const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    server.listen(0, '127.0.0.1'); await once(server, 'listening');
    const running = (await processSnapshot()).find(row => row.pid === process.pid);
    check(running, 'status_repro_owner');
    await writeNew(resolve(root, 'server-owner.json'), { owner: running, port: server.address().port });
    process.stdout.write(`status_repro_url=http://127.0.0.1:${server.address().port}/\n`);
    await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value =>
  process.stdout.write(`${JSON.stringify(value)}\n`)).catch(error => {
  process.stdout.write(`${JSON.stringify({ error: /^status_repro_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'status_repro_failed' })}\n`);
  process.exitCode = 1;
});
