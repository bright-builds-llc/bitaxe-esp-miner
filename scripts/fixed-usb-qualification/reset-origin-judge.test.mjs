import assert from "node:assert/strict";
import { readFile, readdir, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { completeResetOriginFixture } from "./reset-origin-fixtures.mjs";
import { judgeResetOrigin, readResetOrigin } from "./reset-origin-judge.mjs";
import { digest, fileDigest, readJson } from "./contract.mjs";

async function change(f, name, mutate) {
  const path = resolve(f.root, name),
    value = await readJson(path);
  mutate(value);
  await writeFile(path, JSON.stringify(value));
}
async function batches(f, mutate) {
  const names = (await readdir(f.root)).filter((n) => /^diagnostic-export-\d{4}\.json$/u.test(n)).sort();
  for (const name of names) await change(f, name, (v) => mutate(v, Number(name.slice(18, 22))));
}

test("full observation qualifies only as observed_stable and preserves unknown prior panic attribution", async (t) => {
  // Arrange
  const f = await completeResetOriginFixture(t),
    sourceHash = await fileDigest(resolve(f.sourceRoot, "failed-inventory.json"));
  // Act
  const result = await judgeResetOrigin(f.root, f.cleanupPath, f.operations),
    receipt = await readResetOrigin(resolve(f.root, "result.json"), f.operations);
  // Assert
  assert.equal(result.result, "observed_stable");
  assert.equal(result.reset_origin_resolved, false);
  assert.equal(result.prior_reset_attribution, "unknown");
  assert.equal(result.restart_performed, false);
  assert.equal(result.qualification_pass, false);
  assert.equal(result.cadence_admission_authorized, false);
  assert.equal(receipt.summary.initialResetReason, "panic");
  assert(receipt.summary.bootAdvances >= 30);
  assert(receipt.summary.healthyStartupAdvances >= 120);
  assert.equal(await fileDigest(resolve(f.sourceRoot, "failed-inventory.json")), sourceHash);
});

test("snapshot map order does not fabricate unbound startup observations", async (t) => {
  // Arrange
  const f = await completeResetOriginFixture(t);
  await batches(f, (b) => {
    if (b.sequence > 0) b.observations.reverse();
  });
  // Act / Assert
  const result = await judgeResetOrigin(f.root, f.cleanupPath, f.operations);
  assert.equal(result.result, "observed_stable");
});

test("stale snapshots cannot fake device spans, advances or category coverage", async (t) => {
  // Arrange
  const f = await completeResetOriginFixture(t),
    prime = (await readJson(resolve(f.root, "diagnostic-export-0000.json"))).observations;
  await batches(f, (b) => {
    if (b.sequence > 0) b.observations = structuredClone(prime);
  });
  // Act / Assert
  await assert.rejects(judgeResetOrigin(f.root, f.cleanupPath, f.operations), { code: "reset_origin_no_fresh_progress" });
});

test("prime-to-first boot transitions and startup regressions fail even before a second measured boot", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["ordinal", "reset", "uptime", "startup"]) {
    const f = await completeResetOriginFixture(t);
    await change(f, "diagnostic-export-0001.json", (b) => {
      const d = b.observations.find((x) => x.category === (mode === "startup" ? "startup" : "boot"));
      if (mode === "ordinal") d.boot_ordinal++;
      else if (mode === "reset") d.reset_reason = "software_cpu";
      else d.uptime_ms = 500;
    });
    await assert.rejects(judgeResetOrigin(f.root, f.cleanupPath, f.operations));
  }
});

test("host window and per-category progress gaps stay independently bounded", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["short", "long", "gap"]) {
    const f = await completeResetOriginFixture(t);
    if (mode === "gap")
      await batches(f, (b) => {
        if (b.sequence >= 100 && b.sequence <= 120) b.observations.find((d) => d.category === "boot").uptime_ms = 50500;
      });
    else
      await change(f, "reset-origin-end.json", (e) => {
        e.hostMonotonicMs = mode === "short" ? 119200 : 136200;
      });
    await assert.rejects(judgeResetOrigin(f.root, f.cleanupPath, f.operations));
  }
});

test("storage readiness and immutable server claim are required independently of advancing clocks", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["storage", "claim", "private"]) {
    const f = await completeResetOriginFixture(t);
    if (mode === "claim") await rm(resolve(f.root, "reset-origin-server-claim.json"));
    if (mode === "storage") {
      await batches(f, (b) => {
        b.observations = b.observations.filter((d) => d.category !== "storage_http_status");
      });
      await change(f, "reset-origin-start.json", (s) => {
        s.primeObservations = s.primeObservations.filter((d) => d.category !== "storage_http_status");
      });
    }
    if (mode === "private")
      await change(f, "diagnostic-export-0001.json", (b) => {
        b.observations[0].private = "forbidden";
      });
    await assert.rejects(judgeResetOrigin(f.root, f.cleanupPath, f.operations));
  }
});

test("actual panic/allocation receipts and unhealthy startup or storage block observation success", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["panic", "allocation", "startup", "storage"]) {
    const f = await completeResetOriginFixture(t);
    await change(f, "diagnostic-export-0002.json", (b) => {
      if (mode === "panic") b.observations.push({ category: "panic", authoritative: false, file_hash: "12345678", line: 1 });
      if (mode === "allocation")
        b.observations.push({ category: "allocation_failure", authoritative: false, requested_bytes: 16, capabilities: "00000001" });
      if (mode === "startup") {
        const d = b.observations.find((x) => x.category === "startup");
        d.state = "failed";
        d.first_failure = "nvs";
      }
      if (mode === "storage") b.observations.find((x) => x.category === "storage_http_status").http_ready = "false";
    });
    await assert.rejects(judgeResetOrigin(f.root, f.cleanupPath, f.operations));
  }
});

test("disconnection, changed accounting and incomplete cleanup cannot be observation evidence", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["disconnect", "ledger", "cleanup"]) {
    const f = await completeResetOriginFixture(t);
    if (mode === "disconnect")
      await change(f, "no-mining-state-0002.json", (r) => {
        r.state.connected = false;
      });
    if (mode === "ledger")
      await change(f, "no-mining-accounting-after.json", (r) => {
        r.ledger.total_charged_ms++;
      });
    if (mode === "cleanup")
      await change(f, "cleanup.json", (r) => {
        r.serial_holders_absent = false;
      });
    await assert.rejects(judgeResetOrigin(f.root, f.cleanupPath, f.operations));
  }
});

test("an observation receipt cannot be relabelled recovered or given cadence authority", async (t) => {
  // Arrange
  const f = await completeResetOriginFixture(t);
  await judgeResetOrigin(f.root, f.cleanupPath, f.operations);
  const path = resolve(f.root, "result.json"),
    saved = await readJson(path);
  saved.receipt.result = "recovered";
  saved.receipt.cadence_admission_authorized = true;
  saved.sha256 = digest(JSON.stringify(saved.receipt));
  // Act / Assert
  await writeFile(path, JSON.stringify(saved));
  await assert.rejects(readResetOrigin(path, f.operations), { code: "reset_origin_result_changed" });
});

test("minimum advance counts are enforced even when both spans and coverage gaps pass", async (t) => {
  // Arrange / Act / Assert
  for (const category of ["boot", "startup"]) {
    const f = await completeResetOriginFixture(t),
      every = category === "boot" ? 9 : 4;
    await batches(f, (b) => {
      if (b.sequence > 0)
        b.observations.find((d) => d.category === category).uptime_ms = 1000 + Math.floor(b.sequence / every) * every * 500;
    });
    await assert.rejects(judgeResetOrigin(f.root, f.cleanupPath, f.operations), { code: "reset_origin_observation_unqualified" });
  }
});

test("unchanged failed startup history in the prime cannot disappear through deduplication", async (t) => {
  // Arrange
  const f = await completeResetOriginFixture(t);
  const failed = { category: "startup", authoritative: false, stage: "hardware", state: "failed", first_failure: "hardware", uptime_ms: 750 };
  await batches(f, (batch) => { batch.observations.push(failed); });
  await change(f, "reset-origin-start.json", (start) => { start.primeObservations.push(failed); });
  // Act / Assert
  await assert.rejects(judgeResetOrigin(f.root, f.cleanupPath, f.operations), { code: "reset_origin_prime_failure" });
});
