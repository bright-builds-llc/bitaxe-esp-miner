// Break diagnostics for the local mining soaks: when mining resumes after a break, read the device's
// retained log at once, before its ring rolls the break's lines away (about 40 minutes of lines),
// keep the raw log privately, and journal only the value-free Wi-Fi reconnect and local mining
// lifecycle lines around the break. Raw lines can hold the station address, so nothing else from
// the log reaches the journal.

/** Line prefixes that carry only categories, ordinals and uptimes. */
const PREFIX = /^(wifi_reconnect=|wifi_reconnect_probe=|local_mining=)/u;
// No `:` or `.` in values, so neither a MAC nor an address can pass, even in a future marker.
const MARKER = /^(wifi_reconnect=|wifi_reconnect_probe=|local_mining=)[a-z_]+( [a-z_]+=[A-Za-z0-9_()-]+)*$/u;
/** Lines this far before the break's first idle sample still explain it. */
export const BREAK_LOG_LEAD_MS = 60_000;
export const BREAK_LOG_MAXIMUM_LINES = 40;

const uptimeOf = (line) => {
  const match = / uptime_ms=(\d+)/u.exec(line);
  return match === null ? null : Number(match[1]);
};

/**
 * The value-free marker lines from a retained log whose uptime falls in the break, from
 * `BREAK_LOG_LEAD_MS` before `fromUptimeMs` through `toUptimeMs`. Anything with an address or an
 * unexpected shape is dropped.
 */
export function breakMarkers(text, fromUptimeMs, toUptimeMs) {
  return breakLog(text, fromUptimeMs, toUptimeMs).markers;
}

/**
 * The break's marker lines, and how many lines with a marker prefix the shape check dropped, so a
 * marker in an unexpected form is counted rather than lost without a trace.
 */
export function breakLog(text, fromUptimeMs, toUptimeMs) {
  const inWindow = (line) => {
    const maybeUptime = uptimeOf(line);
    return maybeUptime !== null && maybeUptime >= fromUptimeMs - BREAK_LOG_LEAD_MS && maybeUptime <= toUptimeMs;
  };
  const prefixed = text.split(/\r?\n/u).map((line) => line.trim()).filter((line) => PREFIX.test(line) && inWindow(line));
  const markers = prefixed.filter((line) => MARKER.test(line) && !/\d+\.\d+\.\d+\.\d+/u.test(line));
  return { markers: markers.slice(-BREAK_LOG_MAXIMUM_LINES), dropped: prefixed.length - markers.length };
}

/**
 * Watches the samples for breaks in mining and reads the retained log when one ends. `readLogs`
 * returns the raw log text and `keepLogs` stores it privately; both may fail without stopping the
 * soak, which only notes the failure.
 */
export function breakWatcher({ readLogs, keepLogs, note }) {
  let maybeBreakFrom = null, sawMining = false, captures = 0;
  return {
    /** Called with each sample in order, after its own live checks. */
    async observe(sample, isMining) {
      if (!isMining) {
        if (sawMining && maybeBreakFrom === null && sample.uptimeSeconds !== null) maybeBreakFrom = sample.uptimeSeconds * 1000;
        return;
      }
      sawMining = true;
      if (maybeBreakFrom === null) return;
      const from = maybeBreakFrom;
      maybeBreakFrom = null;
      captures += 1;
      try {
        const text = await readLogs();
        await keepLogs(text, captures);
        const toUptime = sample.uptimeSeconds === null ? Number.MAX_SAFE_INTEGER : sample.uptimeSeconds * 1000;
        const { markers, dropped } = breakLog(text, from, toUptime);
        await note({ event: "break_log", capture: captures, atMs: sample.atMs, markers, dropped });
      } catch {
        await note({ event: "break_log_failed", capture: captures, atMs: sample.atMs });
      }
    },
    /** A device that could not be reached is a break too, from the last reachable sample. */
    unreachable(maybeLastSample) {
      if (sawMining && maybeBreakFrom === null && maybeLastSample?.uptimeSeconds != null) {
        maybeBreakFrom = maybeLastSample.uptimeSeconds * 1000;
      }
    },
  };
}
