#!/usr/bin/env node
// Gate: every thread stack audit must find its owner and fit on the built firmware ELF.
import { execFileSync } from 'node:child_process';
import { accessSync, constants, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { auditBapStack } from './bap-stack-audit.mjs';
import { auditControlStack } from './control-stack-audit.mjs';
import { auditHttpStack } from './http-stack-audit.mjs';
import { auditLinkStack } from './link-stack-audit.mjs';
import { auditWriterStack } from './writer-stack-audit.mjs';
import { boundToFunctions, functionSizes } from './virtual-emulator/noise-stack-audit.mjs';

const PASS = 'selected_path_with_headroom';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

/** One row per audit; an audit that throws (for example a missing root) is a failed row, never skipped. */
export function stackAuditRows(disassembly, { usbOwner, preparedLink, httpServer, bapAdapter }) {
  const audits = [['control', () => auditControlStack(disassembly, usbOwner)],
    ['writer', () => auditWriterStack(disassembly, usbOwner)], ['link', () => auditLinkStack(disassembly, preparedLink)],
    ['http', () => auditHttpStack(disassembly, httpServer)], ['bap', () => auditBapStack(disassembly, bapAdapter)]];
  return audits.map(([audit, run]) => {
    try {
      const result = run();
      return { audit, result: result.result, deepest: result.deepest_normal_path_bytes, budget: result.budget_bytes,
        headroom: result.headroom_bytes, root_frame: result.root_frame ?? 'owner_symbol' };
    } catch (error) {
      return { audit, result: 'blocked', error: error.message };
    }
  });
}

/** The espup-managed objdump, found as `verify-native-usb-symbols.mjs` finds its `nm`. */
export function managedObjdump() {
  const user = process.env.USER;
  if (!user) throw Error('stack_audit_gate_user');
  const toolRoot = join('/Users', user, '.rustup/toolchains/esp/xtensa-esp-elf');
  const candidates = readdirSync(toolRoot, { withFileTypes: true }).filter((entry) => entry.isDirectory())
    .map((entry) => join(toolRoot, entry.name, 'xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump'))
    .filter((candidate) => { try { accessSync(candidate, constants.X_OK); return true; } catch { return false; } });
  if (candidates.length !== 1) throw Error('stack_audit_gate_objdump');
  return candidates[0];
}

export function main([elf, ...rest]) {
  if (!elf || rest.length) throw Error('stack_audit_gate_arguments');
  const objdump = managedObjdump(), options = { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, timeout: 120000 };
  // Bound each body by its symbol size, as the audit CLIs do, so literal pools never decode as calls.
  const disassembly = boundToFunctions(execFileSync(objdump, ['-d', '-C', elf], options),
    functionSizes(execFileSync(objdump, ['-t', elf], options)));
  const read = (path) => readFileSync(join(root, 'firmware/bitaxe/src', path), 'utf8');
  return stackAuditRows(disassembly, { usbOwner: read('bwg_worker_usb.rs'), preparedLink: read('bwg_worker_usb/prepared_link.rs'),
    httpServer: read('http_api.rs'), bapAdapter: read('bap_adapter.rs') });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const rows = main(process.argv.slice(2));
  for (const row of rows) console.log(JSON.stringify(row));
  if (rows.some((row) => row.result !== PASS)) process.exitCode = 1;
}
