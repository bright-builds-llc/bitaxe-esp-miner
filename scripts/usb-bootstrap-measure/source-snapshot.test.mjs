import test from "node:test";
import assert from "node:assert/strict";
import { chmod, link, mkdir, mkdtemp, readFile, realpath, rm, symlink, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { inventory, retain } from "../str005-noise-serial/files.mjs";
import { sha256 } from "./values.mjs";
import { readSourceSnapshot, sourceSnapshotRelative, verifySourceSnapshot, writeSourceSnapshot } from "./source-snapshot.mjs";
const PUBLISHED = "scripts/phase28.1.1.1-synthetic-pool-credentials.mjs", CLIENT = "scripts/usb-bootstrap-measure/client.mjs";
const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
async function fixture(t, version = 2) {
  const root = await mkdtemp(resolve(await realpath(tmpdir()), "bootstrap-source-")); await chmod(root, 0o700);
  t.after(() => rm(root, { recursive: true, force: true }));
  const firmwareRoot = resolve(root, "repo"), evidence = resolve(root, "evidence"); await mkdir(evidence, { mode: 0o700 });
  const sourceInventory = [];
  for (const path of [PUBLISHED, CLIENT].sort()) {
    // These are public checked-in program bytes, read only; neither module is imported or executed.
    const bytes = await readFile(resolve(repository, path)); await retain(resolve(firmwareRoot, path), bytes);
    sourceInventory.push({ path, sha256: sha256(bytes), length: bytes.length });
  }
  return { root: evidence, context: { schema: `usb-bootstrap-measure-context-v${version}`, firmwareRoot, sourceInventory } };
}
test("published synthetic-source filename survives encoded snapshot, inventory and fixed asset lookup", async t => {
  // Arrange
  const f = await fixture(t);
  // Act
  await writeSourceSnapshot(f.root, f.context); await verifySourceSnapshot(f.root, f.context);
  const files = await inventory(f.root), client = await readSourceSnapshot(f.root, f.context, CLIENT);
  // Assert
  assert.equal(files.length, 2); assert.ok(files.every(file => /^snapshot\/source\/[a-f0-9]{64}\.bin$/u.test(file.path)));
  assert.deepEqual(client, await readFile(resolve(repository, CLIENT)));
  assert.equal(sourceSnapshotRelative(f.context, PUBLISHED), `snapshot/source/${sha256(Buffer.from(PUBLISHED))}.bin`);
});
test("original verbatim layout still triggers the unchanged generic filename guard", async t => {
  // Arrange
  const f = await fixture(t, 1);
  await retain(resolve(f.root, sourceSnapshotRelative(f.context, PUBLISHED)), await readFile(resolve(repository, PUBLISHED)));
  // Act / Assert
  await assert.rejects(inventory(f.root), { code: "noise_forbidden_inventory_name" });
  await assert.rejects(writeSourceSnapshot(f.root, f.context), { code: "bootstrap_source_layout_readonly" });
});
for (const mutation of ["credentials", "extra", "missing", "substitution", "symlink", "hardlink", "directory", "mode"]) test(`snapshot rejects ${mutation}`, async t => {
  // Arrange
  const f = await fixture(t); await writeSourceSnapshot(f.root, f.context);
  const path = resolve(f.root, sourceSnapshotRelative(f.context, CLIENT)), directory = dirname(path);
  // Act
  if (mutation === "credentials") await retain(resolve(directory, "pool-credentials.json"), Buffer.from("unexpected artifact"));
  if (mutation === "extra") await retain(resolve(directory, `${"f".repeat(64)}.bin`), Buffer.from("extra"));
  if (mutation === "missing") await unlink(path);
  if (mutation === "substitution") await writeFile(path, "changed");
  if (mutation === "symlink") { await unlink(path); await symlink(resolve(f.context.firmwareRoot, CLIENT), path); }
  if (mutation === "hardlink") { await unlink(path); await link(resolve(f.context.firmwareRoot, CLIENT), path); }
  if (mutation === "directory") await mkdir(resolve(directory, "extra"), { mode: 0o700 });
  if (mutation === "mode") await chmod(path, 0o644);
  // Assert
  await assert.rejects(verifySourceSnapshot(f.root, f.context));
});
test("duplicate or unindexed logical paths cannot select or write storage", async t => {
  // Arrange
  const f = await fixture(t); const duplicate = { ...f.context, sourceInventory: [f.context.sourceInventory[0], f.context.sourceInventory[0]] };
  // Act / Assert
  assert.throws(() => sourceSnapshotRelative(duplicate, PUBLISHED), { code: "bootstrap_source_inventory" });
  await assert.rejects(writeSourceSnapshot(f.root, duplicate), { code: "bootstrap_source_inventory" });
  assert.throws(() => sourceSnapshotRelative(f.context, "../private"), { code: "bootstrap_source_unindexed" });
});
test("version one reads only verbatim bytes and never falls back to encoded files", async t => {
  // Arrange
  const f = await fixture(t); await writeSourceSnapshot(f.root, f.context);
  const legacy = { ...f.context, schema: "usb-bootstrap-measure-context-v1" };
  // Act / Assert
  await assert.rejects(readSourceSnapshot(f.root, legacy, CLIENT), { code: "ENOENT" });
  const bytes = await readFile(resolve(repository, CLIENT)); await retain(resolve(f.root, sourceSnapshotRelative(legacy, CLIENT)), bytes);
  assert.deepEqual(await readSourceSnapshot(f.root, legacy, CLIENT), bytes);
  await unlink(resolve(f.root, sourceSnapshotRelative(f.context, CLIENT)));
  await assert.rejects(readSourceSnapshot(f.root, f.context, CLIENT), { code: "ENOENT" });
});
