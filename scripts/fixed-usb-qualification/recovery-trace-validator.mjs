// Bun loads the parsers from the exact admitted Gate checkout; stdout is closed metadata only.
const { parseBrowserSerialTrace, parseDeviceSerialTrace } = await import(process.argv[2]);
const input = await Bun.stdin.json();
process.stdout.write(JSON.stringify(input.source === "browser" ? parseBrowserSerialTrace(input.trace) : parseDeviceSerialTrace(input.trace)));
