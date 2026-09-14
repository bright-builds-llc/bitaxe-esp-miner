import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { auditTelemetryStack } from './telemetry-stack-audit.mjs';
const [elf, objdump, sdkconfig] = process.argv.slice(2);
try {
  if (!elf || !objdump || !sdkconfig || process.argv.length !== 5) throw Error('telemetry_audit_arguments');
  const bytes = await readFile(elf);
  const disassembly = execFileSync(objdump, ['-Cd', elf], { encoding: 'utf8', timeout: 30000, maxBuffer: 128 * 1024 * 1024, stdio: ['ignore','pipe','pipe'] });
  const hash = value => createHash('sha256').update(value).digest('hex');
  if (hash(await readFile(elf)) !== hash(bytes)) throw Error('telemetry_elf_changed');
  const configuration = await readFile(sdkconfig, 'utf8');
  const result = auditTelemetryStack(disassembly, configuration);
  console.log(JSON.stringify({ ...result, elf_sha256: hash(bytes), stack_configuration_sha256: hash(configuration) }));
  if (result.result !== 'targeted_path_fits') process.exitCode = 1;
} catch (error) {
  const reason = /^telemetry_[a-z0-9_]+$/.test(error.message) ? error.message : 'telemetry_audit_unavailable';
  console.log(JSON.stringify({ schema: 'telemetry-stack-audit-v1', result: 'unproven', reason, complete_callgraph_bound: false, hardware_safety_verified: false }));
  process.exitCode = 1;
}
