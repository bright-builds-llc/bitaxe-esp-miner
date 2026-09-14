import { exactObject, requireCondition as check } from "./contract.mjs";
const COUNTERS = ["stack_caps", "before_free_bytes", "before_largest_block_bytes", "after_free_bytes", "after_largest_block_bytes"];
const u32 = (value) => Number.isSafeInteger(value) && value >= 0 && value <= 0xffffffff;
export function parseStatisticsStartup(value) {
  exactObject(value, ["category", "authoritative", "state", "errno", "stack_bytes", ...COUNTERS]);
  check(
    value.category === "statistics_startup" &&
      value.authoritative === false &&
      ["prepared", "active", "cancelled", "spawn_failed", "config_failed"].includes(value.state) &&
      value.stack_bytes === 8192,
    "statistics_startup_shape",
  );
  check(
    value.errno === "unavailable" || (Number.isSafeInteger(value.errno) && value.errno >= -2147483648 && value.errno <= 2147483647),
    "statistics_startup_errno",
  );
  if (value.state === "config_failed")
    check(value.errno === "unavailable" && COUNTERS.every((key) => value[key] === "unavailable"), "statistics_startup_unavailable");
  else {
    check(
      COUNTERS.every((key) => u32(value[key])) &&
        value.before_largest_block_bytes <= value.before_free_bytes &&
        value.after_largest_block_bytes <= value.after_free_bytes,
      "statistics_startup_heap",
    );
    if (value.state !== "spawn_failed") check(value.errno === "unavailable", "statistics_startup_errno");
  }
  return { ...value };
}
export function parseStatisticsStartupLine(line) {
  const match =
    /^statistics_startup schema=v1 state=(prepared|active|cancelled|spawn_failed|config_failed) errno=(-?[0-9]+|unavailable) stack_bytes=8192 stack_caps=([0-9]+|unavailable) before_free_bytes=([0-9]+|unavailable) before_largest_block_bytes=([0-9]+|unavailable) after_free_bytes=([0-9]+|unavailable) after_largest_block_bytes=([0-9]+|unavailable) redacted=true$/u.exec(
      line,
    );
  check(match, "statistics_startup_line");
  const number = (value) => (value === "unavailable" ? value : Number(value));
  return parseStatisticsStartup({
    category: "statistics_startup",
    authoritative: false,
    state: match[1],
    errno: number(match[2]),
    stack_bytes: 8192,
    ...Object.fromEntries(COUNTERS.map((key, index) => [key, number(match[index + 3])])),
  });
}
/** Lifecycle metadata has no boot, startup-progress, authority or timing credit. */
export function requireActiveStatistics(values) {
  const rows = values.filter((value) => value.category === "statistics_startup").map(parseStatisticsStartup);
  check(rows.length > 0, "statistics_startup_missing");
  let active = false;
  for (const row of rows) {
    check(
      ["prepared", "active"].includes(row.state) && row.stack_caps === 2052 && (!active || row.state === "active"),
      "statistics_startup_not_active",
    );
    if (row.state === "active") active = true;
  }
  check(active && rows.at(-1).state === "active", "statistics_startup_not_active");
  return rows.at(-1);
}
