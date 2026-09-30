import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { SCENARIOS } from './profiles.mjs';
import { runOne, runCorpus } from './runner.mjs';

export function parseArguments(args) {
  const [command, ...rest] = args;
  if (!['run', 'qualify', 'preflash'].includes(command)) throw Error('virtual_command');
  const options = { command };
  while (rest.length) {
    const key = rest.shift(), value = rest.shift();
    if (!['--scenario', '--backend', '--seed', '--manifest', '--evidence-dir'].includes(key) || !value || options[key]) throw Error('virtual_arguments');
    options[key] = value;
  }
  if (!options['--evidence-dir']) throw Error('virtual_evidence_required');
  options.seed = Number(options['--seed'] ?? 1);
  if (!Number.isSafeInteger(options.seed) || options.seed < 0) throw Error('virtual_seed');
  options['--backend'] ??= 'host';
  if (!['host', 'qemu'].includes(options['--backend'])) throw Error('virtual_backend');
  if (command === 'run' && !SCENARIOS.includes(options['--scenario'])) throw Error('virtual_scenario');
  if (command === 'preflash' && !options['--manifest']) throw Error('production_manifest_required');
  return options;
}
export async function main(binary, args) {
  process.umask(0o077);
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const options = parseArguments(args);
  return options.command === 'run' ? runOne(resolve(binary), repo, options) : runCorpus(resolve(binary), repo, options);
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv[2], process.argv.slice(3)).then(value => { console.log(JSON.stringify(value)); if (!['passed'].includes(value.status)) process.exitCode = 1; })
    .catch(error => { console.error(JSON.stringify({ status: 'blocked', category: error.message, device_effects: false })); process.exitCode = 1; });
}
