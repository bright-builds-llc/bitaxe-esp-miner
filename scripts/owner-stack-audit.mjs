export const OWNER_ENTRY_BUDGET_BYTES = 8192;
export const OWNER_STACK_BYTES = 24576;
export const MINIMUM_MEASURED_FREE_BYTES = 4096;

/** Checks the actual native entry frame, not a complete worst-case call-chain bound. */
export function auditOwnerStack(disassembly, source) {
  if (!source.includes('const OWNER_STACK_BYTES: usize = 24 * 1024;')
      || !source.includes('.stack_size(OWNER_STACK_BYTES)')) {
    throw Error('owner_stack_contract');
  }
  const sections = disassembly.split(/(?=^[0-9a-f]+ <[^\n]+>:\s*$)/m);
  const matches = sections.filter((section) =>
    /^[0-9a-f]+ <bitaxe_production_owner_entry>:\s*$/m.test(section.split('\n')[0]));
  if (matches.length !== 1) throw Error('owner_entry_missing_or_multiple');
  const entries = [...matches[0].matchAll(/\bentry\s+a1,\s*(0x[0-9a-f]+|[0-9]+)\s*$/gm)];
  if (entries.length !== 1) throw Error('owner_entry_frame_missing_or_multiple');
  const frame = Number(entries[0][1]);
  if (!Number.isSafeInteger(frame) || frame < 32 || frame % 16 !== 0
      || frame > OWNER_ENTRY_BUDGET_BYTES) {
    throw Error('owner_entry_frame_budget');
  }
  if (/\b(?:add|addi|addmi|sub|mov|movsp)(?:\.n)?\s+a1\s*,/m.test(matches[0])) {
    throw Error('owner_dynamic_stack_unbounded');
  }
  return {
    schema: 'production-owner-stack-audit-v1',
    stack_bytes: OWNER_STACK_BYTES,
    owner_entry_bytes: frame,
    owner_entry_budget_bytes: OWNER_ENTRY_BUDGET_BYTES,
    required_measured_free_bytes: MINIMUM_MEASURED_FREE_BYTES,
  };
}
