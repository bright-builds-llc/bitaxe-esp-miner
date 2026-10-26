import assert from "node:assert/strict";
import test from "node:test";
import { breakLog, breakMarkers, breakWatcher } from "./break-log.mjs";

const LOG = [
  "runtime_health boot_session=abc operator_snapshot_revision=4 redacted=true",
  "wifi_reconnect=disconnected reason=beacon_timeout retry_ordinal=1 fallback=true retry_delay_ms=5000 uptime_ms=100000",
  "wifi_status=connected ipv4=192.168.1.20 device_url=http://192.168.1.20",
  "wifi_reconnect=connected completed_retry_ordinal=1 retry_ordinal=0 fallback=false uptime_ms=106000",
  "local_mining=run_ended generation=7 ending=Some(Requested) uptime_ms=107000",
  "local_mining=run_began generation=8 uptime_ms=130000",
  "local_mining=run_began generation=2 uptime_ms=10000",
].join("\n");

test("only value-free reconnect and local mining lines around the break are kept", () => {
  // Arrange / Act
  const markers = breakMarkers(LOG, 101_000, 131_000);
  // Assert
  assert.deepEqual(markers, [LOG.split("\n")[1], LOG.split("\n")[3], LOG.split("\n")[4], LOG.split("\n")[5]]);
});

test("a line with an address never passes, whatever its prefix", () => {
  // Arrange
  const text = "local_mining=hold reason=192.168.1.20 uptime_ms=100000";
  // Act / Assert
  assert.deepEqual(breakMarkers(text, 100_000, 100_000), []);
});

/** Samples at 2 s from uptime 100 s: mining, `idle` of them idle, then mining. */
function samples(idle) {
  return Array.from({ length: idle + 4 }, (_, index) => ({ atMs: index * 2000, uptimeSeconds: 100 + index * 2,
    mining: index < 2 || index >= idle + 2 }));
}

test("the log is read once when mining resumes after a break, and kept privately", async () => {
  // Arrange
  const kept = [], events = [];
  const watcher = breakWatcher({ readLogs: async () => LOG, keepLogs: async (text, capture) => { kept.push(capture); },
    note: async (event) => { events.push(event); } });
  // Act
  for (const sample of samples(3)) await watcher.observe(sample, sample.mining);
  // Assert
  assert.deepEqual(kept, [1]);
  assert.equal(events.length, 1);
  assert.equal(events[0].event, "break_log");
  assert.ok(!JSON.stringify(events).includes("192.168"));
});

test("a failed log read is noted and never stops the soak", async () => {
  // Arrange
  const events = [];
  const watcher = breakWatcher({ readLogs: async () => { throw new Error("timeout"); }, keepLogs: async () => {},
    note: async (event) => { events.push(event); } });
  // Act
  for (const sample of samples(2)) await watcher.observe(sample, sample.mining);
  // Assert
  assert.deepEqual(events.map((event) => event.event), ["break_log_failed"]);
});

test("an unreachable spell after mining counts as a break", async () => {
  // Arrange
  let reads = 0;
  const watcher = breakWatcher({ readLogs: async () => { reads += 1; return ""; }, keepLogs: async () => {}, note: async () => {} });
  // Act
  await watcher.observe({ atMs: 0, uptimeSeconds: 100 }, true);
  watcher.unreachable({ atMs: 0, uptimeSeconds: 100 });
  await watcher.observe({ atMs: 30_000, uptimeSeconds: 130 }, true);
  // Assert
  assert.equal(reads, 1);
});

test("the firmware's own marker formats all pass", () => {
  // Arrange: the exact shapes local_owner.rs and wifi_adapter/reconnect.rs write.
  const lines = [
    "local_mining=hold reason=local_gate_link uptime_ms=100100",
    "local_mining=handover generation=12 revoked=true uptime_ms=100200",
    "local_mining=begin_refused uptime_ms=100300",
    "local_mining=run_ended generation=12 ending=None uptime_ms=100400",
    "local_mining=run_began generation=13 uptime_ms=100500",
    "wifi_reconnect=attempt_started retry_ordinal=1 uptime_ms=100600",
    "wifi_reconnect_probe=stable completed_retry_ordinal=1 stability_ms=15000 uptime_ms=100700",
  ];
  // Act
  const { markers, dropped } = breakLog(lines.join("\n"), 100_000, 101_000);
  // Assert
  assert.deepEqual([markers.length, dropped], [lines.length, 0]);
});

test("a marker in an unexpected form is counted as dropped", () => {
  // Arrange: a hardware fault context with a space, and a value with a colon.
  const text = "local_mining=hardware_fault context=No pool uptime_ms=100000\nlocal_mining=hold reason=aa:bb uptime_ms=100000";
  // Act
  const { markers, dropped } = breakLog(text, 100_000, 100_000);
  // Assert
  assert.deepEqual([markers, dropped], [[], 2]);
});

test("a run that stops during a break still reads the log once", async () => {
  // Arrange
  let reads = 0;
  const watcher = breakWatcher({ readLogs: async () => { reads += 1; return ""; }, keepLogs: async () => {}, note: async () => {} });
  await watcher.observe({ atMs: 0, uptimeSeconds: 100 }, true);
  await watcher.observe({ atMs: 2000, uptimeSeconds: 102 }, false);
  // Act
  await watcher.stopped(70_000);
  await watcher.stopped(72_000);
  // Assert
  assert.equal(reads, 1);
});

test("a run that stops while mining reads no log", async () => {
  // Arrange
  let reads = 0;
  const watcher = breakWatcher({ readLogs: async () => { reads += 1; return ""; }, keepLogs: async () => {}, note: async () => {} });
  await watcher.observe({ atMs: 0, uptimeSeconds: 100 }, true);
  // Act
  await watcher.stopped(2000);
  // Assert
  assert.equal(reads, 0);
});

test("a held break reads no log while mining", async () => {
  // Arrange
  let reads = 0;
  const watcher = breakWatcher({ readLogs: async () => { reads += 1; return ""; }, keepLogs: async () => {}, note: async () => {} });
  await watcher.observe({ atMs: 0, uptimeSeconds: 100 }, true);
  await watcher.observe({ atMs: 2000, uptimeSeconds: 102 }, false);
  // Act
  await watcher.observe({ atMs: 4000, uptimeSeconds: 104 }, true, true);
  // Assert
  assert.equal(reads, 0);
});

test("a held break is read once mining continues without the hold, from its first idle sample", async () => {
  // Arrange
  const notes = [];
  const log = "local_mining=handover generation=2 revoked=true uptime_ms=101000\nlocal_mining=run_began generation=5 uptime_ms=300000";
  const watcher = breakWatcher({ readLogs: async () => log, keepLogs: async () => {}, note: async (event) => { notes.push(event); } });
  await watcher.observe({ atMs: 0, uptimeSeconds: 100 }, true);
  await watcher.observe({ atMs: 2000, uptimeSeconds: 102 }, false);
  await watcher.observe({ atMs: 4000, uptimeSeconds: 104 }, true, true);
  await watcher.observe({ atMs: 200_000, uptimeSeconds: 290 }, false);
  // Act
  await watcher.observe({ atMs: 210_000, uptimeSeconds: 300 }, true);
  // Assert
  assert.deepEqual(notes.map((event) => [event.event, event.markers]), [["break_log", [
    "local_mining=handover generation=2 revoked=true uptime_ms=101000", "local_mining=run_began generation=5 uptime_ms=300000"]]]);
});
