// Test-only cold prerequisites; host ownership and process boundaries remain real.
import { spawn, execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { processSnapshot } from "../str005-noise-serial/host-resources.mjs";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";
export function rehearsalOperations(context, request) {
  return {
    cleanPushed() {}, ignored() {}, hostPlatform: "darwin",
    git: path => path === context.firmware_root ? context.firmware_commit : context.gate_commit,
    nativeSourceFiles: context.native_source_files, nativeAuditorSources: context.native_auditor_sources,
    inspectNative: value => request("inspectNative", [value]),
    readNoiseAnchorProof: (root, name) => request("readNoiseAnchorProof", [root, name]),
    inspectPredecessor: (path, scope) => request("inspectPredecessor", [path, scope]),
    inspectPermissionClosure: path => request("inspectPermissionClosure", [path]),
    inspectChannelSuccessor: path => request("inspectChannelSuccessor", [path]),
    inspectShareSuccessor: path => request("inspectShareSuccessor", [path]),
    checkCurrentShareSuccessorOwnership: value => request("checkCurrentShareSuccessorOwnership", [value]),
    checkCurrentSuccessorOwnership: value => request("checkCurrentSuccessorOwnership", [value]),
    processSnapshot,
    execFileSync(program, args, configuration) {
      // The only simulated OS resource is the nonexistent physical Serial device.
      if (program === "/usr/sbin/lsof" && args[0] === "-t" && /^\/dev\/(?:cu|tty)\.synthetic$/u.test(args[1]))
        throw Object.assign(Error("synthetic_serial_absence"), { status: 1, signal: null, stdout: "", stderr: "" });
      return execFileSync(program, args, configuration);
    },
    spawn(program, args, configuration) {
      // Execute the exact source-bound test fixture bytes with an interpreter.
      // Additional IPC exists only in this helper and closes on parent death.
      return spawn(process.execPath, [program, ...args], {
        ...configuration, stdio: [...configuration.stdio, "ipc"], env: { ...configuration.env, TMPDIR: tmpdir(),
          ...(process.env.TZ === undefined ? {} : { TZ: process.env.TZ }), ...nodeRuntimeEnvironment() },
      });
    },
  };
}
