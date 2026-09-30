import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bootstrap, doctor, newEvidenceRoot } from './setup.mjs';
import { buildGuest } from './build.mjs';
import { diagnoseHeap } from './diagnose.mjs';
import { runGuest } from './run.mjs';

export async function main(args = process.argv.slice(2)) {
  process.umask(0o077);
  const command = args.shift();
  const options = {};
  while (args.length) {
    const key = args.shift(); const value = args.shift();
    if (!['--evidence-dir', '--manifest', '--commands', '--scenario', '--seed'].includes(key) || !value || options[key]) throw Error('emulator_arguments');
    options[key] = value;
  }
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  if (!options['--evidence-dir']) throw Error('emulator_evidence_required');
  const evidence = resolve(repo, options['--evidence-dir']);
  await newEvidenceRoot(evidence);
  if (command === 'bootstrap') return bootstrap(repo, evidence);
  if (command === 'doctor') return doctor(repo, evidence);
  if (command === 'build' && options['--manifest']) return buildGuest(repo, evidence, resolve(repo, options['--manifest']));
  if (command === 'diagnose-heap' && options['--manifest']) return diagnoseHeap(repo, resolve(repo, options['--manifest']), evidence);
  if (command === 'run' && options['--manifest']) return runGuest(repo, resolve(repo, options['--manifest']), evidence,
    { commands: options['--commands']?.split(',') ?? ['status', 'allocation'], scenario: options['--scenario'], seed: Number(options['--seed'] ?? 1) });
  throw Error('emulator_arguments');
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().then(value => console.log(JSON.stringify(value))).catch(error => {
    console.error(JSON.stringify({ status: 'blocked', category: error.message, device_effects: false })); process.exitCode = 1;
  });
}
