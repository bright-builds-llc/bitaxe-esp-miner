// Pure rules for the logs, update, command and provenance workflows.
//
// Mirrors `firmware/bitaxe/static/www/assets/app.js` and `provenance.js`.
// Upstream AxeOS breadcrumbs (feature reference only, never copied):
// `reference/esp-miner/main/http_server/axe-os/src/app/components/logs/` for
// the live log view and `.../components/settings/` for the exact-filename
// firmware and www update uploads.

/** Characters of log text kept in memory, matching the current variant. */
export const LOG_TEXT_LIMIT = 60000;

/** Appends streamed text and keeps only the newest `LOG_TEXT_LIMIT` characters. */
export function appendBoundedLog(existing: string, text: string, limit = LOG_TEXT_LIMIT): string {
  const combined = `${existing}${text}`;
  return combined.slice(Math.max(0, combined.length - limit));
}

/** Log text shown for a substring filter; an empty filter shows everything. */
export function visibleLogText(text: string, filter: string): string {
  const visible = filter === ""
    ? text
    : text.split("\n").filter((line) => line.includes(filter)).join("\n");
  return visible || "No matching logs.";
}

export type UploadKind = "firmware" | "www";

type UploadRule = Readonly<{
  fileName: string;
  statusName: string;
  wrongFile: string;
  confirm: string;
  progress: string;
  success: string;
  reloadAfterMs: number | null;
}>;

/** Exact-filename upload rules; each upload also needs a separate `confirm()`. */
export const UPLOAD_RULES: Readonly<Record<UploadKind, UploadRule>> = Object.freeze({
  firmware: Object.freeze({
    fileName: "esp-miner.bin",
    statusName: "update",
    wrongFile: "Choose a file named esp-miner.bin.",
    confirm: "Upload this firmware image and restart the device?",
    progress: "Uploading firmware. Do not disconnect power.",
    success: "Firmware accepted. The device will restart.",
    reloadAfterMs: null,
  }),
  www: Object.freeze({
    fileName: "www.bin",
    statusName: "www-update",
    wrongFile: "Choose a file named www.bin.",
    confirm: "Erase and replace the AxeOS web interface with this image?",
    progress: "Uploading AxeOS. Do not disconnect power.",
    success: "AxeOS updated. The page will reload in a few seconds.",
    reloadAfterMs: 2000,
  }),
});

export function isAdmittedUpload(kind: UploadKind, maybeFileName: string | null | undefined): boolean {
  return maybeFileName === UPLOAD_RULES[kind].fileName;
}

export type CommandName = "pause" | "resume" | "restart";

export const COMMANDS: readonly CommandName[] = Object.freeze(["pause", "resume", "restart"]);

export function isCommandName(value: unknown): value is CommandName {
  return typeof value === "string" && (COMMANDS as readonly string[]).includes(value);
}

/** Confirmation prompt shown before a device command is sent. */
export function commandPrompt(name: CommandName): string {
  if (name === "restart") return "Restart the device now?";
  return `${name === "pause" ? "Pause" : "Resume"} mining?`;
}

const COMMIT_PATTERN = /^[0-9a-f]{12,40}$/iu;
const COMMIT_URL = "https://github.com/bright-builds-llc/bitaxe-esp-miner/commit/";
const TIMESTAMP_PATTERN = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/u;

export type Provenance = Readonly<{ version: string; commitLabel: string; commitUrl: string; built: string }>;

/** Firmware provenance from `/api/system/info`, or `null` so the UI shows `Unavailable`. */
export function maybeValidatedProvenance(payload: unknown): Provenance | null {
  if (!payload || typeof payload !== "object") return null;
  const { semanticVersion, sourceCommit, buildTimestampUtc, sourceDirty } = payload as Record<string, unknown>;
  if (
    typeof semanticVersion !== "string" || semanticVersion.trim() === ""
    || typeof sourceCommit !== "string" || !COMMIT_PATTERN.test(sourceCommit)
    || typeof buildTimestampUtc !== "string" || !TIMESTAMP_PATTERN.test(buildTimestampUtc)
    || typeof sourceDirty !== "boolean"
  ) {
    return null;
  }
  return Object.freeze({
    version: semanticVersion,
    commitLabel: `${sourceCommit.slice(0, 12)}${sourceDirty ? " (dirty)" : ""}`,
    commitUrl: `${COMMIT_URL}${sourceCommit}`,
    built: buildTimestampUtc,
  });
}
