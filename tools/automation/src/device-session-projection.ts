import { readFile } from "node:fs/promises";

export type JsonObject = Readonly<Record<string, unknown>>;

export type DeviceSessionProjectionFailure = {
  readonly category: "hardware_blocked" | "evidence_invalid";
  readonly message: string;
  readonly facts: Readonly<Record<string, unknown>>;
};

const fields = new Set([
  "schema_version", "terminal_category", "platform_category", "board_category",
  "same_physical_device", "stable_enumeration", "reenumerated", "reader_armed",
  "pre_restart_serial_delivery", "post_restart_serial_delivery", "serial_delivery",
  "request_outcome", "request_attempt_count", "service_loss_observed",
  "trusted_origin_preserved", "application_recovered", "build_identity_matches",
  "boot_session_changed", "boot_ordinal_advanced_by_one", "software_reset_observed",
  "postcondition_matches", "cleanup_complete", "usb_disappearance_count",
  "enumeration_change_count", "serial_byte_count", "http_observation_count",
  "duration_millis",
]);

const requiredBooleans = [
  "same_physical_device", "stable_enumeration", "reenumerated", "reader_armed",
  "pre_restart_serial_delivery", "post_restart_serial_delivery", "service_loss_observed",
  "trusted_origin_preserved", "application_recovered", "build_identity_matches",
  "boot_session_changed", "boot_ordinal_advanced_by_one", "software_reset_observed",
  "postcondition_matches", "cleanup_complete",
] as const;

const requiredCounts = [
  "request_attempt_count", "usb_disappearance_count", "enumeration_change_count",
  "serial_byte_count", "http_observation_count", "duration_millis",
] as const;

function projectionFailure(
  category: DeviceSessionProjectionFailure["category"],
  message: string,
  facts: Readonly<Record<string, unknown>> = {},
): DeviceSessionProjectionFailure {
  return { category, message, facts };
}

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw projectionFailure("evidence_invalid", "device-session projection must be an object");
  }
  return value as JsonObject;
}

export function parseClosedDeviceSession(value: unknown): JsonObject {
  const projection = object(value);
  const keys = Object.keys(projection);
  if (keys.length !== fields.size || keys.some((key) => !fields.has(key))) {
    throw projectionFailure("evidence_invalid", "device-session projection fields are invalid");
  }
  if (projection["schema_version"] !== "esp-device-session-v1") {
    throw projectionFailure("evidence_invalid", "device-session projection schema is invalid");
  }
  const terminalCategory = projection["terminal_category"];
  if (typeof terminalCategory !== "string" || terminalCategory === "") {
    throw projectionFailure("evidence_invalid", "device-session terminal category is invalid");
  }
  const validPlatform = ["macos", "linux", "windows", "other"].includes(String(projection["platform_category"]));
  const validSerialDelivery = ["correlated", "silent", "reacquired", "failed"].includes(String(projection["serial_delivery"]));
  const validCounts = requiredCounts.every((field) => {
    const candidate = projection[field];
    return typeof candidate === "number" && Number.isSafeInteger(candidate) && candidate >= 0;
  });
  if (
    projection["board_category"] !== "205"
    || !validPlatform
    || !validSerialDelivery
    || requiredBooleans.some((field) => typeof projection[field] !== "boolean")
    || !validCounts
  ) {
    throw projectionFailure("evidence_invalid", "device-session projection values are invalid");
  }
  if (terminalCategory !== "ready") {
    throw projectionFailure("hardware_blocked", "device-session did not become ready", {
      terminal_category: terminalCategory,
    });
  }
  const requiredTrue = [
    "same_physical_device", "reader_armed", "trusted_origin_preserved",
    "application_recovered", "build_identity_matches", "boot_session_changed",
    "boot_ordinal_advanced_by_one", "software_reset_observed", "postcondition_matches",
    "cleanup_complete",
  ];
  const requestOutcome = projection["request_outcome"];
  if (
    projection["platform_category"] !== "macos"
    || projection["request_attempt_count"] !== 1
    || (requestOutcome !== "response_received" && requestOutcome !== "response_missing")
    || requiredTrue.some((field) => projection[field] !== true)
  ) {
    throw projectionFailure("evidence_invalid", "ready device-session projection is incomplete");
  }
  return projection;
}

export async function readClosedDeviceSession(output: string): Promise<JsonObject> {
  try {
    return parseClosedDeviceSession(JSON.parse(await readFile(output, "utf8")));
  } catch (error) {
    if (isDeviceSessionProjectionFailure(error)) throw error;
    throw projectionFailure("evidence_invalid", "device-session projection is missing or malformed");
  }
}

export function isDeviceSessionProjectionFailure(error: unknown): error is DeviceSessionProjectionFailure {
  if (typeof error !== "object" || error === null || Array.isArray(error)) return false;
  const maybeFailure = error as Partial<DeviceSessionProjectionFailure>;
  return (maybeFailure.category === "hardware_blocked" || maybeFailure.category === "evidence_invalid")
    && typeof maybeFailure.message === "string"
    && typeof maybeFailure.facts === "object"
    && maybeFailure.facts !== null;
}
