import { mkdir, readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { runPrivate } from '../virtual-emulator/process.mjs';
import { runGuest } from '../virtual-emulator/run.mjs';

export async function componentCoverage(repo, root, targetManifest, coverage) {
  const set = (id, status, category) => Object.assign(coverage.find(row => row.id === id), { status, category });
  try {
    await runPrivate('bazel', ['test', '//crates/bitaxe-virtual-board:models_tests', '//crates/bitaxe-virtual-board:asic_tests', '//crates/bitaxe-virtual-board:memory_tests',
      '//crates/bitaxe-runtime:actuation_tests', '//crates/bitaxe-simulation:scenario_tests', '//crates/bitaxe-simulation:tests', '//tools/stratum-v2-fixture:fixture_oracle_tests', '//scripts:virtual_board_test', '//scripts:virtual_emulator_test'],
    root, 'component-tests', { cwd: repo, timeoutMs: 120000 });
    set('model-conformance-mutations', 'passed', 'independent_model_corpus_passed');
  } catch { set('model-conformance-mutations', 'failed', 'component_suite_failed'); }
  if (!targetManifest) return;
  for (const [id, commands] of [['target-reset-persistence', ['status', 'allocation', 'restart']], ['target-panic-core-decode', ['status', 'panic']]]) {
    const path = join(root, id); await mkdir(path, { mode: 0o700 });
    try {
      const result = await runGuest(repo, targetManifest, path, { commands });
      const check = result.checks.find(row => row.id === (id === 'target-reset-persistence' ? 'target_reset_persistence' : 'target_panic_core_decoding'));
      set(id, check?.status === 'passed' && result.process_released ? 'passed' : 'failed', check?.category ?? 'target_component_result');
    } catch { set(id, 'failed', 'target_component_boundary'); }
  }
}
/** A surviving producer marker forbids sealing its ancestor report. */
export async function liveWriters(root) {
  const writers = [];
  async function visit(path) {
    for (const entry of await readdir(path, { withFileTypes: true })) {
      const child = join(path, entry.name);
      if (entry.isSymbolicLink()) throw Error('evidence_symlink');
      if (entry.isDirectory()) await visit(child);
      else if (entry.name === 'writer.json' || entry.name.endsWith('.writer.json')) writers.push(child);
    }
  }
  await visit(root);
  return writers;
}
