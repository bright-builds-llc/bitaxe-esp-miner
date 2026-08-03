import { writeFile } from "node:fs/promises";

export async function fetchJsonFromSameOrigin(
  origin: URL,
  route: string,
  privateOutput: string,
): Promise<unknown> {
  if (origin.username !== "" || origin.password !== "" || origin.pathname !== "/" || origin.search !== "" || origin.hash !== "") {
    throw new Error("device origin must be origin-only");
  }
  if (!route.startsWith("/") || route.startsWith("//")) throw new Error("API route must be same-origin relative");
  const target = new URL(route, origin);
  if (target.origin !== origin.origin) throw new Error("API target escaped the admitted origin");
  const response = await fetch(target, {
    method: "GET",
    redirect: "error",
    signal: AbortSignal.timeout(10_000),
    headers: { accept: "application/json" },
  });
  if (!response.ok) throw new Error(`same-origin API returned HTTP ${String(response.status)}`);
  const value: unknown = await response.json();
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("same-origin API response must be a JSON object");
  }
  await writeFile(privateOutput, `${JSON.stringify(value)}\n`, { encoding: "utf8", mode: 0o600, flag: "wx" });
  return value;
}

export function uniqueRuntimeOrigin(document: string): URL {
  const values = new Set<string>();
  for (const line of document.split(/\r?\n/u)) {
    if (!line.includes("runtime_origin ")) continue;
    const match = /\bdevice_url=(https?:\/\/[^\s]+)\s+redacted=true\b/u.exec(line);
    if (match?.[1] !== undefined) values.add(match[1]);
  }
  if (values.size !== 1) throw new Error("monitor capture must contain exactly one runtime origin");
  const [value] = values;
  if (value === undefined) throw new Error("runtime origin is missing");
  const origin = new URL(value);
  if (origin.protocol !== "http:" && origin.protocol !== "https:") throw new Error("runtime origin protocol is invalid");
  if (origin.username !== "" || origin.password !== "" || origin.pathname !== "/" || origin.search !== "" || origin.hash !== "") {
    throw new Error("runtime origin must be origin-only");
  }
  return origin;
}
