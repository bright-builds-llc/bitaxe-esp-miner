import { parseNoiseStatusV2 } from "./device-v2.mjs";
import { parseFixtureReady, parseFixtureTerminal } from "./fixture.mjs";
import { parseStartV2 } from "./contract-v2.mjs";
import { canonical, check, digest } from "./files.mjs";

/** Protocol joins only; the caller must independently admit every evidence source. */
export function inspectProtocolV2(startValue, statuses, readyValue, terminalValue) {
  const start = parseStartV2(startValue), ready = parseFixtureReady(readyValue), terminal = parseFixtureTerminal(terminalValue);
  check(statuses.length > 1, "noise_admission_record_missing");
  const first = parseNoiseStatusV2(statuses[0]), last = parseNoiseStatusV2(statuses.at(-1));
  check(first.state === "admitted" && last.state === "terminal" && last.job.terminal.outcome === "accepted" && terminal.outcome === "accepted", "noise_protocol_incomplete");
  const job = last.job;
  check(start.attemptId === first.job.attemptId && start.attemptId === job.attemptId && start.attemptId === ready.attemptId &&
    start.attemptId === terminal.attemptId && job.inputSha256 === digest(canonical(start)), "noise_attempt_join");
  check(start.expectedBootOrdinal === job.bootOrdinal && start.networkObservedAtUs <= job.admittedAtUs &&
    job.admittedAtUs - start.networkObservedAtUs <= 5_000_000, "noise_fresh_network_join");
  check(ready.listenIpv4 === start.fixtureIpv4 && ready.listenPort === start.fixturePort && ready.authorityPublicKey === start.authorityPublicKey,
    "noise_fixture_binding");
  check(job.localSocketPort === terminal.candidates[0].remotePort, "noise_tcp_tuple");
  const milliseconds = (us) => Math.ceil(us / 1000), stage = (name) => job.stages.find((row) => row.stage === name);
  return { timings: { preparation: milliseconds(stage("noise_prepared").durationUs), connect: milliseconds(stage("tcp_connected").durationUs),
    act_one_write: milliseconds(stage("act_one_written").durationUs), act_two_read: milliseconds(stage("act_two_received").durationUs),
    proof_write: milliseconds(stage("proof_written").durationUs), diagnostic: milliseconds(job.terminal.decidedAtUs - job.admittedAtUs),
    device_cleanup: milliseconds(job.resources.releasedAtUs - job.resources.startedAtUs) }, job };
}
