import { connect, createServer } from "node:net";
import { chmod, lstat, realpath } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { protectedPath } from "../str005-noise-serial/files.mjs";
import { check } from "./values.mjs";
export const MESSAGE_LIMIT = 16384;
export async function socketPath(path, existing = true) {
  check(Buffer.byteLength(path) <= 103 && resolve(path) === path, "v2_operator_protocol");
  const parent = dirname(path); check(await realpath(parent) === parent, "v2_operator_protocol");
  await protectedPath(parent, true);
  if (existing) { const stat = await lstat(path); check(stat.isSocket() && !stat.isSymbolicLink() && stat.uid === process.getuid() && (stat.mode & 0o777) === 0o600, "v2_operator_protocol"); }
}
function decoder(onValue, onError) {
  let bytes = Buffer.alloc(0), received = false;
  return chunk => {
    if (received) return onError();
    bytes = Buffer.concat([bytes, chunk]);
    if (bytes.length > MESSAGE_LIMIT) return onError();
    const index = bytes.indexOf(10); if (index < 0) return;
    received = true;
    if (index !== bytes.length - 1) return onError();
    try { onValue(JSON.parse(bytes.subarray(0, index).toString("utf8"))); } catch { onError(); }
  };
}
export async function serveOperatorSocket(path, handle, operations = {}) {
  await socketPath(path, false);
  const server = createServer(socket => {
    socket.setTimeout(5000, () => socket.destroy());
    socket.on("error", () => socket.destroy());
    socket.on("data", decoder(value => {
      Promise.resolve(handle(value)).then(result => {
        const bytes = Buffer.from(JSON.stringify(result) + "\n");
        if (bytes.length > MESSAGE_LIMIT) { socket.destroy(); return; }
        socket.end(bytes);
      }, () => socket.destroy());
    }, () => socket.destroy()));
  });
  try {
    await new Promise((done, reject) => { server.once("error", reject); server.listen(path, done); });
    await (operations.chmod ?? chmod)(path, 0o600);
    return server;
  } catch (error) {
    await new Promise(done => server.close(closeError => {
      if (closeError && closeError.code !== "ERR_SERVER_NOT_RUNNING") error.cause ??= closeError;
      done();
    }));
    throw error;
  }
}
export async function exchange(path, value) {
  await socketPath(path);
  const bytes = Buffer.from(JSON.stringify(value) + "\n"); check(bytes.length <= MESSAGE_LIMIT, "v2_operator_protocol");
  return new Promise((done, reject) => {
    const socket = connect(path), fail = () => { socket.destroy(); reject(Object.assign(Error("v2_operator_protocol"), { code: "v2_operator_protocol" })); };
    socket.setTimeout(5000, fail); socket.once("error", fail);
    socket.once("connect", () => socket.write(bytes));
    socket.on("data", decoder(result => { socket.destroy(); done(result); }, fail));
    socket.once("end", fail);
  });
}
