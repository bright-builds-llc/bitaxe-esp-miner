import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { createRecoveryBootstrap } from "./recovery-client.mjs";
import { configuration } from "./server-assets.mjs";
import { projectRecoveryPart } from "./recovery-evidence.mjs";

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

// Real Gate diagnostic parse/export adds its non-authority field before persistence.
const diagnosticModule = await import(pathToFileURL(resolve(gateRoot, "web/worker-serial-diagnostics.ts")).href);
const diagnosticExport = await import(pathToFileURL(resolve(gateRoot, "web/worker-diagnostic-export.ts")).href);
const boot = diagnosticModule.maybeWorkerSerialDiagnostic("usb_reboot_discriminator schema=v1 boot_ordinal=13 reset_reason=panic uptime_ms=2000 redacted=true");
assert.equal(boot.authoritative, false);
const exported = diagnosticExport.parseWorkerDiagnosticExport({ schema: "worker-diagnostic-export-v1", observations: [boot] });
const projected = projectRecoveryPart("diagnostics", exported, context);
assert.equal(projected.observations.length, 1);
assert.equal(projected.observations[0].boot_ordinal, 13);
assert.equal(projected.authoritative, false);

// Actual Gate control parser/correlation with a synthetic authenticated transport response.
const controlModule = await import(pathToFileURL(resolve(gateRoot, "web/worker-v2-serial-control.ts")).href);
const idle = { schema: "worker-stratum-v2-status-v1", scope: "share", state: "idle", connection: null, record: null,
  observation: { bootOrdinal: 13, workerGeneration: 0, serialTransportEpoch: 1, observedAtUs: 2000000,
    clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } };
let fresh = true, requests = 0;
const control = new controlModule.WorkerV2SerialControl({
  requireScope(scope, effect) { assert.equal(scope, "share"); assert.equal(effect, false); },
  maybeBinding: () => "fresh-binding", possessionFresh: () => fresh,
  async request(command) { assert.equal(command, "stratum_v2_status"); requests++; return idle; },
});
const currentIdle = await control.status("share", null, "fresh-binding");
assert.equal(currentIdle.state, "idle"); assert.equal(currentIdle.record, null);
await assert.rejects(control.status("share", Buffer.alloc(16, 1).toString("base64url"), "fresh-binding"),
  error => error.category === "v2_attempt_correlation");
const beforeStale = requests;
await assert.rejects(control.status("share", null, "old-binding"), error => error.category === "v2_possession");
fresh = false;
await assert.rejects(control.status("share", null, "fresh-binding"), error => error.category === "v2_possession");
assert.equal(requests, beforeStale);
process.stdout.write("gate_configuration_boundary_passed\n");
