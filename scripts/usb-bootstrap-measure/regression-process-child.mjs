// Fixed gate: no test program runs until its durable process-group owner is observed.
import { spawn } from "node:child_process";
process.on("SIGTERM", () => {});
process.once("disconnect", () => { process.kill(-process.pid, "SIGKILL"); });
process.once("message", ({ program, args, cwd, env }) => {
  const child = spawn(program, args, { cwd, env, shell: false, stdio: ["ignore", "inherit", "inherit"] });
  let delivered = false;
  const finish = value => {
    if (delivered) return; delivered = true;
    if (process.connected) process.send(value);
  };
  child.once("error", () => finish({ status: null, signal: null, failed: true }));
  child.once("exit", (status, signal) => finish({ status, signal, failed: false }));
});
