import assert from 'node:assert/strict';
import test from 'node:test';
import {auditOwnerStack} from './owner-stack-audit.mjs';
const source='const OWNER_STACK_BYTES: usize = 24 * 1024;\n.stack_size(OWNER_STACK_BYTES)';
const frame=(size)=>`42000000 <bitaxe_production_owner_entry>:\n42000000: 000136 entry a1, ${size}\n42000003: 000081 retw.n\n`;
test('native entry fits bounded stack contract but still requires measured runtime headroom',()=>{assert.deepEqual(auditOwnerStack(frame('0x1ae0'),source),{schema:'production-owner-stack-audit-v1',stack_bytes:24576,owner_entry_bytes:6880,owner_entry_budget_bytes:8192,required_measured_free_bytes:4096});});
for(const [name,text]of[['oversized',frame(8208)],['multiple_entries',frame(6880)+'42000004: 000136 entry a1, 32\n'],['missing',''],['multiple',frame(6880)+frame(6880)],['no_entry',frame(6880).replace('entry','movi')],['dynamic',frame(6880)+'42000004: 000001 addi a1, a1, -16\n']]){
 test(`rejects ${name} native owner frame`,()=>{assert.throws(()=>auditOwnerStack(text,source));});
}
test('rejects old stack contract even if native entry individually fits',()=>{assert.throws(()=>auditOwnerStack(frame(6880),source.replace('24 * 1024','16 * 1024')));});
