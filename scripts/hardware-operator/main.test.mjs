import test from "node:test";
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { workspaceFixture } from "./fixtures.test-helper.mjs";
import { hostEnvironment } from "./host.mjs";
import { argumentsFor, commandLine, exitCodeFor, typedCode } from "./main.mjs";

const main = fileURLToPath(new URL("./main.mjs", import.meta.url));
const run = (args, env) => promisify(execFile)(process.execPath, [main, ...args], { env }).then(
  ({ stdout }) => ({ code: 0, result: JSON.parse(stdout) }), (error) => ({ code: error.code, result: JSON.parse(error.stdout) }));

test("both flag spellings parse, including a value that starts with a dash", () => {
  // Arrange
  const argv = ["noise-send", "--private-root=/p/r", "--command", "status", "--timeout-seconds=-1"];
  // Act
  const { action, values } = argumentsFor(argv);
  // Assert
  assert.deepEqual([action, values["private-root"], values.command, values["timeout-seconds"]], ["noise-send", "/p/r", "status", "-1"]);
});

test("send flags become the parent's command line", () => {
  // Arrange
  const values = [{ command: "install", index: "2" }, { command: "browser-closed" }];
  // Act
  const lines = values.map(commandLine);
  // Assert
  assert.deepEqual(lines, ['{"action":"install","index":2}', '{"action":"browser-closed"}']);
});

test("unknown actions, unknown flags, positionals and missing required flags are refused", () => {
  // Arrange
  const cases = [["flash"], ["owner-finish", "--owner", "x", "--private-root", "/r", "--port", "/dev/x"], ["noise-launch", "/r"], ["owner-finish", "--owner", "x"]];
  // Act
  const codes = cases.map((argv) => { try { argumentsFor(argv); return "accepted"; } catch (error) { return error.code; } });
  // Assert
  assert.deepEqual(codes, ["hardware_operator_action", "hardware_operator_arguments", "hardware_operator_arguments", "hardware_operator_arguments"]);
});

test("a failed finish, read, judgement or parent error exits nonzero", () => {
  // Arrange
  const results = [{ event: "owner_finished", finish_exit: 1 }, { event: "core_dump_read_finished", read_exit: 2 },
    { event: "heap_capture_finished", judgement: { passed: false } }, { event: "operator_error", code: "x" }, { event: "owner_finished", finish_exit: 0 }];
  // Act
  const codes = results.map(exitCodeFor);
  // Assert
  assert.deepEqual(codes, [1, 1, 1, 1, 0]);
});

test("only repo snake_case codes are reported; system error codes are not", () => {
  // Arrange / Act
  const codes = [typedCode({ code: "private_path_policy" }), typedCode({ code: "ENOENT" }), typedCode(new Error("x"))];
  // Assert
  assert.deepEqual(codes, ["private_path_policy", null, null]);
});

test("the command prints one JSON line and exits 0 for a passing heap series", async () => {
  // Arrange
  const fixture = await workspaceFixture();
  const env = { ...fixture.env, BUILD_WORKING_DIRECTORY: fixture.base };
  // Act
  const { code, result } = await run(["heap-capture", "--parent", "parent", "--name", "idle", "--seconds", "360",
    "--min-free-bytes", "16384", "--min-largest-block-bytes", "8192", "--min-samples", "2"], env);
  // Assert
  assert.deepEqual([code, result.event, result.judgement.passed], [0, "heap_capture_finished", true]);
});

test("a typed refusal prints its code and exits 2", async () => {
  // Arrange
  const fixture = await workspaceFixture();
  // Act
  const { code, result } = await run(["owner-finish", "--owner", "str005-accepted-share", "--private-root", fixture.root], fixture.env);
  // Assert
  assert.deepEqual([code, result], [2, { event: "hardware_operator_error", code: "owner_record_missing" }]);
});

test("nested commands get the caller's environment without this binary's launcher state", () => {
  // Arrange
  const environment = { PATH: "/bazel/bin/scripts/hardware_operator_node_bin:/usr/local/bin:/usr/bin", HOME: "/h",
    JS_BINARY__NODE_WRAPPER: "/bazel/bin/scripts/hardware_operator_node_bin/node", JS_BINARY__FS_PATCH_ROOTS: "/x", BUILD_WORKSPACE_DIRECTORY: "/w" };
  // Act
  const result = hostEnvironment(environment);
  // Assert
  assert.deepEqual(result, { PATH: "/usr/local/bin:/usr/bin", HOME: "/h", BUILD_WORKSPACE_DIRECTORY: "/w" });
});
