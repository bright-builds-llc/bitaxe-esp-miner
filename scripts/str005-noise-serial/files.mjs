import { constants } from "node:fs";
import { lstat, mkdir, open, readdir, readFile, realpath } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { digest, exactObject, protectedPath, readJson, requireCondition as check, within, writeNew } from "../fixed-usb-qualification/contract.mjs";
export { digest, writeNew, protectedPath, readJson, check };

export async function privateRoot(root) {
  check(await realpath(root) === resolve(root), "noise_root_alias");
  await protectedPath(root, true);
  return resolve(root);
}
export async function proof(root, relative) {
  const path = within(root, resolve(root, relative));
  let parent = dirname(path);
  while (parent !== resolve(root)) { await protectedPath(parent, true); parent = dirname(parent); }
  await protectedPath(path);
  const handle = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW);
  let bytes;
  try { const stat = await handle.stat(); check(stat.isFile() && stat.size <= 8_388_608, "noise_evidence_bound"); bytes = await handle.readFile(); }
  finally { await handle.close(); }
  return { value: JSON.parse(bytes.toString("utf8")), sha256: digest(bytes), bytes };
}
export async function retain(path, bytes) {
  await mkdir(dirname(path), { recursive: true, mode: 0o700 });
  const file = await open(path, "wx", 0o600);
  try { await file.writeFile(bytes); await file.sync(); } finally { await file.close(); }
}
export async function inventory(root, exclusions = new Set()) {
  await privateRoot(root);
  const files = [];
  async function visit(dir, prefix) {
    for (const name of (await readdir(dir)).sort()) {
      const relative = `${prefix}${name}`;
      if (exclusions.has(relative)) continue;
      check(!/(?:credentials|private[-_]key|authority[-_]directory)/iu.test(name), "noise_forbidden_inventory_name");
      const path = resolve(dir, name), stat = await lstat(path);
      check(!stat.isSymbolicLink(), "noise_inventory_symlink");
      if (stat.isDirectory()) { await protectedPath(path, true); await visit(path, `${relative}/`); }
      else { await protectedPath(path); const bytes = await readFile(path); files.push({ path: relative, sha256: digest(bytes), length: bytes.length }); }
    }
  }
  await visit(root, "");
  return files;
}
export async function verifyInventory(root, expected, exclusions = new Set()) {
  check(Array.isArray(expected), "noise_inventory_shape");
  for (const item of expected) exactObject(item, ["path", "sha256", "length"]);
  check(JSON.stringify(await inventory(root, exclusions)) === JSON.stringify(expected), "noise_inventory_changed");
}
export function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}
