import assert from "node:assert/strict";
import { mkdtemp, readFile, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import { internalCommandSpec } from "./contracts.generated.js";
import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome } from "./process.js";
import {
  configurationCandidates,
  MacOsProvisioningClient,
  parseDnsResponse,
  ProvisioningClientError,
  type ProvisioningClientBoundary,
  wifiInterfaces,
} from "./provisioning-client.js";

const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

async function testPrivateRoot(name: string): Promise<string> {
  return mkdtemp(path.join(tmpdir(), `bitaxe-corewlan-${name}-`));
}

async function coreWlanResult(
  args: readonly string[],
  status = "ready",
): Promise<ProcessOutcome> {
  const resultIndex = args.indexOf("--result");
  const resultPath = args[resultIndex + 1];
  assert.equal(typeof resultPath, "string");
  assert.equal(args.some((argument) => argument.includes("Bitaxe_")), false);
  await writeFile(resultPath as string, JSON.stringify({
    schema_version: "bitaxe-macos-provisioning-association-result-v1",
    status,
  }), { mode: 0o600 });
  return ok();
}

test("host inventory parsers select only one canonical Wi-Fi interface and unique Bitaxe candidates", () => {
  // Arrange
  const hardware = "Hardware Port: Wi-Fi\nDevice: en0\nEthernet Address: private\n\nHardware Port: Ethernet\nDevice: en1\n";
  const profile = {
    SPAirPortDataType: [{ private: { "Bitaxe_A1B2": { ssid: "Bitaxe_A1B2" }, noise: "Bitaxe_not-valid" } }],
  };

  // Act / Assert
  assert.deepEqual(wifiInterfaces(hardware), ["en0"]);
  assert.deepEqual(configurationCandidates(profile), ["Bitaxe_A1B2"]);
});

test("DNS parser accepts one wildcard A answer with the pinned TTL", () => {
  // Arrange
  const question = Buffer.from([6, ...Buffer.from("net002"), 7, ...Buffer.from("invalid"), 0, 0, 1, 0, 1]);
  const response = Buffer.alloc(12 + question.length + 16);
  response.writeUInt16BE(0x4e02, 0);
  response.writeUInt16BE(0x8180, 2);
  response.writeUInt16BE(1, 4);
  response.writeUInt16BE(1, 6);
  question.copy(response, 12);
  let offset = 12 + question.length;
  response.writeUInt16BE(0xc00c, offset);
  offset += 2;
  response.writeUInt16BE(1, offset);
  response.writeUInt16BE(1, offset + 2);
  response.writeUInt32BE(300, offset + 4);
  response.writeUInt16BE(4, offset + 8);
  Buffer.from([192, 168, 4, 1]).copy(response, offset + 10);

  // Act / Assert
  assert.deepEqual(parseDnsResponse(response, "192.168.4.1"), {
    answerMatchesGateway: true,
    ttlSeconds: 300,
  });
});

test("CoreWLAN helper real child validates a protected fixture without network effects", async () => {
  // Arrange
  const privateRoot = await testPrivateRoot("real-child");
  const intentPath = path.join(privateRoot, "intent.private.json");
  const resultPath = path.join(privateRoot, "result.private.json");
  await writeFile(intentPath, JSON.stringify({
    schema_version: "bitaxe-macos-provisioning-association-intent-v1",
    operation: "fixture",
    interface_name: "fixture-interface",
    configuration_candidate: "Bitaxe_A1B2",
  }), { mode: 0o600 });
  const helperRoot = process.env["TEST_SRCDIR"] === undefined
    ? process.cwd()
    : path.join(process.env["TEST_SRCDIR"] as string, "_main");
  const helper = path.join(helperRoot, "tools/automation/src/macos-provisioning-associate.swift");
  const spec = internalCommandSpec(
    "/usr/bin/xcrun",
    ["swift", helper, "--intent", intentPath, "--result", resultPath],
    (value) => value,
  );
  const processPort = createLocalProcessPort({ cwd: process.cwd(), timeoutMs: 5_000 });

  // Act
  const outcome = await processPort.run(spec, 45_000);

  // Assert
  assert.equal(outcome.exitCode, 0, outcome.stderr);
  assert.equal(outcome.stdout, "");
  assert.equal(outcome.stderr, "");
  assert.equal((await stat(resultPath)).mode & 0o777, 0o600);
  assert.deepEqual(JSON.parse(await readFile(resultPath, "utf8")), {
    schema_version: "bitaxe-macos-provisioning-association-result-v1",
    status: "ready",
  });
  assert.equal(spec.args.some((argument) => argument.includes("Bitaxe_")), false);
});

test("macOS client admits, joins, observes, and restores in strict order", async () => {
  // Arrange
  const calls: string[] = [];
  let candidateScan = 0;
  const processPort = createFakeProcessPort(async (spec) => {
    const command = [spec.program, ...spec.args].join(" ");
    calls.push(command);
    if (command === "networksetup -listallhardwareports") {
      return ok("Hardware Port: Wi-Fi\nDevice: en0\nEthernet Address: private\n");
    }
    if (command === "networksetup -getairportpower en0") return ok("Wi-Fi Power (en0): On\n");
    if (command === "networksetup -getairportnetwork en0") {
      return ok("You are not associated with an AirPort network.\n");
    }
    if (command === "system_profiler SPAirPortDataType -json") {
      candidateScan += 1;
      return ok(JSON.stringify(candidateScan === 1 ? {} : { "Bitaxe_A1B2": {} }));
    }
    if (spec.program === "/usr/bin/xcrun") return coreWlanResult(spec.args);
    if (command === "ipconfig getifaddr en0") return ok("192.168.4.2\n");
    if (command === "ipconfig getoption en0 router") return ok("192.168.4.1\n");
    if (command === "networksetup -setairportpower en0 off") return ok();
    if (command === "networksetup -setairportpower en0 on") return ok();
    throw new Error(`unexpected command ${command}`);
  });
  const fetch = async (input: string | URL | globalThis.Request): Promise<Response> => {
    const target = new URL(String(input));
    return target.pathname === "/api/system/info"
      ? new Response(JSON.stringify({ wifiStatus: "credentials_missing" }), { status: 200 })
      : new Response("Redirect to the captive portal", { status: 302, headers: { location: "/" } });
  };
  const client = new MacOsProvisioningClient(
    processPort,
    "darwin",
    async () => ({ answerMatchesGateway: true, ttlSeconds: 300 }),
    fetch,
  );

  // Act
  const privateRoot = await testPrivateRoot("ready");
  const admission = await client.admit();
  const observation = await client.observe(admission, "Bitaxe_A1B2", privateRoot);
  const restored = await client.cleanup(admission);

  // Assert
  assert.equal(observation.dhcpObserved, true);
  assert.equal(observation.captiveRedirectObserved, true);
  assert.equal(restored, true);
  assert.ok(calls.findIndex((command) => command.startsWith("/usr/bin/xcrun swift "))
    < calls.indexOf("ipconfig getifaddr en0"));
  assert.equal(calls.some((command) => command.includes("Bitaxe_A1B2")), false);
  assert.ok(calls.indexOf("networksetup -setairportpower en0 off")
    < calls.indexOf("networksetup -setairportpower en0 on"));
});

test("host admission rejects association and ambiguous candidates before effects", async () => {
  for (const testCase of [
    { network: "Current Wi-Fi Network: private-existing\n", profile: "{}" },
    { network: "You are not associated with an AirPort network.\n", profile: JSON.stringify({ "Bitaxe_A1B2": {} }) },
  ]) {
    // Arrange
    const port = createFakeProcessPort(async (spec) => {
      if (spec.args[0] === "-listallhardwareports") return ok("Hardware Port: Wi-Fi\nDevice: en0\n");
      if (spec.args[0] === "-getairportpower") return ok("Wi-Fi Power (en0): On\n");
      if (spec.args[0] === "-getairportnetwork") return ok(testCase.network);
      return ok(testCase.profile);
    });
    const client = new MacOsProvisioningClient(port, "darwin");

    // Act / Assert
    await assert.rejects(client.admit());
  }
});

test("macOS client assigns a closed token to every observation boundary", async () => {
  const boundaries: readonly ProvisioningClientBoundary[] = [
    "configuration_candidate",
    "association",
    "dhcp",
    "wildcard_dns",
    "captive_redirect",
    "system_info",
  ];

  for (const boundary of boundaries) {
    // Arrange
    let profileCalls = 0;
    let fetchCalls = 0;
    const port = createFakeProcessPort(async (spec) => {
      const command = [spec.program, ...spec.args].join(" ");
      if (command === "networksetup -listallhardwareports") {
        return ok("Hardware Port: Wi-Fi\nDevice: en0\n");
      }
      if (command === "networksetup -getairportpower en0") return ok("Wi-Fi Power (en0): On\n");
      if (command === "networksetup -getairportnetwork en0") {
        return ok("You are not associated with an AirPort network.\n");
      }
      if (command === "system_profiler SPAirPortDataType -json") {
        profileCalls += 1;
        if (profileCalls === 1) return ok("{}");
        const candidates = boundary === "configuration_candidate"
          ? { "Bitaxe_A1B2": {}, "Bitaxe_C3D4": {} }
          : { "Bitaxe_A1B2": {} };
        return ok(JSON.stringify(candidates));
      }
      if (spec.program === "/usr/bin/xcrun") {
        return coreWlanResult(spec.args, boundary === "association" ? "association_rejected" : "ready");
      }
      if (command.startsWith("ipconfig ")) {
        if (boundary === "dhcp") throw new Error("private DHCP detail");
        return command === "ipconfig getifaddr en0" ? ok("192.168.4.2\n") : ok("192.168.4.1\n");
      }
      throw new Error(`unexpected command ${command}`);
    });
    const dnsQuery = async () => {
      if (boundary === "wildcard_dns") throw new Error("private DNS detail");
      return { answerMatchesGateway: true, ttlSeconds: 300 };
    };
    const fetch = async (): Promise<Response> => {
      fetchCalls += 1;
      if (boundary === "captive_redirect" || (boundary === "system_info" && fetchCalls === 2)) {
        throw new Error("private HTTP detail");
      }
      return fetchCalls === 1
        ? new Response("Redirect to the captive portal", { status: 302, headers: { location: "/" } })
        : new Response("{}", { status: 200 });
    };
    const client = new MacOsProvisioningClient(port, "darwin", dnsQuery, fetch);
    const privateRoot = await testPrivateRoot(boundary);
    const admission = await client.admit();

    // Act / Assert
    await assert.rejects(client.observe(admission, "Bitaxe_A1B2", privateRoot), (error: unknown) => {
      assert.ok(error instanceof ProvisioningClientError);
      assert.equal(error.boundary, boundary);
      assert.doesNotMatch(error.message, /DHCP|DNS|HTTP|Bitaxe|192\.168/u);
      return true;
    });
  }
});

test("CoreWLAN missing malformed timeout and every non-ready subtype stay at association", async () => {
  const cases = [
    "missing",
    "malformed",
    "timeout",
    "input_invalid",
    "interface_unavailable",
    "directed_scan_failed",
    "candidate_absent",
    "candidate_ambiguous",
    "association_rejected",
    "association_not_running",
  ] as const;
  for (const testCase of cases) {
    // Arrange
    const privateRoot = await testPrivateRoot(`failure-${testCase}`);
    let cleanupStarted = false;
    const port = createFakeProcessPort(async (spec) => {
      const command = [spec.program, ...spec.args].join(" ");
      if (command === "networksetup -listallhardwareports") return ok("Hardware Port: Wi-Fi\nDevice: en0\n");
      if (command === "networksetup -getairportpower en0") return ok("Wi-Fi Power (en0): On\n");
      if (command === "networksetup -getairportnetwork en0") {
        return ok("You are not associated with an AirPort network.\n");
      }
      if (command === "system_profiler SPAirPortDataType -json") return ok("{}");
      if (command === "networksetup -setairportpower en0 off") {
        cleanupStarted = true;
        return ok();
      }
      if (command === "networksetup -setairportpower en0 on") return ok();
      if (spec.program === "/usr/bin/xcrun") {
        if (testCase === "timeout") return { ...ok(), exitCode: 1, timedOut: true };
        if (testCase === "missing") return ok();
        const resultPath = spec.args[spec.args.indexOf("--result") + 1];
        assert.equal(typeof resultPath, "string");
        await writeFile(
          resultPath as string,
          testCase === "malformed"
            ? "{"
            : JSON.stringify({ schema_version: "bitaxe-macos-provisioning-association-result-v1", status: testCase }),
          { mode: 0o600 },
        );
        return ok();
      }
      throw new Error(`unexpected command ${command}`);
    });
    const client = new MacOsProvisioningClient(port, "darwin");
    const admission = await client.admit();

    // Act / Assert
    await assert.rejects(client.observe(admission, "Bitaxe_A1B2", privateRoot), (error: unknown) => {
      assert.ok(error instanceof ProvisioningClientError);
      assert.equal(error.boundary, "association");
      return true;
    });
    assert.equal(await client.cleanup(admission), true);
    assert.equal(cleanupStarted, true);
  }
});

test("macOS client joins the exact detector candidate when enumeration omits it", async () => {
  // Arrange
  let profileCalls = 0;
  const port = createFakeProcessPort(async (spec) => {
    const command = [spec.program, ...spec.args].join(" ");
    if (command === "networksetup -listallhardwareports") {
      return ok("Hardware Port: Wi-Fi\nDevice: en0\n");
    }
    if (command === "networksetup -getairportpower en0") return ok("Wi-Fi Power (en0): On\n");
    if (command === "networksetup -getairportnetwork en0") {
      return ok("You are not associated with an AirPort network.\n");
    }
    if (command === "system_profiler SPAirPortDataType -json") {
      profileCalls += 1;
      return ok("{}");
    }
    if (spec.program === "/usr/bin/xcrun") return coreWlanResult(spec.args);
    if (command === "ipconfig getifaddr en0") return ok("192.168.4.2\n");
    if (command === "ipconfig getoption en0 router") return ok("192.168.4.1\n");
    throw new Error(`unexpected command ${command}`);
  });
  const fetch = async (input: string | URL | globalThis.Request): Promise<Response> => {
    return new URL(String(input)).pathname === "/api/system/info"
      ? new Response("{}", { status: 200 })
      : new Response("Redirect to the captive portal", { status: 302, headers: { location: "/" } });
  };
  const client = new MacOsProvisioningClient(
    port,
    "darwin",
    async () => ({ answerMatchesGateway: true, ttlSeconds: 300 }),
    fetch,
  );
  const privateRoot = await testPrivateRoot("invisible");
  const admission = await client.admit();

  // Act
  const observation = await client.observe(admission, "Bitaxe_A1B2", privateRoot);

  // Assert
  assert.equal(observation.associationObserved, true);
  assert.equal(profileCalls, 2);
});
