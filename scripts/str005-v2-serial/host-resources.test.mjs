import assert from "node:assert/strict";
import test from "node:test";
import { createServer } from "node:net";
import { LISTENER_ARGS, parseListenerInventory, requirePoolListenerAbsent } from "./host-resources.mjs";

test("listener parser supports repeated PID fields, wildcard and IPv6 names", () => {
  const parsed = parseListenerInventory("p123\nn127.0.0.1:80\nn[::1]:81\np123\nn*:82\np456\nn[fe80::1%en0]:83\n");
  assert.deepEqual(parsed.map((entry) => entry.port), [80, 81, 82, 83]);
  for (const value of ["", "n*:80\n", "p1\ncunknown\nn*:80\n", "p1\nn999.1.1.1:80\n", "p1\nn*:65536\n", "p1\nn127.0.0.1:80->127.0.0.2:90\n"])
    assert.throws(() => parseListenerInventory(value));
});

test("private pool port never enters lsof argv or errors", () => {
  const privatePort = 54321, calls = [];
  const operations = { spawnSync(command, args, options) { calls.push({ command, args, options }); return { status: 0, signal: null, stdout: "p1\nn*:80\n", stderr: "" }; } };
  requirePoolListenerAbsent(privatePort, operations);
  assert.deepEqual(calls[0].args, [...LISTENER_ARGS]);
  assert(!JSON.stringify(calls).includes(String(privatePort)));
  assert.throws(() => requirePoolListenerAbsent(privatePort, { spawnSync: () => ({ status: 0, stdout: `p1\nn*:${privatePort}\n`, stderr: "", signal: null }) }),
    (error) => error.code === "v2_pool_listener_present" && !error.message.includes(String(privatePort)));
});

test("lsof unavailable, partial warning and malformed output never prove absence", () => {
  requirePoolListenerAbsent(54321, { spawnSync: () => ({ status: 1, signal: null, stdout: "", stderr: "" }) });
  for (const result of [{ status: 0, stdout: "", stderr: "" }, { status: 1, stdout: "", stderr: "private warning" },
    { status: 0, stdout: "p1\nn*:80\n", stderr: "partial inventory" }, { status: null, signal: "SIGKILL", stdout: "", stderr: "" },
    { error: new Error("private endpoint"), stdout: "", stderr: "" }]) {
    assert.throws(() => requirePoolListenerAbsent(54321, { spawnSync: () => result }));
  }
  assert.throws(() => requirePoolListenerAbsent(54321, { spawnSync: () => { throw new Error("synthetic-private:54321"); } }),
    (error) => error.message === "v2_pool_listener_unproved");
});

test("macOS numeric descriptor records bind exactly one listener name to each file", () => {
  // Arrange / Act
  const result = parseListenerInventory("p123\nf0\nn127.0.0.1:80\nf9\nn[::1]:81\np456\nf0\nn*:82\nf12\nn[fe80::1%en0]:83\n");
  // Assert: no address or descriptor enters the runtime projection.
  assert.deepEqual(result, [{ pid: 123, port: 80 }, { pid: 123, port: 81 }, { pid: 456, port: 82 }, { pid: 456, port: 83 }]);
});

test("repeated PID groups may contain distinct numeric descriptors", () => {
  // Arrange / Act / Assert
  assert.deepEqual(parseListenerInventory("p123\nf4\nn*:80\np123\nf5\nn*:81\n"), [{ pid: 123, port: 80 }, { pid: 123, port: 81 }]);
});

for (const [name, text] of [
  ["file before process", "f4\nn*:80\n"],
  ["empty process group", "p1\np2\nf4\nn*:80\n"],
  ["dangling process", "p1\nf4\nn*:80\np2\n"],
  ["dangling file", "p1\nf4\nn*:80\nf5\n"],
  ["file missing name", "p1\nf4\nf5\nn*:80\n"],
  ["name missing file", "p1\nf4\nn*:80\nn*:81\n"],
  ["duplicate file", "p1\nf4\nn*:80\nf4\nn*:81\n"],
  ["duplicate file across PID groups", "p1\nf4\nn*:80\np1\nf4\nn*:81\n"],
  ["nonnumeric file", "p1\nfcwd\nn*:80\n"],
  ["access suffix", "p1\nf4u\nn*:80\n"],
  ["negative file", "p1\nf-1\nn*:80\n"],
  ["padded file", "p1\nf04\nn*:80\n"],
  ["file overflow", "p1\nf2147483648\nn*:80\n"],
  ["mixed formats", "p1\nn*:80\np2\nf4\nn*:81\n"],
  ["missing descriptor in later process", "p1\nf4\nn*:80\np2\nn*:81\n"],
  ["unknown field", "p1\nf4\ncinventory-command\nn*:80\n"],
  ["malformed address", "p1\nf4\nn999.1.1.1:80\n"],
  ["out-of-range port", "p1\nf4\nn*:65536\n"],
]) test(`descriptor inventory rejects ${name} instead of dropping an unproved record`, () => {
  // Arrange / Act / Assert
  assert.throws(() => parseListenerInventory(text));
});

test("production absence check accepts macOS FD records without exposing the private port", () => {
  // Arrange
  const privatePort = 54321, calls = [];
  const operations = { spawnSync(command, args, options) {
    calls.push({ command, args, options });
    return { status: 0, signal: null, stdout: "p123\nf4\nn127.0.0.1:80\nf9\nn[::1]:81\n", stderr: "" };
  } };
  // Act
  const result = requirePoolListenerAbsent(privatePort, operations);
  // Assert
  assert.equal(result, undefined); assert.deepEqual(calls[0].args, [...LISTENER_ARGS]);
  assert(!JSON.stringify(calls).includes(String(privatePort)));
  assert(!JSON.stringify(calls).includes("127.0.0.1"));
});

test("production FD-aware presence rejection is closed and never prints the port or inventory", () => {
  // Arrange
  const privatePort = 54321;
  const operations = { spawnSync: () => ({ status: 0, signal: null,
    stdout: `p123\nf4\nn127.0.0.1:${privatePort}\n`, stderr: "" }) };
  // Act / Assert
  assert.throws(() => requirePoolListenerAbsent(privatePort, operations), error =>
    error.code === "v2_pool_listener_present" && error.message === "v2_pool_listener_present");
});

test("FD output cannot bypass lsof exit, signal, stderr or spawn-error guards", () => {
  // Arrange
  const stdout = "p123\nf4\nn*:80\n";
  for (const change of [{ status: 1 }, { signal: "SIGTERM" }, { stderr: "partial inventory" }, { error: Error("private detail") }]) {
    const result = { status: 0, signal: null, stdout, stderr: "", ...change };
    // Act / Assert
    assert.throws(() => requirePoolListenerAbsent(54321, { spawnSync: () => result }), { code: "v2_pool_listener_unproved" });
  }
});

function closeLocal(server) {
  return new Promise((done, reject) => server.close(error => error ? reject(Error("local_listener_close_failed")) : done()));
}

test("actual macOS lsof detects a RAM-only local listener and proves its absence after close", { skip: process.platform !== "darwin" }, async t => {
  // Arrange: this is a host-only socket, never a device or mining connection.
  const server = createServer();
  t.after(async () => { if (server.listening) await closeLocal(server); });
  await new Promise((done, reject) => {
    server.once("error", () => reject(Error("local_listener_setup_failed")));
    server.listen(0, "127.0.0.1", done);
  });
  const maybeAddress = server.address();
  assert(maybeAddress && typeof maybeAddress === "object", "local_listener_address_unavailable");
  const privatePort = maybeAddress.port;
  // Act / Assert: production lsof invocation has no private address or port argument.
  assert.throws(() => requirePoolListenerAbsent(privatePort), { code: "v2_pool_listener_present", message: "v2_pool_listener_present" });
  await closeLocal(server);
  assert.equal(requirePoolListenerAbsent(privatePort), undefined);
});
