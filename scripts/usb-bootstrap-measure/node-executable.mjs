import { open, realpath, lstat } from "node:fs/promises";
import { isAbsolute } from "node:path";
import { check } from "./values.mjs";
/** Resolve the native runtime behind Bazel's shell shim before binding its digest. */
export async function actualNodePath({ environment = process.env, executable = process.execPath } = {}) {
  const candidate = environment.JS_BINARY__NODE_BINARY ?? executable;
  check(typeof candidate === "string" && isAbsolute(candidate), "bootstrap_node_executable");
  const path = await realpath(candidate), stat = await lstat(path);
  check(stat.isFile() && (stat.mode & 0o111) !== 0, "bootstrap_node_executable");
  const file = await open(path, "r"), header = Buffer.alloc(4);
  try { const { bytesRead } = await file.read(header, 0, 4, 0); check(bytesRead === 4 && ["7f454c46", "cffaedfe", "feedfacf", "cefaedfe", "feedface", "cafebabe", "bebafeca"].includes(header.toString("hex")), "bootstrap_node_executable"); }
  finally { await file.close(); }
  return path;
}
