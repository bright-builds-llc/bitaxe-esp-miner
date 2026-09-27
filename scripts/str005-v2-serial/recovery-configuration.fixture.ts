import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { createRecoveryBootstrap } from "./recovery-client.mjs";
import { configuration } from "./server-assets.mjs";

const gateRoot = process.argv[2];
const configModule = await import(pathToFileURL(resolve(gateRoot, "web/worker-serial-acceptance-config.ts")).href);
const transitionModule = await import(pathToFileURL(resolve(gateRoot, "web/worker-v2-configuration.ts")).href);
const trust = JSON.parse(await readFile(resolve(gateRoot, "conformance/bwg-worker-deployment-trust-0.2/trust.json"), "utf8"));
const context = { gate_commit: "a".repeat(40), firmware_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64), scope: "share",
  before_source: { firmware_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64) } };
const before = configModule.parseWorkerSerialAcceptanceConfiguration(configuration(context, "before", trust), context.gate_commit);
const candidate = configModule.parseWorkerSerialAcceptanceConfiguration(configuration(context, "candidate", trust), context.gate_commit);
// The real imported Gate boundary reproduces the rejected direct-candidate bootstrap.
assert.throws(() => transitionModule.requireWorkerV2ConfigurationTransition(undefined, candidate), /v2_before_configuration_required/u);
assert.throws(() => configModule.requireWorkerAcceptanceModeTransition(undefined, candidate), /v2_before_configuration_required/u);
configModule.requireWorkerAcceptanceModeTransition(undefined, before);
let currentConfig = before;
const state = { status: "ready", connected: true, running: false, serialOwnershipReleased: false, deviceBaselineConfirmed: true,
  deviceLeaseInactive: true, gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
  expectedAppElfSha256: context.app_elf_sha256, preservation: { baseline_id: "same-page", settings_match: true,
    device_identity_match: true, authorization_high_water_match: true, mine_on_boot: false } };
const calls = [];
const bootstrap = createRecoveryBootstrap({ candidateConfiguration: candidate, published: () => state,
  gate: {
    async close() { calls.push("close"); Object.assign(state, { status: "closed", connected: false, serialOwnershipReleased: true }); },
    configure(input) {
      const next = configModule.parseWorkerSerialAcceptanceConfiguration(input, context.gate_commit);
      configModule.requireWorkerAcceptanceModeTransition(currentConfig, next);
      currentConfig = next; calls.push("configure"); state.status = "configured";
    },
  }, collect: async () => { calls.push("collect"); } });
await bootstrap.prepare();
assert.equal(currentConfig.stratumV2Qualification, "candidate");
await assert.rejects(bootstrap.collect(), /reconnect_required/u);
Object.assign(state, { status: "ready", connected: true, serialOwnershipReleased: false });
await bootstrap.collect();
assert.deepEqual(calls, ["close", "configure", "collect"]);
process.stdout.write("gate_configuration_boundary_passed\n");
