import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
const [output, ...pairs] = process.argv.slice(2);
if (!output || pairs.length % 2) throw Error('compiler_stamp_arguments');
const rows = [];
for (let i = 0; i < pairs.length; i += 2) rows.push([pairs[i], createHash('sha256').update(await readFile(pairs[i + 1])).digest('hex')]);
rows.sort((a,b) => a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0);
await writeFile(output, `BITAXE_VIRTUAL_COMPILED_INPUTS_JSON=${JSON.stringify(rows)}\n`, { flag: 'wx' });
