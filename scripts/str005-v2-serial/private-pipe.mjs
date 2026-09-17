import { check } from "./values.mjs";

/** One memory-only UTF-8 record, accepted only after EOF proves no trailing record. */
export function readPrivateRecord(stream, { deadlineMs, now = () => performance.now(), maximum = 4096 }) {
  return new Promise((resolve, reject) => {
    const chunks = []; let length = 0, settled = false;
    const timer = setTimeout(() => finish("v2_private_pipe_timeout"), Math.max(0, deadlineMs - now()));
    function finish(code, value) {
      if (settled) return; settled = true; clearTimeout(timer);
      for (const chunk of chunks) chunk.fill(0); chunks.length = 0;
      if (code) reject(Object.assign(new Error(code), { code })); else resolve(value);
    }
    stream.on("data", (chunk) => {
      if (settled) return;
      length += chunk.length;
      if (length > maximum || now() > deadlineMs) return finish("v2_private_pipe_bound");
      chunks.push(Buffer.from(chunk));
    });
    stream.once("error", () => finish("v2_private_pipe_failed"));
    stream.once("end", () => {
      if (settled) return;
      let combined;
      try {
        combined = Buffer.concat(chunks);
        check(now() <= deadlineMs && combined.length > 1 && combined.at(-1) === 10 &&
          combined.indexOf(10) === combined.length - 1, "v2_private_pipe_record");
        const text = new TextDecoder("utf-8", { fatal: true }).decode(combined.subarray(0, -1));
        const value = JSON.parse(text); finish(null, value);
      } catch { finish("v2_private_pipe_record"); }
      finally { combined?.fill(0); }
    });
    stream.once("close", () => { if (!settled) finish("v2_private_pipe_truncated"); });
  });
}

/** No logging or durable staging of fixture input is permitted. */
export function writePrivateRecord(stream, value) {
  const encoded = Buffer.from(`${JSON.stringify(value)}\n`, "utf8");
  check(encoded.length <= 4096, "v2_private_pipe_bound");
  return new Promise((resolve, reject) => {
    let settled = false;
    function finish(error) {
      if (settled) return; settled = true; encoded.fill(0);
      if (error) reject(Object.assign(new Error("v2_private_pipe_write"), { code: "v2_private_pipe_write" })); else resolve();
    }
    stream.once("error", finish);
    stream.end(encoded, finish);
  });
}
