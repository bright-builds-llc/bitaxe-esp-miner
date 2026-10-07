import test from "node:test";
import assert from "node:assert/strict";
import { link } from "node:fs/promises";
import { readdir } from "node:fs/promises";
import { SCENARIOS } from "./contract.mjs";
import { context, passingInput, privateDirectory } from "./fixtures.test-helper.mjs";
import { judgeScenario } from "./judge.mjs";
import { buildProjection, FACT_ALLOWLIST, publishProjectionSet, validateProjection } from "./projection.mjs";

const throwsWith = (operation, code) => assert.throws(operation, (error) => error.code === code);

function projection(scenario) {
  return buildProjection({ attemptId: "bwg007-attempt-001", context, scenarioResult: judgeScenario(passingInput(scenario)),
    recordsSha256: "7".repeat(64), scenarioResultSha256: "8".repeat(64) });
}

test("every passed scenario projects to the closed 0.2 profile with only its allowlisted facts", () => {
  // Arrange / Act
  const projections = SCENARIOS.map(projection);
  // Assert
  for (const value of projections) {
    assert.equal(value.profile, "bwg-worker-restoration-result/0.2");
    assert.deepEqual(Object.keys(value.facts).sort(), [...FACT_ALLOWLIST[value.scenario]].sort());
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
  const projections = SCENARIOS.map(projection);
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
  const projections = SCENARIOS.map(projection);
  await publishProjectionSet(directory, projections);
  const drifted = projections.map((value, index) => index === 3 ? { ...value, appElfSha256: "9".repeat(64) } : value);
  // Act / Assert
  await assert.rejects(publishProjectionSet(directory, projections), (error) => error.code === "projection_exists");
  await assert.rejects(publishProjectionSet(directory, projections.slice(1)), (error) => error.code === "projection_set_incomplete");
  await assert.rejects(publishProjectionSet(await privateDirectory("projection-"), drifted), (error) => error.code === "projection_identity_drift");
});
