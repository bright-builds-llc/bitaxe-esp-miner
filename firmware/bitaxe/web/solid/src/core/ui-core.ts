// Pure operator UI rules shared by every page of the SolidJS variant.
//
// Behavior mirrors the handwritten `current` variant
// (`firmware/bitaxe/static/www/assets/ui-core.js`) so both variants pass the
// same contract tests. Upstream AxeOS breadcrumbs (feature reference only, never
// copied): `reference/esp-miner/main/http_server/axe-os/src/app/app-routing.module.ts`
// for the route names and `.../components/edit/edit.component.ts` for the
// write-only password handling.

/** Operator UI pages in navigation order. */
export const PAGES = [
  "dashboard",
  "network",
  "pool",
  "settings",
  "scoreboard",
  "logs",
  "update",
  "theme",
] as const;

export type Page = (typeof PAGES)[number];

/** History routes; mirrored server-side by `OPERATOR_UI_ROUTES` in `crates/bitaxe-api/src/static_plan.rs`. */
export const ROUTES: Readonly<Record<string, Page>> = Object.freeze({
  "/": "dashboard",
  "/ap": "network",
  "/system": "dashboard",
  "/network": "network",
  "/pool": "pool",
  "/settings": "settings",
  "/scoreboard": "scoreboard",
  "/logs": "logs",
  "/update": "update",
  "/design": "theme",
});

export type PatchKind = "network" | "pool" | "settings";

const PATCH_FIELDS: Readonly<Record<PatchKind, readonly string[]>> = Object.freeze({
  network: Object.freeze(["hostname", "ssid", "wifiPass"]),
  pool: Object.freeze(["stratumProtocol", "stratumURL", "stratumPort", "stratumUser", "stratumPassword"]),
  settings: Object.freeze(["statsFrequency"]),
});
const NUMBER_FIELDS = new Set(["stratumPort", "statsFrequency"]);
const SECRET_FIELDS = new Set(["wifiPass", "stratumPassword"]);
const DEFAULT_ACCENT = "#f7931a";
const ACCENT_PATTERN = /^#[0-9a-f]{6}$/iu;

export type SettingsPatch = Readonly<Record<string, string | number>>;

export function normalizePath(pathname: unknown): string {
  if (typeof pathname !== "string") return "/";
  const path = pathname.split(/[?#]/u, 1)[0] || "/";
  return path.length > 1 && path.endsWith("/") ? path.slice(0, -1) : path;
}

export function routeFor(pathname: unknown): Page {
  return ROUTES[normalizePath(pathname)] ?? "dashboard";
}

export function isKnownRoute(pathname: unknown): boolean {
  return Object.hasOwn(ROUTES, normalizePath(pathname));
}

/** `Bitaxe · Dashboard`, matching the current variant's document titles. */
export function pageTitle(page: Page): string {
  return `Bitaxe · ${page[0]?.toUpperCase() ?? ""}${page.slice(1)}`;
}

function isPatchKind(kind: unknown): kind is PatchKind {
  return typeof kind === "string" && Object.hasOwn(PATCH_FIELDS, kind);
}

/** Builds a write-only PATCH body: blank fields, unknown fields and unsafe numbers are dropped. */
export function buildSettingsPatch(kind: unknown, source: unknown): SettingsPatch {
  if (!isPatchKind(kind) || !source || typeof source !== "object") return Object.freeze({});
  const values = source as Record<string, unknown>;
  const patch: Record<string, string | number> = {};
  for (const field of PATCH_FIELDS[kind]) {
    const rawValue = values[field];
    if (typeof rawValue !== "string") continue;
    const value = rawValue.trim();
    if (value === "") continue;
    if (!NUMBER_FIELDS.has(field)) {
      patch[field] = value;
      continue;
    }
    const number = Number(value);
    if (Number.isSafeInteger(number)) patch[field] = number;
  }
  return Object.freeze(patch);
}

export function patchFieldNames(kind: unknown): readonly string[] {
  return isPatchKind(kind) ? PATCH_FIELDS[kind] : Object.freeze([]);
}

/** Field names that changed, with secrets reduced to `field:updated`. */
export function patchSummary(kind: unknown, patch: SettingsPatch): readonly string[] {
  return Object.freeze(
    patchFieldNames(kind)
      .filter((field) => Object.hasOwn(patch, field))
      .map((field) => (SECRET_FIELDS.has(field) ? `${field}:updated` : field)),
  );
}

export function publicError(error: unknown): string {
  const category = (error as { category?: unknown } | null | undefined)?.category;
  if (category === "timeout") return "The device did not respond before the request timed out.";
  if (category === "http") return "The device rejected the request.";
  if (category === "invalid-response") return "The device returned an invalid response.";
  return "The device is unavailable.";
}

function maybeFiniteNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return days > 0 ? `${days}d ${hours}h` : `${hours}h ${minutes}m`;
}

const UNIT_FORMATTERS: Readonly<Record<string, (value: number) => string>> = Object.freeze({
  hashRate: (value) => `${value.toFixed(1)} GH/s`,
  temp: (value) => `${value.toFixed(1)} °C`,
  power: (value) => `${value.toFixed(1)} W`,
  fanrpm: (value) => `${Math.round(value)} RPM`,
  wifiRSSI: (value) => `${Math.round(value)} dBm`,
  uptimeSeconds: formatUptime,
});

/** Display text for one `/api/system/info` field. */
export function formatMetric(field: string, value: unknown): string {
  const maybeNumber = maybeFiniteNumber(value);
  const maybeFormatter = UNIT_FORMATTERS[field];
  if (maybeFormatter !== undefined) return maybeNumber === null ? "—" : maybeFormatter(maybeNumber);
  if (maybeNumber !== null) return String(maybeNumber);
  return typeof value === "string" && value.trim() !== "" ? value : "Unavailable";
}

export type ScoreboardRow = Readonly<{
  difficulty: number;
  jobId: string;
  extranonce2: string;
  ntime: number;
  nonce: string;
  versionBits: string;
}>;

function isBoundedText(value: unknown): value is string {
  return typeof value === "string" && value.length > 0 && value.length <= 31;
}

function maybeScoreboardRow(entry: unknown): ScoreboardRow | null {
  if (!entry || typeof entry !== "object") return null;
  const candidate = entry as Record<string, unknown>;
  const { difficulty, ntime, nonce } = candidate;
  const versionBits = candidate["version_bits"];
  const valid = typeof difficulty === "number" && Number.isFinite(difficulty) && difficulty > 0
    && isBoundedText(candidate["job_id"]) && isBoundedText(candidate["extranonce2"])
    && Number.isSafeInteger(ntime) && (ntime as number) >= 0
    && typeof nonce === "string" && /^[0-9A-F]{8}$/u.test(nonce)
    && typeof versionBits === "string" && /^[0-9A-F]{8}$/u.test(versionBits);
  if (!valid) return null;
  return Object.freeze({
    difficulty,
    jobId: candidate["job_id"] as string,
    extranonce2: candidate["extranonce2"] as string,
    ntime: ntime as number,
    nonce,
    versionBits,
  });
}

/** Admits only the bounded, exact, descending `/api/system/scoreboard` wire shape. */
export function scoreboardRows(payload: unknown): readonly ScoreboardRow[] {
  if (!Array.isArray(payload) || payload.length > 20) return Object.freeze([]);
  const rows: ScoreboardRow[] = [];
  for (const entry of payload) {
    const maybeRow = maybeScoreboardRow(entry);
    const previous = rows.at(-1)?.difficulty ?? Number.POSITIVE_INFINITY;
    if (maybeRow === null || previous < maybeRow.difficulty) return Object.freeze([]);
    rows.push(maybeRow);
  }
  return Object.freeze(rows);
}

/** Rendered table cells for one scoreboard row. */
export function scoreboardCells(row: ScoreboardRow, index: number): readonly string[] {
  return [
    String(index + 1),
    row.difficulty.toFixed(1),
    row.jobId,
    row.extranonce2,
    String(row.ntime),
    row.nonce,
    row.versionBits,
  ];
}

export type ColorScheme = "dark" | "light";
export type Theme = Readonly<{ scheme: ColorScheme; accent: string }>;

function validAccent(candidate: unknown): string {
  return typeof candidate === "string" && ACCENT_PATTERN.test(candidate) ? candidate : DEFAULT_ACCENT;
}

/** Dark by default; accepts `accentColors.primary` or the legacy `accentColor`. */
export function themeFromPayload(payload: unknown): Theme {
  const source = (payload ?? {}) as { colorScheme?: unknown; accentColor?: unknown; accentColors?: { primary?: unknown } };
  const scheme: ColorScheme = source.colorScheme === "light" ? "light" : "dark";
  return Object.freeze({ scheme, accent: validAccent(source.accentColors?.primary ?? source.accentColor) });
}

export type ThemePayload = Readonly<{ colorScheme: ColorScheme; accentColors: Readonly<{ primary: string }> }>;

export function themePayload(values: unknown): ThemePayload {
  const source = (values ?? {}) as { colorScheme?: unknown; accentColor?: unknown };
  return Object.freeze({
    colorScheme: source.colorScheme === "light" ? "light" : "dark",
    accentColors: Object.freeze({ primary: validAccent(source.accentColor) }),
  });
}
