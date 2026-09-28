import { constants } from 'node:fs';
import { lstat, open, readdir, realpath } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { git, cleanPushed, ignored, missing } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, protectedPath, inventory, verifyInventory, writeNew, digest } from '../str005-noise-serial/files.mjs';
import { check } from '../str005-v2-serial/values.mjs';
import { producerResult, fullRegion, acquisitionBlockers, DUMP_BYTES, INSTALLED_SOURCE, INSTALLED_ELF } from './model.mjs';
const RECEIPT = 'acquisition-verification.json', SEAL = 'sealed-inventory.json';
const ORIGINAL_FILES = ['core-dump.private.bin', 'partition-table.private.bin', 'result.private.json'];
export function argumentsFor(argv) {
  check(argv.length === 3 && argv[0] === 'verify-acquisition' && argv[1] === '--private-root' &&
    typeof argv[2] === 'string' && resolve(argv[2]) === argv[2], 'acquisition_arguments');
  return argv[2];
}
async function bytes(root, name, bound) {
  const path = resolve(root, name); await protectedPath(path);
  const handle = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW);
  try { const meta = await handle.stat(); check(meta.isFile() && meta.size <= bound, 'acquisition_input_bound'); return await handle.readFile(); }
  finally { await handle.close(); }
}
async function protectedInputs(root) {
  await privateRoot(root);
  for (let current = root; ; current = dirname(current)) {
    check(!(await lstat(current)).isSymbolicLink() && await realpath(current) === current, 'acquisition_alias');
    if (dirname(current) === current) break;
  }
  await missing(resolve(root, RECEIPT)); await missing(resolve(root, SEAL));
  const files = await readdir(root);
  check(files.length <= ORIGINAL_FILES.length && files.every(name => ORIGINAL_FILES.includes(name)), 'acquisition_unexpected_file');
  for (const name of files) {
    const meta = await lstat(resolve(root, name)); await protectedPath(resolve(root, name));
    check(meta.size <= (name === 'core-dump.private.bin' ? DUMP_BYTES : name === 'partition-table.private.bin' ? 4096 : 16384), 'acquisition_input_bound');
  }
}
/** No device discovery, serial access, subprocess tools, clearing or decoding occurs here. */
export async function verifyAcquisition(root, { source, recheckSource = async () => {} }) {
  check(/^[a-f0-9]{40}$/u.test(source), 'acquisition_source'); await recheckSource(); await protectedInputs(root);
  const before = await inventory(root), blockers = [];
  let result, table, dump; const observed = new Map();
  try { const producer = await proof(root, 'result.private.json'); observed.set('result.private.json', producer.sha256);
    result = producerResult(producer.value); blockers.push(...acquisitionBlockers(result, source)); }
  catch { blockers.push('acquisition_producer_unverified'); }
  try { table = await bytes(root, 'partition-table.private.bin', 4096); observed.set('partition-table.private.bin', digest(table)); fullRegion(table); }
  catch (error) { blockers.push(/^acquisition_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'acquisition_table_unavailable'); }
  try { dump = await bytes(root, 'core-dump.private.bin', DUMP_BYTES); observed.set('core-dump.private.bin', digest(dump)); check(dump.length === DUMP_BYTES, 'acquisition_dump_length'); }
  catch (error) { blockers.push(/^acquisition_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'acquisition_dump_unavailable'); }
  check([...observed].every(([path, hash]) => before.some(file => file.path === path && file.sha256 === hash)), 'acquisition_inputs_changed');
  await recheckSource(); await verifyInventory(root, before);
  const released = result?.cleanup_complete === true;
  const receipt = { schema: 'str005-share-crash-acquisition-v1', source_commit: source,
    installed_source: INSTALLED_SOURCE, installed_elf_sha256: INSTALLED_ELF,
    complete: blockers.length === 0, blockers, release_proven: released,
    full_region_bytes: dump?.length ?? null, dump_sha256: dump ? digest(dump) : null,
    partition_table_sha256: table ? digest(table) : null,
    producer_result_sha256: before.find(file => file.path === 'result.private.json')?.sha256 ?? null,
    decoder_verified: false, panic_cause_proven: false, parity_promotion: false };
  await writeNew(resolve(root, RECEIPT), receipt);
  const receiptProof = await proof(root, RECEIPT);
  const expected = [...before, { path: RECEIPT, sha256: receiptProof.sha256, length: receiptProof.bytes.length }].sort((a, b) => a.path.localeCompare(b.path));
  await verifyInventory(root, expected);
  if (released) await writeNew(resolve(root, SEAL), { files: expected });
  return { ...receipt, sealed: released, verification_sha256: receiptProof.sha256 };
}
export async function main(argv) {
  const root = argumentsFor(argv), workspace = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  const source = git(workspace, ['rev-parse', 'HEAD']); cleanPushed(workspace, source); ignored(workspace, root);
  return verifyAcquisition(root, { source, recheckSource: async () => cleanPushed(workspace, source) });
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => {
  process.stdout.write(`${JSON.stringify(value)}\n`); if (!value.complete) process.exitCode = 1;
}).catch(error => { process.stdout.write(`${JSON.stringify({ complete: false, sealed: false,
  blocker: /^acquisition_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'acquisition_verification_failed' })}\n`); process.exitCode = 1; });
