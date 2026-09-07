import { QualificationError, requireCondition } from "./contract.mjs";

export async function body(request) {
  const chunks = [];
  let size = 0;
  for await (const chunk of request) {
    size += chunk.length;
    requireCondition(size <= 65536, "request_body_bound");
    chunks.push(chunk);
  }
  try { return JSON.parse(Buffer.concat(chunks).toString("utf8")); }
  catch { throw new QualificationError("request_json"); }
}
export function send(response, status, value, contentType = "application/json") {
  response.writeHead(status, { "Content-Type": contentType, "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff",
    "Referrer-Policy": "no-referrer", "Cross-Origin-Resource-Policy": "same-origin" });
  response.end(Buffer.isBuffer(value) ? value : JSON.stringify(value));
}
