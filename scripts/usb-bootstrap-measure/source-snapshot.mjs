import { constants } from "node:fs";
import { lstat, open, readFile, readdir, realpath } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { sourceShape } from "./shapes.mjs";
import { check, sha256 } from "./values.mjs";
import { inventory, privateRoot, protectedPath, retain, canonical } from "../str005-noise-serial/files.mjs";
const V1 = "usb-bootstrap-measure-context-v1", V2 = "usb-bootstrap-measure-context-v2", V3 = "usb-bootstrap-measure-context-v3";
function mapping(context) {
  check([V1, V2, V3].includes(context.schema), "bootstrap_source_layout"); sourceShape(context.sourceInventory);
  const rows = context.sourceInventory.map(row => ({ ...row, storage: context.schema !== V1 ? `${sha256(Buffer.from(row.path, "utf8"))}.bin` : row.path }));
  check(new Set(rows.map(row => row.storage)).size === rows.length, "bootstrap_source_storage_duplicate"); return rows;
}
/** Storage identity hashes the logical path, independently of the content digest. */
export function sourceSnapshotRelative(context, logicalPath) {
  const row = mapping(context).find(row => row.path === logicalPath);
  check(row, "bootstrap_source_unindexed"); return `snapshot/source/${row.storage}`;
}
async function checkedBytes(root, relative, row) {
  root = await privateRoot(root); const path = resolve(root, relative);
  for (let parent = dirname(path); parent !== root; parent = dirname(parent)) await protectedPath(parent, true);
  await protectedPath(path);
  const handle = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const stat = await handle.stat();
    check(stat.isFile() && stat.nlink === 1 && stat.size === row.length, "bootstrap_source_alias_or_length");
    const bytes = await handle.readFile(); check(bytes.length === row.length && sha256(bytes) === row.sha256, "bootstrap_source_snapshot_changed"); return bytes;
  } finally { await handle.close(); }
}
/** Fixed callers select an indexed logical source; no route accepts arbitrary paths. */
export async function readSourceSnapshot(root, context, logicalPath) {
  const row = mapping(context).find(row => row.path === logicalPath); check(row, "bootstrap_source_unindexed");
  return checkedBytes(root, sourceSnapshotRelative(context, logicalPath), row);
}
/** Only new v2 contexts may create snapshots. Historical layouts are never rewritten. */
export async function writeSourceSnapshot(root, context) {
  check([V2, V3].includes(context.schema), "bootstrap_source_layout_readonly"); const rows = mapping(context);
  root = await privateRoot(root);
  for (const row of rows) {
    const path = resolve(context.firmwareRoot, row.path), stat = await lstat(path);
    check(stat.isFile() && !stat.isSymbolicLink() && stat.nlink === 1 && await realpath(path) === path, "bootstrap_source_alias_or_length");
    const bytes = await readFile(path); check(bytes.length === row.length && sha256(bytes) === row.sha256, "bootstrap_sources_changed");
    await retain(resolve(root, "snapshot/source", row.storage), bytes);
  }
  await verifySourceSnapshot(root, context);
}
/** Exact membership includes directories; extra empty directories are not evidence. */
export async function verifySourceSnapshot(root, context) {
  const rows = mapping(context), directory = resolve(root, "snapshot/source");
  const actual = await inventory(directory), expected = rows.map(row => ({ path: row.storage, sha256: row.sha256, length: row.length })).sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
  check(canonical(actual) === canonical(expected), "bootstrap_source_membership");
  const expectedDirs = new Set();
  for (const row of rows) { let parent = dirname(row.storage); while (parent !== ".") { expectedDirs.add(parent); parent = dirname(parent); } }
  async function directories(path, prefix = "") {
    for (const entry of await readdir(path, { withFileTypes: true })) if (entry.isDirectory()) {
      const relative = prefix + entry.name; check(expectedDirs.delete(relative), "bootstrap_source_membership"); await directories(resolve(path, entry.name), `${relative}/`);
    }
  }
  await directories(directory); check(expectedDirs.size === 0, "bootstrap_source_membership");
  for (const row of rows) await checkedBytes(root, `snapshot/source/${row.storage}`, row);
}
