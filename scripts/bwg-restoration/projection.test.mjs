import test from "node:test";
import assert from "node:assert/strict";
import { link, readdir, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { ATTEMPT_008 } from "./attempt-008.test-helper.mjs";
import { SCENARIOS } from "./contract.mjs";
import { context, passingInput, privateDirectory } from "./fixtures.test-helper.mjs";
import { judgeScenario } from "./judge.mjs";
import { attemptFacts, buildProjection, canonicalJson, FACT_ALLOWLIST, publishProjectionSet, validateProjection } from "./projection.mjs";

const throwsWith = (operation, code) => assert.throws(operation, (error) => error.code === code);
const CLEAN_SCAN = { credential_scan: { files: 12, hits: 0 } };

/** The sealed private scenario results of a passing attempt, as publish reads them. */
const scenarioResults = () => SCENARIOS.map((scenario) => {
  const input = passingInput(scenario);
  return { ...judgeScenario(input), final_state: input.finalState };
});

function projection(scenario, { results = scenarioResults(), sealed = CLEAN_SCAN } = {}) {
  return buildProjection({ attemptId: "bwg007-attempt-001", context, scenarioResult: results[SCENARIOS.indexOf(scenario)],
    attempt: attemptFacts(results, sealed), recordsSha256: "7".repeat(64), scenarioResultSha256: "8".repeat(64) });
}

test("every passed scenario projects to the closed 0.3 profile with only its allowlisted facts", () => {
  // Arrange / Act
  const projections = SCENARIOS.map((scenario) => projection(scenario));
  // Assert
  for (const value of projections) {
    assert.equal(value.profile, "bwg-worker-restoration-result/0.3");
    assert.deepEqual(Object.keys(value.facts).sort(), [...FACT_ALLOWLIST[value.scenario]].sort());
    assert.deepEqual([value.facts.deviceIdentityStable, value.facts.poolConfigurationUnchanged, value.poolConfigurationNeverPersisted],
      [true, true, true]);
    assert.equal(value.facts.sameKeyReacquired, ["disconnect", "reboot"].includes(value.scenario) ? true : undefined);
  }
  assert.equal(projections.at(-1).facts.durableReplayAttributed, true);
  assert.equal(projections[4].facts.monotonicDetectionCounted, true);
});

test("a field outside the allowlist is refused", () => {
  // Arrange
  const value = { ...projection("pause"), note: "fine" };
  // Act / Assert
  throwsWith(() => validateProjection(value), "projection_fields");
});

test("a secret-looking string is refused wherever it appears", () => {
  // Arrange
  const cases = [{ ...projection("pause"), attemptId: "bwg007-attempt-001@pool.invalid" },
    { ...projection("pause"), facts: { ...projection("pause").facts, operatorEndedLease: "stratum+tcp://pool.invalid:3333/" } },
    { ...projection("pause"), terminalReason: "/dev/cu.usbmodem1101" }];
  // Act / Assert
  for (const value of cases) throwsWith(() => validateProjection(value), "projection_private_value");
});

test("an unverified fact or extra fact cannot be published", () => {
  // Arrange
  const base = projection("expiry");
  // Act / Assert
  throwsWith(() => validateProjection({ ...base, facts: { ...base.facts, noRenewal: false } }), "projection_facts");
  throwsWith(() => validateProjection({ ...base, facts: { ...base.facts, renewalAccepted: true } }), "projection_facts");
});

test("publication writes all eight projections or none", async () => {
  // Arrange
  const directory = await privateDirectory("projection-");
  const projections = SCENARIOS.map((scenario) => projection(scenario));
  let links = 0;
  const failing = { link: async (from, to) => { links += 1; if (links === 5) throw new Error("disk_full"); return link(from, to); } };
  // Act
  await assert.rejects(publishProjectionSet(directory, projections, failing), (error) => error.code === "projection_publication_failed");
  const afterFailure = await readdir(directory);
  const targets = await publishProjectionSet(directory, projections);
  // Assert
  assert.deepEqual(afterFailure, []);
  assert.equal(targets.length, 8);
  assert.deepEqual((await readdir(directory)).sort(), SCENARIOS.map((scenario) => `bwg007-attempt-001-${scenario}.json`).sort());
});

test("publication refuses an existing target, a missing scenario and identity drift", async () => {
  // Arrange
  const directory = await privateDirectory("projection-");
  const projections = SCENARIOS.map((scenario) => projection(scenario));
  await publishProjectionSet(directory, projections);
  const drifted = projections.map((value, index) => index === 3 ? { ...value, appElfSha256: "9".repeat(64) } : value);
  // Act / Assert
  await assert.rejects(publishProjectionSet(directory, projections), (error) => error.code === "projection_exists");
  await assert.rejects(publishProjectionSet(directory, projections.slice(1)), (error) => error.code === "projection_set_incomplete");
  await assert.rejects(publishProjectionSet(await privateDirectory("projection-"), drifted), (error) => error.code === "projection_identity_drift");
});

test("the published attempt-008 0.2 projections still validate", async () => {
  // Arrange
  const directory = resolve(dirname(fileURLToPath(import.meta.url)), "../../docs/parity/evidence/bwg-worker-restoration");
  const published = await readdir(directory).catch(() => null);
  // Act
  const validated = Object.values(ATTEMPT_008).map(validateProjection);
  // Assert
  assert.equal(validated.length, 8);
  assert.ok(validated.every((value) => value.profile === "bwg-worker-restoration-result/0.2"));
  // The copy must equal the published bytes wherever the checkout provides them (not inside Bazel's sandbox).
  if (published) {
    for (const [name, value] of Object.entries(ATTEMPT_008)) assert.equal(await readFile(resolve(directory, name), "utf8"), `${canonicalJson(value)}\n`);
  }
});

test("a 0.2 projection cannot carry 0.3 fields and a 0.3 projection cannot omit them", () => {
  // Arrange
  const legacy = ATTEMPT_008["bwg007-attempt-008-reboot.json"];
  const current = projection("reboot");
  const { poolConfigurationNeverPersisted, ...withoutPool } = current;
  // Act / Assert
  throwsWith(() => validateProjection({ ...legacy, poolConfigurationNeverPersisted: true }), "projection_fields");
  throwsWith(() => validateProjection({ ...legacy, facts: { ...legacy.facts, sameKeyReacquired: true } }), "projection_facts");
  throwsWith(() => validateProjection(withoutPool), "projection_fields");
  throwsWith(() => validateProjection({ ...current, profile: "bwg-worker-restoration-result/0.4" }), "projection_identity");
  assert.equal(poolConfigurationNeverPersisted, true);
});

test("a second device identity anywhere in the attempt cannot be published as the same device", () => {
  // Arrange
  const results = scenarioResults();
  results[7] = { ...results[7], final_state: { ...results[7].final_state, deviceIdentity: { epoch: 2, observations: 40 } } };
  const unstable = scenarioResults();
  unstable[2] = { ...unstable[2], facts: { ...unstable[2].facts, deviceIdentityStable: false } };
  // Act
  const facts = [attemptFacts(results, CLEAN_SCAN), attemptFacts(unstable, CLEAN_SCAN)];
  // Assert
  assert.deepEqual(facts.map((value) => value.sameDeviceAcrossScenarios), [false, false]);
  throwsWith(() => projection("pause", { results }), "projection_booleans");
});

test("a scan hit or a missing scan cannot be published as credential absence", () => {
  // Arrange
  const sealed = [{ credential_scan: { files: 12, hits: 1 } }, {}, { credential_scan: { files: 0, hits: 0 } }];
  // Act
  const absent = sealed.map((value) => attemptFacts(scenarioResults(), value).campaignEventCredentialsAbsent);
  // Assert
  assert.deepEqual(absent, [false, false, false]);
  throwsWith(() => projection("cancel", { sealed: sealed[0] }), "projection_booleans");
});

test("a changed pool configuration or missing measured checks cannot be published", () => {
  // Arrange
  const changed = scenarioResults();
  changed[0] = { ...changed[0], facts: { ...changed[0].facts, poolConfigurationUnchanged: false } };
  const unchecked = scenarioResults();
  const { checks, ...withoutChecks } = unchecked[1];
  unchecked[1] = withoutChecks;
  // Act / Assert
  throwsWith(() => projection("completion", { results: changed }), "projection_booleans");
  throwsWith(() => projection("pause", { results: unchecked }), "projection_booleans");
  assert.deepEqual(checks, { cleanupConfirmed: true, baselineConfirmed: true });
});
