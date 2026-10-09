import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { appendFile, chmod, mkdir, readdir, readFile, stat, utimes, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { digest, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { inventory } from "../str005-noise-serial/files.mjs";
import { RESTORATION_TASK, RESTORATION_TASK_LINE, RESULT_SCHEMA, SCENARIO_RESULT_SCHEMA, SCENARIOS } from "./contract.mjs";
import { context, passingInput, privateDirectory } from "./fixtures.test-helper.mjs";
import { judgeScenario } from "./judge.mjs";
import { argumentsFor, main } from "./main.mjs";

const GATE_COMMIT = "e".repeat(40), FIRMWARE_COMMIT = "a".repeat(40), PHYSICAL = "5".repeat(64);
const rejectsWith = (promise, code) => assert.rejects(promise, (error) => error.code === code);
const key = { kid: "k1", kty: "OKP", crv: "Ed25519", x: Buffer.alloc(32, 7).toString("base64url"), alg: "Ed25519", use: "sig", key_ops: ["verify"] };
const trust = { profile: "bwg-worker-deployment-trust/0.2", updateAuthority: { issuer: "i", audience: "a", keys: [key] }, workLeaseAuthority: { issuer: "i", audience: "a", keys: [key] } };

/** A real Git firmware checkout (for check-ignore), a Gate tree, an authority directory and fresh inputs. */
async function fixture({ enabled = true, pin = GATE_COMMIT } = {}) {
  const base = await privateDirectory("restoration-main-");
  const firmware = resolve(base, "firmware"), gate = resolve(base, "gate"), authority = resolve(base, "authority"), parent = resolve(firmware, "scratch/p");
  execFileSync("git", ["init", "-q", firmware]);
  await writeFile(resolve(firmware, ".gitignore"), "scratch/\n");
  await writeFile(resolve(firmware, "TASKS.md"), `## Active\n\n### ${RESTORATION_TASK} | 2026-08-29 | x\n\n${enabled ? RESTORATION_TASK_LINE : ""}\n`);
  await writeFile(resolve(firmware, "MODULE.bazel"), `strip_prefix = "bitaxe-turnstile-system-${pin}",\n`);
  await mkdir(resolve(firmware, "firmware/bitaxe/bwg"), { recursive: true });
  await writeFile(resolve(firmware, "firmware/bitaxe/bwg/deployment-trust.json"), JSON.stringify(trust));
  await mkdir(resolve(gate, "dist/worker-restoration"), { recursive: true });
  await mkdir(resolve(gate, "conformance/bwg-worker-serial-0.2"), { recursive: true });
  await writeFile(resolve(gate, "dist/worker-restoration/worker-restoration-page.js"), `// ${GATE_COMMIT}`);
  await writeFile(resolve(gate, "conformance/bwg-worker-serial-0.2/restoration.html"), "<!doctype html>");
  await mkdir(authority, { mode: 0o700 });
  await mkdir(parent, { recursive: true, mode: 0o700 });
  await chmod(parent, 0o700);
  const pool = resolve(parent, "pool.json"), detector = resolve(parent, "detector.stdout.log");
  await writeFile(pool, "{}", { mode: 0o600 });
  await writeFile(detector, `port: /dev/cu.usbmodem1101\nphysical_identity_sha256: ${PHYSICAL}\nusb_profile: serial_jtag_runtime\n`, { mode: 0o600 });
  const argv = (root = resolve(parent, "attempt-001")) => ["preflight", "--private-root", root, "--firmware-root", firmware, "--gate-root", gate,
    "--firmware-commit", FIRMWARE_COMMIT, "--gate-commit", GATE_COMMIT, "--manifest", resolve(firmware, "manifest.json"),
    "--authority-directory", authority, "--pool-credentials", pool, "--detector", detector];
  const operations = { cleanPushed: () => undefined, packageSnapshot: async () => ({ manifest_sha256: "1".repeat(64), app_elf_sha256: "b".repeat(64) }),
    authorityCall: async () => trust, canonicalWatcher: async () => ({ path: "/bin/flash", sha256: "6".repeat(64) }) };
  return { firmware, parent, detector, pool, argv, operations };
}

test("unknown actions, missing options and unknown options are refused", () => {
  // Arrange / Act / Assert
  assert.throws(() => argumentsFor(["start"]), (error) => error.code === "restoration_action");
  assert.throws(() => argumentsFor(["serve", "--private-root", "x"]), (error) => error.code === "restoration_arguments");
  assert.throws(() => argumentsFor(["finish", "--private-root", "x", "--force", "y"]), (error) => error.code === "restoration_arguments");
});

test("preflight freezes the attempt without device effects", async () => {
  // Arrange
  const { parent, argv, operations } = await fixture();
  // Act
  const result = await main(argv(), operations);
  // Assert
  assert.deepEqual(result, { restoration_preflight_created: true, device_effects: false, attempt: "attempt-001", scenarios: 8 });
  const root = resolve(parent, "attempt-001");
  assert.equal((await stat(root)).mode & 0o777, 0o700);
  const saved = JSON.parse(await readFile(resolve(root, "context.json"), "utf8")).context;
  assert.deepEqual([saved.physical_identity_sha256, saved.gate_commit, saved.watcher.sha256], [PHYSICAL, GATE_COMMIT, "6".repeat(64)]);
});

test("preflight refuses without the exact enable line", async () => {
  // Arrange
  const { parent, argv, operations } = await fixture({ enabled: false });
  // Act / Assert
  await rejectsWith(main(argv(), operations), "restoration_task_disabled");
  assert.deepEqual((await readdir(parent)).sort(), ["detector.stdout.log", "pool.json"]);
});

test("preflight refuses a Gate commit other than the pinned one", async () => {
  // Arrange
  const { argv, operations } = await fixture({ pin: "f".repeat(40) });
  // Act / Assert
  await rejectsWith(main(argv(), operations), "gate_commit_not_pinned");
});

test("preflight refuses a detector output older than 300 s", async () => {
  // Arrange
  const { detector, argv, operations } = await fixture();
  const old = new Date(Date.now() - 301000);
  await utimes(detector, old, old);
  // Act / Assert
  await rejectsWith(main(argv(), operations), "restoration_detector_stale");
});

test("preflight refuses an existing root, a misnamed root and a readable pool file", async () => {
  // Arrange
  const { parent, pool, argv, operations } = await fixture();
  await mkdir(resolve(parent, "attempt-002"), { mode: 0o700 });
  // Act / Assert
  await rejectsWith(main(argv(resolve(parent, "attempt-002")), operations), "private_path_exists");
  await rejectsWith(main(argv(resolve(parent, "run-1")), operations), "attempt_name");
  await chmod(pool, 0o644);
  await rejectsWith(main(argv(), operations), "private_path_policy");
});

/** A sealed attempt root with eight judged scenarios, as serve and finish leave it. */
async function sealedRoot({ result = "passed", credentialScan = { files: 12, hits: 0 } } = {}) {
  const base = await privateDirectory("restoration-publish-");
  const firmware = resolve(base, "firmware"), root = resolve(base, "attempt-007");
  await mkdir(firmware); await mkdir(root, { mode: 0o700 });
  const frozen = { ...context, attempt: "attempt-007", firmware_root: firmware };
  await writeNew(resolve(root, "context.json"), { context: frozen, sha256: digest(JSON.stringify(frozen)) });
  for (const [index, scenario] of SCENARIOS.entries()) {
    const value = { schema: SCENARIO_RESULT_SCHEMA, ...judgeScenario(passingInput(scenario)), context_sha256: digest(JSON.stringify(frozen)),
      physical_identity_sha256: frozen.physical_identity_sha256 };
    await writeNew(resolve(root, `scenario-${String(index + 1).padStart(2, "0")}-${scenario}.json`), { value, sha256: digest(JSON.stringify(value)) });
  }
  const final = { schema: RESULT_SCHEMA, result, scenarios: SCENARIOS.map((scenario) => ({ scenario, result: "passed", failures: [] })),
    ...(credentialScan ? { credential_scan: credentialScan } : {}) };
  await writeNew(resolve(root, "result.json"), { result: final, sha256: digest(JSON.stringify(final)) });
  await writeFile(resolve(root, "records.jsonl"), `${JSON.stringify({ scenario: "completion", operation: "statusReview" })}\n`, { mode: 0o600 });
  await writeNew(resolve(root, "sealed-inventory.json"), { files: await inventory(root) });
  return { root, directory: resolve(firmware, "docs/parity/evidence/bwg-worker-restoration") };
}

test("publish writes the eight projections from a sealed, passed root", async () => {
  // Arrange
  const { root, directory } = await sealedRoot();
  // Act
  const result = await main(["publish", "--private-root", root]);
  // Assert
  assert.deepEqual(result, { restoration_published: true, projections: 8 });
  const files = (await readdir(directory)).sort();
  assert.equal(files.length, 8);
  const text = await readFile(resolve(directory, "bwg007-attempt-007-reboot.json"), "utf8");
  assert.equal(JSON.parse(text).facts.rebootClearedStimulus, true);
  await rejectsWith(main(["publish", "--private-root", root]), "projection_exists");
});

test("publish refuses an unverified attempt and a root changed after sealing", async () => {
  // Arrange
  const unverified = await sealedRoot({ result: "unverified" });
  const changed = await sealedRoot();
  await appendFile(resolve(changed.root, "records.jsonl"), "{}\n");
  // Act / Assert
  await rejectsWith(main(["publish", "--private-root", unverified.root]), "restoration_not_passed");
  await rejectsWith(main(["publish", "--private-root", changed.root]), "noise_inventory_changed");
  await rejectsWith(readdir(unverified.directory), "ENOENT");
});

test("publish refuses a sealed result without a clean credential scan", async () => {
  // Arrange
  const unscanned = await sealedRoot({ credentialScan: null });
  const leaked = await sealedRoot({ credentialScan: { files: 12, hits: 1 } });
  // Act / Assert
  await rejectsWith(main(["publish", "--private-root", unscanned.root]), "restoration_credential_scan_missing");
  await rejectsWith(main(["publish", "--private-root", leaked.root]), "restoration_credential_scan_missing");
});
