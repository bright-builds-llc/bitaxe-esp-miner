import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import test from "node:test";

import { sendInterruptedFirmwareUpload } from "./interrupted-upload.js";

test("real child server receives one strict prefix beneath the declared body length", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-interrupted-upload-"));
  const observationPath = path.join(root, "observation.json");
  const script = `
    const net = require("node:net");
    const fs = require("node:fs");
    const output = process.argv[1];
    const server = net.createServer((socket) => {
      const chunks = [];
      socket.on("data", (chunk) => chunks.push(chunk));
      socket.on("end", () => {
        const received = Buffer.concat(chunks);
        const split = received.indexOf(Buffer.from("\\r\\n\\r\\n"));
        const head = received.subarray(0, split).toString("ascii");
        const body = received.subarray(split + 4);
        const match = /Content-Length: (\\d+)/i.exec(head);
        fs.writeFileSync(output, JSON.stringify({ declared: Number(match[1]), body: body.length }));
        server.close();
      });
    });
    server.listen(0, "127.0.0.1", () => process.stdout.write(String(server.address().port) + "\\n"));
  `;
  const child = spawn(process.execPath, ["-e", script, observationPath], {
    stdio: ["ignore", "pipe", "pipe"],
  });
  const port = await firstLine(child.stdout);

  try {
    // Act
    const result = await sendInterruptedFirmwareUpload(
      new URL(`http://127.0.0.1:${port}`),
      Buffer.alloc(16_384, 0x5a),
      4_096,
    );
    const childExit = await new Promise<number | null>((resolve) => child.once("close", resolve));
    const observed = JSON.parse(await readFile(observationPath, "utf8")) as {
      declared: number;
      body: number;
    };

    // Assert
    assert.equal(childExit, 0);
    assert.deepEqual(result, {
      declared_body_bytes: 16_384,
      transmitted_body_bytes: 4_096,
      connection_closed: true,
    });
    assert.deepEqual(observed, { declared: 16_384, body: 4_096 });
  } finally {
    child.kill("SIGKILL");
    await rm(root, { recursive: true, force: true });
  }
});

async function firstLine(stream: NodeJS.ReadableStream): Promise<number> {
  return await new Promise<number>((resolve, reject) => {
    let text = "";
    stream.on("data", (chunk: Buffer) => {
      text += chunk.toString("utf8");
      const newline = text.indexOf("\n");
      if (newline < 0) return;
      const port = Number(text.slice(0, newline));
      if (!Number.isSafeInteger(port)) reject(new Error("child server port is invalid"));
      else resolve(port);
    });
    stream.once("error", reject);
  });
}
