// Deliberately hostile synthetic child used only for host process-cleanup tests.
import { mkdir, writeFile } from "node:fs/promises";
const [root, mode] = process.argv.slice(2);
process.on("SIGTERM", () => {});
await mkdir(root, { mode: 0o700 });
if (mode === "bad-ready") await writeFile(`${root}/ready.json`, '{"schema":"invalid"}\n', { flag: "wx", mode: 0o600 });
process.stdout.write("synthetic_child_ready\n");
setInterval(() => {}, 1000);
