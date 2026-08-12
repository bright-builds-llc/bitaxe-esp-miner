import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureProvisioningNetworkEvidence,
  ProvisioningNetworkEvidenceError,
  type ProvisioningClientPort,
} from "./provisioning-network-evidence.js";
import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome } from "./process.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

function apLog(): string {
  return [
    sourceCommit,
    referenceCommit,
    appElfSha256,
    "runtime_boot_attestation schema_version=1 mining=disabled work_submission=disabled hardware_control=disabled redacted=true",
    "runtime_heartbeat session=00000000000000000000000000000001 sequence=7 uptime_ms=7000 cadence_ms=1000 listener_armed=false redacted=true",
    "runtime_health boot_session=00000000000000000000000000000001 operator_snapshot_revision=7 redacted=true",
  ].join("\n") + "\n";
}

function recoveryLog(): string {
  return [
    "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
    sourceCommit,
    referenceCommit,
    appElfSha256,
    "wifi_status=connected ipv4=private device_url=private",
  ].join("\n") + "\n";
}

function readyClient(configuration: { observeFails?: boolean; cleanup?: boolean } = {}): ProvisioningClientPort {
  return {
    async admit() { return { interfaceName: "private-interface" }; },
    async observe() {
      if (configuration.observeFails === true) throw new Error("private observation failure");
      return {
        candidateCount: 1,
        associationObserved: true,
        dhcpObserved: true,
        dnsQueryCount: 1,
        wildcardDnsAnswerMatchesGateway: true,
        dnsTtlSeconds: 300,
        captiveRedirectObserved: true,
        captiveRedirectRoot: true,
        captiveRedirectBodyMatches: true,
        systemInfo: {
          wifiStatus: "credentials_missing",
          apEnabled: 1,
          startMiningOnBoot: false,
          sourceCommit,
          referenceCommit,
          appElfSha256,
          hostname: "private-hostname",
        },
      };
    },
    async cleanup() { return configuration.cleanup ?? true; },
  };
}

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-provisioning-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({ source_commit: sourceCommit, reference_commit: referenceCommit, app_elf_sha256: appElfSha256 }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  return {
    root,
    projection: path.join(root, "docs", "provisioning-network.json"),
    options: {
      privateRoot: "scratch/attempt",
      packageManifest: manifest,
      wifiCredentials: credentials,
      port: "/dev/private-port",
      projection: path.join(root, "docs", "provisioning-network.json"),
      captureTimeoutSeconds: 120,
    },
  };
}

async function captureError(promise: Promise<unknown>): Promise<ProvisioningNetworkEvidenceError> {
  try {
    await promise;
    assert.fail("expected capture failure");
  } catch (error) {
    assert.ok(error instanceof ProvisioningNetworkEvidenceError);
    return error;
  }
}

test("late-attached safe runtime emits aggregate-only evidence after the client quorum", async () => {
  // Arrange
  const value = await fixture("ready");
  let flashCount = 0;
  const commands: string[] = [];
  const port = createFakeProcessPort(async (spec) => {
    commands.push(spec.program);
    if (spec.args[0] === "flash-monitor") {
      flashCount += 1;
      return ok(flashCount === 1 ? apLog() : recoveryLog());
    }
    return ok();
  });

  // Act
  const evidence = await captureProvisioningNetworkEvidence(
    value.root, value.options, port, "flash", "validator", readyClient(),
  );
  const projection = await readFile(value.projection, "utf8");

  // Assert
  assert.equal(evidence.provisioning.dns_ttl_seconds, 300);
  assert.equal(evidence.device_recovery_complete, true);
  assert.deepEqual(commands, ["flash", "flash", "validator"]);
  assert.doesNotMatch(
    projection,
    /private-interface|private-hostname|private-port|device_url|ipv4=|wifi\.json/u,
  );
  assert.equal((await stat(path.join(value.root, "scratch", "attempt"))).mode & 0o777, 0o700);
  assert.equal((await stat(path.join(value.root, "scratch", "attempt", "system-info.private.json"))).mode & 0o777, 0o600);
});

test("late-attached runtime without trusted passive safety fails before client observation", async () => {
  // Arrange
  const value = await fixture("missing-safety");
  let flashCount = 0;
  let observationCount = 0;
  const client = readyClient();
  const port = createFakeProcessPort(async (spec) => {
    if (spec.args[0] !== "flash-monitor") return ok();
    flashCount += 1;
    return ok(flashCount === 1
      ? [sourceCommit, referenceCommit, appElfSha256, "runtime_heartbeat redacted=true"].join("\n")
      : recoveryLog());
  });
  const guardedClient: ProvisioningClientPort = {
    admit: client.admit,
    async observe(admission) {
      observationCount += 1;
      return client.observe(admission);
    },
    cleanup: client.cleanup,
  };

  // Act
  const error = await captureError(captureProvisioningNetworkEvidence(
    value.root, value.options, port, "flash", "validator", guardedClient,
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  assert.equal(observationCount, 0);
  assert.equal(flashCount, 2);
  assert.equal(error.publicValue["device_recovery_complete"], true);
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("host admission failure stops before any flash effect", async () => {
  // Arrange
  const value = await fixture("admission");
  let processCount = 0;
  const client: ProvisioningClientPort = {
    async admit() { throw new Error("private host state"); },
    async observe() { throw new Error("unreachable"); },
    async cleanup() { return false; },
  };

  // Act
  const error = await captureError(captureProvisioningNetworkEvidence(
    value.root,
    value.options,
    createFakeProcessPort(async () => { processCount += 1; return ok(); }),
    "flash",
    "validator",
    client,
  ));

  // Assert
  assert.equal(error.category, "hardware_blocked");
  assert.equal(processCount, 0);
});

test("primary observation failure survives cleanup and recovery outcomes", async () => {
  for (const cleanup of [true, false]) {
    // Arrange
    const value = await fixture(`failure-${String(cleanup)}`);
    let flashCount = 0;
    const port = createFakeProcessPort(async (spec) => {
      if (spec.args[0] !== "flash-monitor") return ok();
      flashCount += 1;
      return ok(flashCount === 1 ? apLog() : recoveryLog());
    });

    // Act
    const error = await captureError(captureProvisioningNetworkEvidence(
      value.root, value.options, port, "flash", "validator", readyClient({ observeFails: true, cleanup }),
    ));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    assert.equal(error.publicValue["host_network_restored"], cleanup);
    assert.equal(error.publicValue["device_recovery_complete"], true);
    assert.equal(flashCount, 2);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("validator failure after recovery never starts a second recovery flash", async () => {
  // Arrange
  const value = await fixture("validator");
  let flashCount = 0;
  const port = createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      flashCount += 1;
      return ok(flashCount === 1 ? apLog() : recoveryLog());
    }
    return { ...ok(), exitCode: 9 };
  });

  // Act
  const error = await captureError(captureProvisioningNetworkEvidence(
    value.root, value.options, port, "flash", "validator", readyClient(),
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  assert.equal(flashCount, 2);
  assert.equal(error.publicValue["device_recovery_complete"], true);
});

test("real child stdout drives both flash epochs without invented artifacts", async () => {
  // Arrange
  const value = await fixture("real-child");
  const child = path.join(value.root, "child.sh");
  await writeFile(child, `#!/bin/sh
if [ "$1" = "flash-monitor" ]; then
  case " $* " in
    *" --wifi-credentials "*) cat <<'RECOVERY'
${recoveryLog()}RECOVERY
      ;;
    *) cat <<'AP'
${apLog()}AP
      ;;
  esac
fi
`);
  await chmod(child, 0o700);

  // Act
  const evidence = await captureProvisioningNetworkEvidence(
    value.root,
    value.options,
    createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
    child,
    child,
    readyClient(),
  );

  // Assert
  assert.equal(evidence.schema_version, "bitaxe-provisioning-network-evidence-v1");
  await assert.rejects(readFile(path.join(value.root, "scratch", "attempt", "flash-monitor.log"), "utf8"), { code: "ENOENT" });
});
