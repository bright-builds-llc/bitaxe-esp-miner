import { connect } from "node:net";
import { readFile } from "node:fs/promises";
const input = JSON.parse(await readFile(process.argv[3], "utf8"));
const socket = connect(process.argv[2]);
socket.once("error", () => process.exit(1));
socket.once("connect", () => socket.write(JSON.stringify(input) + "\n", () => {
  // Real client disappears without waiting for any server reply.
  socket.destroy(); process.exit(0);
}));
