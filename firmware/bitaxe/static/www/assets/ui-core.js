(function installBitaxeUiCore(global) {
  "use strict";

  const ROUTES = Object.freeze({
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
  const PATCH_FIELDS = Object.freeze({
    network: Object.freeze(["hostname", "ssid", "wifiPass"]),
    pool: Object.freeze([
      "stratumProtocol",
      "stratumURL",
      "stratumPort",
      "stratumUser",
      "stratumPassword",
    ]),
    settings: Object.freeze(["statsFrequency"]),
  });
  const NUMBER_FIELDS = new Set(["stratumPort", "statsFrequency"]);
  const SECRET_FIELDS = new Set(["wifiPass", "stratumPassword"]);

  function normalizePath(pathname) {
    if (typeof pathname !== "string") {
      return "/";
    }
    const path = pathname.split(/[?#]/u, 1)[0] || "/";
    if (path.length > 1 && path.endsWith("/")) {
      return path.slice(0, -1);
    }
    return path;
  }

  function routeFor(pathname) {
    return ROUTES[normalizePath(pathname)] ?? "dashboard";
  }

  function isKnownRoute(pathname) {
    return Object.hasOwn(ROUTES, normalizePath(pathname));
  }

  function buildSettingsPatch(kind, source) {
    const fields = PATCH_FIELDS[kind];
    if (!fields || !source || typeof source !== "object") {
      return Object.freeze({});
    }

    const patch = {};
    for (const field of fields) {
      const rawValue = source[field];
      if (typeof rawValue !== "string") {
        continue;
      }
      const value = rawValue.trim();
      if (value === "") {
        continue;
      }
      if (NUMBER_FIELDS.has(field)) {
        const number = Number(value);
        if (!Number.isSafeInteger(number)) {
          continue;
        }
        patch[field] = number;
        continue;
      }
      patch[field] = value;
    }
    return Object.freeze(patch);
  }

  function patchFieldNames(kind) {
    return PATCH_FIELDS[kind] ?? Object.freeze([]);
  }

  function patchSummary(kind, patch) {
    const allowed = PATCH_FIELDS[kind] ?? [];
    return Object.freeze(
      allowed
        .filter((field) => Object.hasOwn(patch, field))
        .map((field) => (SECRET_FIELDS.has(field) ? `${field}:updated` : field)),
    );
  }

  function publicError(error) {
    const category = error?.category;
    if (category === "timeout") {
      return "The device did not respond before the request timed out.";
    }
    if (category === "http") {
      return "The device rejected the request.";
    }
    if (category === "invalid-response") {
      return "The device returned an invalid response.";
    }
    return "The device is unavailable.";
  }

  function finiteNumber(value) {
    return typeof value === "number" && Number.isFinite(value) ? value : null;
  }

  function formatMetric(field, value) {
    const number = finiteNumber(value);
    if (field === "hashRate") {
      return number === null ? "—" : `${number.toFixed(1)} GH/s`;
    }
    if (field === "temp") {
      return number === null ? "—" : `${number.toFixed(1)} °C`;
    }
    if (field === "power") {
      return number === null ? "—" : `${number.toFixed(1)} W`;
    }
    if (field === "fanrpm") {
      return number === null ? "—" : `${Math.round(number)} RPM`;
    }
    if (field === "wifiRSSI") {
      return number === null ? "—" : `${Math.round(number)} dBm`;
    }
    if (field === "uptimeSeconds") {
      if (number === null) {
        return "—";
      }
      const days = Math.floor(number / 86400);
      const hours = Math.floor((number % 86400) / 3600);
      const minutes = Math.floor((number % 3600) / 60);
      return days > 0 ? `${days}d ${hours}h` : `${hours}h ${minutes}m`;
    }
    if (number !== null) {
      return String(number);
    }
    return typeof value === "string" && value.trim() !== ""
      ? value
      : "Unavailable";
  }

  function scoreboardRows(payload) {
    if (!Array.isArray(payload) || payload.length > 20) {
      return Object.freeze([]);
    }
    const rows = [];
    for (const entry of payload) {
      const valid = entry && typeof entry === "object"
        && Number.isFinite(entry.difficulty) && entry.difficulty > 0
        && typeof entry.job_id === "string" && entry.job_id.length > 0 && entry.job_id.length <= 31
        && typeof entry.extranonce2 === "string" && entry.extranonce2.length > 0 && entry.extranonce2.length <= 31
        && Number.isSafeInteger(entry.ntime) && entry.ntime >= 0
        && /^[0-9A-F]{8}$/u.test(entry.nonce)
        && /^[0-9A-F]{8}$/u.test(entry.version_bits);
      if (!valid || (rows.at(-1)?.difficulty ?? Number.POSITIVE_INFINITY) < entry.difficulty) {
        return Object.freeze([]);
      }
      rows.push(Object.freeze({
        difficulty: entry.difficulty,
        jobId: entry.job_id,
        extranonce2: entry.extranonce2,
        ntime: entry.ntime,
        nonce: entry.nonce,
        versionBits: entry.version_bits,
      }));
    }
    return Object.freeze(rows);
  }

  function themeFromPayload(payload) {
    const scheme = payload?.colorScheme === "light" ? "light" : "dark";
    const candidate = payload?.accentColors?.primary ?? payload?.accentColor;
    const accent = typeof candidate === "string" && /^#[0-9a-f]{6}$/iu.test(candidate)
      ? candidate
      : "#f7931a";
    return Object.freeze({ scheme, accent });
  }

  function themePayload(values) {
    const scheme = values?.colorScheme === "light" ? "light" : "dark";
    const accent = typeof values?.accentColor === "string" && /^#[0-9a-f]{6}$/iu.test(values.accentColor)
      ? values.accentColor
      : "#f7931a";
    return Object.freeze({
      colorScheme: scheme,
      accentColors: Object.freeze({ primary: accent }),
    });
  }

  global.BitaxeUiCore = Object.freeze({
    buildSettingsPatch,
    formatMetric,
    isKnownRoute,
    normalizePath,
    patchFieldNames,
    patchSummary,
    publicError,
    routeFor,
    scoreboardRows,
    themeFromPayload,
    themePayload,
  });
})(globalThis);
