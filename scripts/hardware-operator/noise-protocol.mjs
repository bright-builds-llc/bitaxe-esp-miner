// Pure command and reply rules shared by the Noise-serial operator parent and its launcher.
import { HardwareOperatorError } from "./errors.mjs";

/** The Gate origin port that holds the Ultra 205 Web Serial grant (AGENTS.md, Persistent Gate Browser Tab). */
export const GATE_PORT = 48765;
export const INSTALL_INDEXES = [0, 1, 2, 3, 4];
const ACTIONS = new Set(["install", "status", "browser-closed", "stop", "cleanup", "exit"]);

export const operatorDirectory = (root) => `${root}.operator`;

/** Parse one operator command line; only the parent's closed action set and install indexes 0-4 pass. */
export function parseCommand(line) {
  let command;
  try { command = JSON.parse(line); } catch { throw new HardwareOperatorError("noise_command_json"); }
  if (command === null || typeof command !== "object" || Array.isArray(command) || !ACTIONS.has(command.action))
    throw new HardwareOperatorError("noise_command_action");
  const keys = Object.keys(command).sort().join(",");
  if (command.action === "install") {
    if (keys !== "action,index" || !INSTALL_INDEXES.includes(command.index)) throw new HardwareOperatorError("noise_command_install_index");
    return { action: "install", index: command.index };
  }
  if (keys !== "action") throw new HardwareOperatorError("noise_command_fields");
  return { action: command.action };
}

/** Complete stdout lines only; a partially written trailing line is not yet a reply. */
export function completeLines(text) {
  const lines = text.split("\n");
  lines.pop();
  return lines;
}

/** The single reply to a command sent after `before` complete lines, or null while none is complete. */
export function replyAfter(text, before) {
  const lines = completeLines(text);
  if (lines.length <= before) return null;
  let reply;
  try { reply = JSON.parse(lines[before]); } catch { throw new HardwareOperatorError("noise_reply_json"); }
  if (reply === null || typeof reply !== "object" || typeof reply.event !== "string") throw new HardwareOperatorError("noise_reply_shape");
  return reply;
}

/** The parent's first line decides launch readiness. */
export function readinessOf(text) {
  const reply = replyAfter(text, 0);
  if (reply === null) return null;
  return reply.event === "supervisor_ready" ? { ready: true, reply } : { ready: false, reply };
}
