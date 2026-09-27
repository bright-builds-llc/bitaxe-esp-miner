// Executed by Bun against the exact pinned Gate checkout; synthetic records only.
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const root = process.argv[2];
const { WorkerSerialDiagnosticHistory, maybeWorkerSerialDiagnostic } = await import(pathToFileURL(resolve(root, 'web/worker-serial-diagnostics.ts')));
const { parseWorkerDiagnosticExport } = await import(pathToFileURL(resolve(root, 'web/worker-diagnostic-export.ts')));
function observe(history, line) {
  const record = maybeWorkerSerialDiagnostic(line); if (!record) throw Error('synthetic_diagnostic_invalid'); history.observe(record);
}
function ready(history, uptime) { observe(history, `usb_startup schema=v1 stage=runtime_ready state=complete first_failure=none uptime_ms=${uptime} redacted=true`); }
function boot(history, ordinal) { observe(history, `usb_reboot_discriminator schema=v1 boot_ordinal=${ordinal} reset_reason=software_cpu uptime_ms=500 redacted=true`); }
function identity(history, elf = 'b'.repeat(64)) { observe(history, `usb_runtime_identity schema=v1 firmware_commit=${'a'.repeat(40)} app_elf_sha256=${elf} redacted=true`); }
function snapshot(history) { return structuredClone(parseWorkerDiagnosticExport({ schema: 'worker-diagnostic-export-v1', observations: history.values() })); }
const history = new WorkerSerialDiagnosticHistory();
// Initial insertion order intentionally differs from wire chronology in later snapshots.
ready(history, 1000); identity(history); boot(history, 3);
const first = snapshot(history); ready(history, 1000); const same = snapshot(history);
ready(history, 2000); const second = snapshot(history);
boot(history, 4); const mixedBoot = snapshot(history); boot(history, 3);
identity(history, 'c'.repeat(64)); const wrongIdentity = snapshot(history); identity(history);
observe(history, 'storage_http_failure schema=v1 phase=http_server error=http_task redacted=true');
process.stdout.write(JSON.stringify({ first, same, second, mixedBoot, wrongIdentity, failure: snapshot(history) }));
