import {readFile} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {auditOwnerStack} from './owner-stack-audit.mjs';
const [elf,objdump,sourcePath]=process.argv.slice(2);
if(!elf||!objdump||!sourcePath)throw Error('usage: owner-stack-audit-cli.mjs ELF OBJDUMP OWNER_SOURCE');
const disassembly=execFileSync(objdump,['-Cd',elf],{encoding:'utf8',timeout:30000,maxBuffer:128*1024*1024});
console.log(JSON.stringify(auditOwnerStack(disassembly,await readFile(sourcePath,'utf8'))));
