import assert from "node:assert/strict";
import { once } from "node:events";
import { readFile, readdir, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import {
  createStartupRecoverySupervisor,
  startupRecoveryPreflight,
  consumeStartupRecoveryInstall,
  loadStartupRecoveryContext,
  judgeStartupRecovery,
  readStartupRecovery,
} from "./cadence-startup-recovery.mjs";
import { startupFixture, installedRecovery, recoveryState, completeRecoveryFixture } from "./cadence-startup-fixtures.mjs";
import { requireStartupInstallation } from "./cadence-startup-install.mjs";
import { readStartupFailure } from "./cadence-startup-failure.mjs";
import { inspectNoMiningSources } from "./no-mining-context.mjs";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { main } from "./main.mjs";

async function httpServer(t, f) {
  const server = await createStartupRecoverySupervisor({ privateRoot: f.root }, f.operations);
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  t.after(
    () =>
      new Promise((done) => {
        server.close(done);
        server.closeAllConnections();
      }),
  );
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, body = {}, headerOrigin = origin) =>
    fetch(origin + path, {
      method: "POST",
      headers: { "Content-Type": "application/json", Origin: headerOrigin },
      body: JSON.stringify(body),
    });
  return { server, origin, post };
}

test("startup recovery preserves failed evidence and reserves exactly one recovery directory", async (t) => {
  // Arrange
  const f = await startupFixture(t),
    before = await fileDigest(resolve(f.failedRoot, "failed-inventory.json"));
  await rm(resolve(f.base, "authority"), { recursive: true });
  // Act
  assert.equal((await startupRecoveryPreflight(f.options, f.operations)).mining_authorized, false);
  const context = await loadStartupRecoveryContext(f.options.privateRoot, { operations: f.operations });
  // Assert
  assert.equal(context.no_mining_context.mining_authorized, false);
  assert.equal(context.no_mining_context.schema, "fixed-usb-no-mining-context-v1");
  assert.equal(await fileDigest(resolve(f.failedRoot, "failed-inventory.json")), before);
  await assert.rejects(startupRecoveryPreflight({ ...f.options, privateRoot: resolve(f.base, "attempts/duplicate") }, f.operations), {
    code: "EEXIST",
  });
});

test("installation claim is irreversible before an evidence child exists", async (t) => {
  // Arrange
  const f = await startupFixture(t);
  await startupRecoveryPreflight(f.options, f.operations);
  // Act / Assert
  assert.equal((await consumeStartupRecoveryInstall(f.options.privateRoot, f.operations)).install_consumed, true);
  await assert.rejects(consumeStartupRecoveryInstall(f.options.privateRoot, f.operations), { code: "EEXIST" });
  await assert.rejects(createStartupRecoverySupervisor({ privateRoot: f.options.privateRoot }, f.operations), { code: "ENOENT" });
  assert(!(await readdir(f.options.privateRoot)).includes("install-001"));
});

test("installation uses natural observer-claim-write chronology and rejects bad startup or release", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["claim_before_observer", "claim_after_flash", "startup", "owner"]) {
    const f = await installedRecovery(t);
    await requireStartupInstallation(f.root, f.context);
    const path = resolve(
      f.root,
      mode.startsWith("claim")
        ? "install-consumed.json"
        : mode === "startup"
          ? "install-001/flash-command-evidence.json"
          : "install-owner-cleanup.json",
    );
    const value = await readJson(path);
    if (mode === "claim_before_observer") value.claimed_at_unix_ms = 500;
    if (mode === "claim_after_flash") value.claimed_at_unix_ms = 10000;
    if (mode === "startup") value.fixed_serial_assessment.stable_boot = false;
    if (mode === "owner") value.serial_holders_absent = false;
    await writeFile(path, JSON.stringify(value));
    await assert.rejects(requireStartupInstallation(f.root, f.context));
  }
});

test("recovery server offers read-only accounting with no signer, work, diagnostic or fault routes", async (t) => {
  // Arrange
  const f = await installedRecovery(t),
    h = await httpServer(t, f),
    inner = f.context.no_mining_context;
  // Act / Assert
  assert.equal((await fetch(h.origin + "/context")).status, 200);
  for (const path of ["/authorize", "/sign-start", "/window", "/consume", "/read-only-interruption", "/diagnostic-export"])
    assert.equal((await h.post(path)).status, 404);
  assert.equal((await h.post("/original-budget-context", {}, "http://foreign.invalid")).status, 400);
  const state = recoveryState(inner);
  assert.equal((await h.post("/record", { state })).status, 200);
  const ledger = {
    schema: "worker-qualification-ledger-v1",
    next_ordinal: 17,
    total_charged_ms: 1380000,
    pending: false,
    last_completed_ordinal: 16,
  };
  assert.equal((await h.post("/accounting", { stage: "before", state, ledger, original_budget: f.original })).status, 200);
  assert(!(await readdir(f.root)).includes("issued.json"));
  assert.equal((await h.post("/record", { state: { ...state, running: true } })).status, 400);
});

test("state recording avoids artifact revalidation while admission and accounting still verify frozen sources", async (t) => {
  // Arrange
  const f = await installedRecovery(t);
  let sourceChecks = 0;
  f.operations.cleanPushed = () => { sourceChecks++; };
  const h = await httpServer(t, f), state = recoveryState(f.context.no_mining_context);
  const baselineChecks = sourceChecks;
  // Act / Assert
  assert.equal((await h.post("/record", { state })).status, 200);
  assert.equal(sourceChecks, baselineChecks);
  assert.equal((await fetch(h.origin + "/context")).status, 200);
  assert(sourceChecks > baselineChecks);
  const contextChecks = sourceChecks;
  assert.equal((await h.post("/activate")).status, 200);
  assert(sourceChecks > contextChecks);
  const admissionChecks = sourceChecks;
  const ledger = { schema: "worker-qualification-ledger-v1", next_ordinal: 17, total_charged_ms: 1380000, pending: false, last_completed_ordinal: 16 };
  assert.equal((await h.post("/accounting", { stage: "before", state, ledger, original_budget: f.original })).status, 200);
  assert(sourceChecks > admissionChecks);
});

test("wrapper refuses context injection, source drift and archived cadence task", async (t) => {
  // Arrange / Act / Assert
  const f = await installedRecovery(t);
  await assert.rejects(createStartupRecoverySupervisor({ privateRoot: f.root, context: f.context.no_mining_context }, f.operations), {
    code: "startup_context_injection",
  });
  await writeFile(resolve(f.options.firmwareRoot, "scripts/fixed-usb-qualification/fixture.mjs"), "changed");
  await assert.rejects(createStartupRecoverySupervisor({ privateRoot: f.root }, f.operations), { code: "startup_validator_drift" });
  const g = await installedRecovery(t);
  await writeFile(resolve(g.options.firmwareRoot, "TASKS.md"), "## Active\n");
  await assert.rejects(createStartupRecoverySupervisor({ privateRoot: g.root }, g.operations), { code: "cadence_active_task_required" });
});

test("recovery judge proves only current recovery baseline and unchanged accounting", async (t) => {
  // Arrange
  const f = await completeRecoveryFixture(t),
    failedHash = await fileDigest(resolve(f.failedRoot, "failed-inventory.json"));
  // Act
  const result = await judgeStartupRecovery(f.root, f.cleanupPath, f.operations),
    receipt = await readStartupRecovery(resolve(f.root, "result.json"), f.operations);
  // Assert
  assert.equal(result.result, "recovered");
  assert.equal(result.qualification_pass, false);
  assert.equal(result.pre_failure_settings_continuity_proven, false);
  assert.equal(receipt.ledger.next_ordinal, 17);
  assert.equal(receipt.ledger.total_charged_ms, 1380000);
  assert.equal(await fileDigest(resolve(f.failedRoot, "failed-inventory.json")), failedHash);
  await assert.rejects(createStartupRecoverySupervisor({ privateRoot: f.root }, f.operations), { code: "private_path_exists" });
});

test("changed ledger, missing final release or session loss cannot yield a recovery receipt", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["ledger", "final", "session", "cleanup"]) {
    const f = await completeRecoveryFixture(t),
      name =
        mode === "ledger"
          ? "no-mining-accounting-after.json"
          : mode === "cleanup"
            ? "host-cleanup.json"
            : mode === "final"
              ? "no-mining-state-0003.json"
              : "no-mining-state-0002.json";
    const path = resolve(f.root, name),
      value = await readJson(path);
    if (mode === "ledger") value.ledger.total_charged_ms++;
    if (mode === "final") value.state.serialOwnershipReleased = false;
    if (mode === "session") value.state.connected = false;
    if (mode === "cleanup") value.owned_children_absent = false;
    await writeFile(path, JSON.stringify(value));
    await assert.rejects(judgeStartupRecovery(f.root, f.cleanupPath, f.operations));
  }
});

test("recovery receipt and original evidence cannot be coherently relabelled or silently extended", async (t) => {
  // Arrange
  const f = await completeRecoveryFixture(t);
  await judgeStartupRecovery(f.root, f.cleanupPath, f.operations);
  const path = resolve(f.root, "result.json"),
    saved = await readJson(path);
  saved.receipt.pre_failure_settings_continuity_proven = true;
  saved.sha256 = digest(JSON.stringify(saved.receipt));
  // Act / Assert
  await writeFile(path, JSON.stringify(saved));
  await assert.rejects(readStartupRecovery(path, f.operations), { code: "startup_recovery_result_shape" });
  const g = await completeRecoveryFixture(t);
  await judgeStartupRecovery(g.root, g.cleanupPath, g.operations);
  await writeNew(resolve(g.root, "extra.json"), {});
  await assert.rejects(readStartupRecovery(resolve(g.root, "result.json"), g.operations), { code: "startup_recovery_evidence_changed" });
});

test("startup failure anchor, campaign mismatch, partial assignment and failed recovery all fail closed", async (t) => {
  // Arrange / Act / Assert
  const f = await startupFixture(t),
    noAnchor = { ...f.operations };
  delete noAnchor.expectedStartupFailureSha256;
  await assert.rejects(readStartupFailure(f.failedRoot, noAnchor), { code: "startup_failure_anchor" });
  await writeFile(
    f.options.originalCampaignRecord,
    JSON.stringify({ schema: "fixed-usb-campaign-v1", campaign_id: Buffer.alloc(16, 9).toString("base64url") }),
  );
  await assert.rejects(startupRecoveryPreflight(f.options, f.operations), { code: "startup_original_campaign_mismatch" });
  const g = await startupFixture(t);
  await assert.rejects(
    startupRecoveryPreflight(g.options, {
      ...g.operations,
      mkdir: async () => {
        throw new Error("partial fixture");
      },
    }),
    /partial fixture/u,
  );
  await assert.rejects(startupRecoveryPreflight({ ...g.options, privateRoot: resolve(g.base, "attempts/other") }, g.operations), {
    code: "EEXIST",
  });
  const h = await installedRecovery(t);
  await writeNew(resolve(h.root, "failed-inventory.json"), {});
  await assert.rejects(loadStartupRecoveryContext(h.root, { operations: h.operations }), { code: "private_path_exists" });
});

test("main command boundary excludes all authority and pool inputs without touching the supplied paths", async () => {
  // Arrange / Act / Assert
  for (const command of [
    "cadence-startup-recovery-preflight",
    "cadence-startup-recovery-serve",
    "cadence-startup-recovery-judge",
    "cadence-startup-recovery-review",
    "cadence-startup-recovery-consume-install",
  ]) {
    for (const option of ["--authority-directory", "--pool-credentials"])
      await assert.rejects(main([command, "--private-root", "/missing", option, "/never-read"]), { code: "command_arguments" });
  }
});

test("historical Hello no-mining command still rejects its archived task", async (t) => {
  // Arrange
  const f = await startupFixture(t);
  // Act / Assert
  await assert.rejects(inspectNoMiningSources(f.options, f.operations), { code: "active_task_missing" });
});
