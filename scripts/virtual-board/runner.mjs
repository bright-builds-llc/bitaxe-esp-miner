import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join, dirname, resolve } from 'node:path';
import { runPrivate } from '../virtual-emulator/process.mjs';
import { newEvidenceRoot, doctor } from '../virtual-emulator/setup.mjs';
import { buildGuest } from '../virtual-emulator/build.mjs';
import { runGuest } from '../virtual-emulator/run.mjs';
import { SCENARIOS, SEEDS, REQUIRED_COVERAGE, VERSION, qualify } from './profiles.mjs';
import { sourceBindings, packageBinding, sha } from './bindings.mjs';
import { lifecycle } from './lifecycle.mjs';
import { nativeAudits } from './audits.mjs';
import { componentCoverage, liveWriters } from './coverage.mjs';
import { compilerBindings, classifyHost } from './compiler-bindings.mjs';

export async function writeJson(path, value) {
  await writeFile(path, JSON.stringify(value, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
}
async function hostRun(binary, repo, scenario, seed, root, maybeManifest) {
  await mkdir(root, { mode: 0o700 });
  const evidence = join(root, 'execution');
  const args = ['run', '--scenario', scenario, '--seed', String(seed), '--evidence-dir', evidence, '--workspace-root', repo];
  if (maybeManifest) args.push('--manifest', maybeManifest);
  const compiler = await compilerBindings(repo);
  const binarySha = sha(await readFile(binary));
  const invokedSource = await sourceBindings(repo);
  const invokedPackage = maybeManifest ? await packageBinding(maybeManifest, repo) : null;
  const life = await lifecycle({
    execute: () => runPrivate(binary, args, root, 'host', { cwd: repo, timeoutMs: 30000 }),
    collect: [['result', async () => JSON.parse(await readFile(join(evidence, 'result.json'), 'utf8'))],
      ['completion', async () => JSON.parse(await readFile(join(root, 'host.process.json'), 'utf8'))]],
    release: async () => JSON.parse(await readFile(join(root, 'host.process.json'), 'utf8')).released === true,
    seal: result => writeJson(join(root, 'lifecycle.json'), result),
  }, 35000);
  const envelope = life.facts.find(item => item.phase === 'result')?.value;
  const completion = life.facts.find(item => item.phase === 'completion')?.value;
  const status = classifyHost(envelope, life, { completion, scenario, seed, binarySha, compiler, source: invokedSource, production: invokedPackage });
  return { backend: 'host', scenario, seed, status, result: envelope?.result ?? null, result_sha256: envelope ? sha(JSON.stringify(envelope)) : null,
    current_resources_released: life.current_resources_released, earliest_failure: life.earliest_failure };
}

export async function runOne(binary, repo, options) {
  const root = resolve(repo, options['--evidence-dir']);
  if (options['--backend'] === 'qemu') {
    if (!options['--manifest']) throw Error('virtual_manifest_required');
    await newEvidenceRoot(root);
    const target = await runGuest(repo, resolve(repo, options['--manifest']), root, { scenario: options['--scenario'], seed: options.seed });
    return { status: target.status, device_effects: false };
  }
  await newEvidenceRoot(root);
  const row = await hostRun(binary, repo, options['--scenario'], options.seed, join(root, 'host'), options['--manifest'] && resolve(repo, options['--manifest']));
  await writeJson(join(root, 'run-result.json'), row);
  return { status: row.status, device_effects: false };
}

/** Always saves local facts; unsupported integration paths remain required blockers. */
export async function runCorpus(binary, repo, options) {
  const root = resolve(repo, options['--evidence-dir']);
  await newEvidenceRoot(root);
  const rows = [], coverage = REQUIRED_COVERAGE.map(id => ({ id, status: 'unsupported', category: 'integration_not_qualified' }));
  const initial = await sourceBindings(repo);
  let manifestPath = options['--manifest'] && resolve(repo, options['--manifest']);
  let production = null, targetManifest = null;
  const preparation = [];
  let audits = [], targetStopped = false;
  try {
    // Qualification's default and preflash both build through the canonical graph.
    if (!manifestPath || options.command === 'preflash') {
      await runPrivate('bazel', ['build', '//firmware/bitaxe:firmware_image'], root, 'package', { cwd: repo, timeoutMs: 1200000 });
      manifestPath ??= join(repo, 'bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json');
    }
    production = await packageBinding(manifestPath, repo);
    if (production.source_commit !== initial.source_commit || production.source_dirty) preparation.push({ status: 'failed', category: 'production_source_binding' });
    await mkdir(join(root, 'native-audits'), { mode: 0o700 });
    audits = await nativeAudits(repo, manifestPath, production, join(root, 'native-audits'));
    preparation.push(...audits.filter(row => row.status !== 'passed').map(row => ({ status: 'failed', category: row.category, audit: row.id })));
    await mkdir(join(root, 'emulator-doctor'), { mode: 0o700 });
    await doctor(repo, join(root, 'emulator-doctor'));
    await mkdir(join(root, 'guest-build'), { mode: 0o700 });
    await buildGuest(repo, join(root, 'guest-build'), manifestPath);
    targetManifest = join(root, 'guest-build/virtual-package.json');
  } catch (error) { preparation.push({ status: 'failed', category: 'package_or_guest_preparation', detail: error.message }); }
  for (const scenario of SCENARIOS) for (const seed of SEEDS) {
    try { rows.push(await hostRun(binary, repo, scenario, seed, join(root, `host-${scenario}-${seed}`), manifestPath)); }
    catch { rows.push({ backend: 'host', scenario, seed, status: 'failed', category: 'host_boundary' }); }
  }
  await componentCoverage(repo, root, null, coverage);
  // Establish host coverage before spending target attempts. Component probes have
  // their own published contracts; missing host integration never admits a retry.
  const hostReady = rows.every(row => row.status === 'passed')
    && coverage.filter(row => !row.id.startsWith('target-') && row.id !== 'host-target-semantic-agreement').every(row => row.status === 'passed');
  for (const scenario of SCENARIOS) for (const seed of SEEDS) {
    if (!hostReady || !targetManifest || targetStopped) rows.push({ backend: 'qemu', scenario, seed, status: 'unsupported',
      category: !hostReady ? 'host_required_coverage_incomplete' : targetStopped ? 'target_stopped_after_failure' : 'guest_not_built' });
    else {
      const targetRoot = join(root, `qemu-${scenario}-${seed}`);
      try {
        await mkdir(targetRoot, { mode: 0o700 });
        const result = await runGuest(repo, targetManifest, targetRoot, { scenario, seed });
        if (result.unexpected_panic || result.missing_events.length || result.checks.some(check => check.status === 'failed')) targetStopped = true;
        rows.push({ backend: 'qemu', scenario, seed, status: result.status, result: result.actual_target_measurements.find(event => event.event === 'scenario')?.result ?? null });
      } catch { targetStopped = true; rows.push({ backend: 'qemu', scenario, seed, status: 'failed', category: 'target_boundary' }); }
    }
  }
  if (hostReady && !targetStopped) await componentCoverage(repo, root, targetManifest, coverage);
  const agreements = rows.filter(row => row.backend === 'host').every(host => {
    const target = rows.find(row => row.backend === 'qemu' && row.scenario === host.scenario && row.seed === host.seed);
    return host.status === 'passed' && target?.status === 'passed' && sha(JSON.stringify(host.result)) === sha(JSON.stringify(target.result));
  });
  const comparisonsAvailable = rows.filter(row => row.backend === 'host').every(host => rows.some(target => target.backend === 'qemu' && target.scenario === host.scenario && target.seed === host.seed && target.result));
  Object.assign(coverage.find(row => row.id === 'host-target-semantic-agreement'), {
    status: comparisonsAvailable ? agreements ? 'passed' : 'failed' : 'unsupported',
    category: comparisonsAvailable ? 'compared_complete_results' : 'target_results_unavailable',
  });
  const final = await sourceBindings(repo);
  if (final.source_sha256 !== initial.source_sha256) preparation.push({ status: 'failed', category: 'source_changed_during_validation' });
  if (production && sha(JSON.stringify(await packageBinding(manifestPath, repo))) !== sha(JSON.stringify(production))) preparation.push({ status: 'failed', category: 'package_changed_during_validation' });
  const bindings = { ...initial, production };
  const decision = qualify(rows, coverage, bindings);
  if (preparation.length) { decision.qualified = false; decision.mode = 'observation'; decision.blockers.push(...preparation); }
  const writers = await liveWriters(root);
  if (writers.length) { decision.qualified = false; decision.mode = 'observation'; decision.blockers.push({ category: 'producer_release_unproven', count: writers.length }); }
  const report = { schema: VERSION, command: options.command, ...decision, bindings, rows, coverage, preparation, native_audits: audits,
    parity: '90/95', device_effects: false, package_frozen: false, rollout: 'observation_only_until_full_qualification' };
  // No frozen physical candidate is produced from incomplete qualification.
  if (report.qualified) {
    const frozen = join(root, 'frozen-package'); await mkdir(frozen, { mode: 0o700 });
    for (const artifact of production.artifacts) {
      const output = join(frozen, artifact.path);
      await mkdir(dirname(output), { recursive: true, mode: 0o700 });
      await writeFile(output, await readFile(artifact.source_path), { flag: 'wx', mode: 0o600 });
      if (sha(await readFile(output)) !== artifact.sha256) throw Error('frozen_artifact_changed');
    }
    await writeFile(join(frozen, 'package.json'), await readFile(manifestPath), { flag: 'wx', mode: 0o600 });
    await writeFile(join(frozen, 'bitaxe-firmware.sdkconfig'), await readFile(join(dirname(manifestPath), 'bitaxe-firmware.sdkconfig')), { flag: 'wx', mode: 0o600 });
    if (sha(await readFile(join(frozen, 'package.json'))) !== production.manifest_sha256
        || sha(await readFile(join(frozen, 'bitaxe-firmware.sdkconfig'))) !== production.sdkconfig_sha256) throw Error('frozen_metadata_changed');
    report.package_frozen = true;
  }
  const reportName = writers.length ? 'partial-validation-report.json' : 'validation-report.json';
  await writeJson(join(root, reportName), { ...report, sealed: writers.length === 0 });
  if (!writers.length) await writeFile(join(root, 'validation-report.sha256'), sha(await readFile(join(root, reportName))) + '\n', { flag: 'wx', mode: 0o600 });
  return { qualified: report.qualified, status: report.qualified ? 'passed' : 'blocked', blocker_count: report.blockers.length, device_effects: false };
}
