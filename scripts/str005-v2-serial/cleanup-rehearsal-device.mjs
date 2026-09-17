// Synthetic Gate/device input only. The TCP peer is a real socket to the owned
// host fixture, never a Bitaxe; no Serial API, hardware grant or mining exists.
import { connect } from "node:net";
import { performance } from "node:perf_hooks";
import { channelFixture } from "./protocol-judge.test-helper.mjs";
import { deviceState, configuredState } from "./cleanup-rehearsal-inputs.mjs";
import { ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { createV2Coordinator } from "./client.mjs";

export function syntheticDevice(context, request, address) {
  let current = configuredState(context), phase = "before", epoch = 3, idleTime = 100, maybeConnection, maybePrivatePort;
  const vector = channelFixture(); vector.deviceRecords.forEach(record => { record.attemptId = context.attemptId; });
  let coordinator;
  const publish = value => { current = value; coordinator.observe(structuredClone(value)); };
  function status(record = null) {
    return { schema: "worker-stratum-v2-status-v1", scope: "channel", state: record?.state ?? "idle",
      observation: { bootOrdinal: 1, workerGeneration: 2, serialTransportEpoch: epoch,
        observedAtUs: record ? 14000 : ++idleTime, clockValid: true, stationIpv4: address, wifiConnected: true, socket: null },
      connection: record?.state === "terminal" ? maybeConnection : null, record };
  }
  const gate = {
    async reviewQualificationAttempts() { return structuredClone(ledger); },
    async reviewBudget() { return structuredClone(original); },
    async refresh() { publish(current); return current; },
    async configure() { phase = "candidate"; },
    async stratumV2Possession() { return Buffer.alloc(32, 1).toString("base64url"); },
    async stratumV2Status(_scope, attempt) { return status(attempt === null ? null : structuredClone(vector.deviceRecords.at(-1))); },
    async probe() { const probe = { paddingBytes: 65000, requestPayloadBytes: 65536, responsePayloadBytes: 65536 }; publish({ ...current, probe }); return probe; },
    async stop() { publish(deviceState(context, phase)); },
    async close() { publish(deviceState(context, phase, true)); },
    async stratumV2ChannelStart(input) {
      if (input.attemptId !== context.attemptId) throw Error("rehearsal_attempt_mismatch");
      const endpoint = new URL(input.stratum.endpoint);
      maybePrivatePort = Number(endpoint.port);
      const socket = connect({ host: endpoint.hostname, port: maybePrivatePort });
      await new Promise((done, reject) => { socket.once("connect", done); socket.once("error", () => reject(Error("rehearsal_peer_failed"))); });
      maybeConnection = { observedAtUs: 3000, bootOrdinal: 1, workerGeneration: 2, serialTransportEpoch: 3,
        poolSessionGeneration: 4, poolTransportEpoch: 5, socket: { localIpv4: socket.localAddress, localPort: socket.localPort,
          remoteIpv4: socket.remoteAddress, remotePort: socket.remotePort } };
      await new Promise((done, reject) => { socket.once("close", done); socket.once("error", () => reject(Error("rehearsal_peer_failed"))); socket.end("synthetic-protocol-input"); });
      return status(structuredClone(vector.deviceRecords[0]));
    },
    async stratumV2ChannelCancel() { return status(structuredClone(vector.deviceRecords.at(-1))); },
  };
  coordinator = createV2Coordinator({ gate, request, published: () => structuredClone(current), now: () => Math.floor(performance.now()),
    sleep: ms => new Promise(done => setTimeout(done, ms)) });
  return { coordinator, gate, privatePort: () => maybePrivatePort,
    async configured() { publish(configuredState(context)); publish(configuredState(context, true)); await coordinator.supervisor.flush(); },
    async ready() { publish(deviceState(context, phase)); await coordinator.supervisor.flush(); },
    async close() { await gate.close(); await coordinator.supervisor.flush(); },
    async reconnect() { epoch++; await this.ready(); },
  };
}
