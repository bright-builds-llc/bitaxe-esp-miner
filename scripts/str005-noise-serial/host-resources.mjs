import { execFileSync } from "node:child_process";
import { processSnapshot, sameProcess } from "../host-stalls/capture.mjs";
import { check } from "./files.mjs";
export { processSnapshot, sameProcess };

export function serialNodes(port) {
  check(typeof port === "string" && /^\/dev\/cu\.[A-Za-z0-9._-]+$/u.test(port), "noise_serial_node");
  return [port, port.replace("/dev/cu.", "/dev/tty.")];
}
export function requireNoHolders(port, operations = {}) {
  for (const node of serialNodes(port)) requireLsofAbsent(["-t", node], operations);
}
export function requireLsofAbsent(args, operations = {}) {
  try {
    (operations.execFileSync ?? execFileSync)("/usr/sbin/lsof", args, { encoding: "utf8", timeout: 5000, stdio: ["ignore", "pipe", "pipe"] });
    check(false, "noise_resource_present");
  } catch (error) {
    check(error.status === 1 && error.signal == null && String(error.stdout ?? "").trim() === "" &&
      String(error.stderr ?? "").trim() === "", "noise_resource_unproved");
  }
}
export async function requireGone(owners, operations = {}) {
  check(Array.isArray(owners) && owners.length > 0 && owners.length <= 512, "noise_owner_inventory");
  const current = await (operations.processSnapshot ?? processSnapshot)();
  for (const owner of owners) {
    check(Number.isSafeInteger(owner.pid) && owner.pid > 0 && Number.isSafeInteger(owner.pgid) && owner.pgid > 0 &&
      typeof owner.startedAt === "string" && owner.startedAt.length > 0, "noise_owner_identity");
    check(!current.some((row) => sameProcess(owner, row) || row.ppid === owner.pid || row.pgid === owner.pgid), "noise_owner_remains");
  }
}
