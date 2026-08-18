import { chmod, writeFile } from "node:fs/promises";

function sameOriginTarget(origin: URL, route: string): URL {
  if (origin.username !== "" || origin.password !== "" || origin.pathname !== "/" || origin.search !== "" || origin.hash !== "") {
    throw new Error("device origin must be origin-only");
  }
  if (!route.startsWith("/") || route.startsWith("//")) throw new Error("API route must be same-origin relative");
  const target = new URL(route, origin);
  if (target.origin !== origin.origin) throw new Error("API target escaped the admitted origin");
  return target;
}

export async function fetchJsonFromSameOrigin(
  origin: URL,
  route: string,
  privateOutput: string,
): Promise<unknown> {
  const target = sameOriginTarget(origin, route);
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

export async function fetchJsonArrayFromSameOrigin(
  origin: URL,
  route: string,
  privateOutput: string,
): Promise<readonly unknown[]> {
  const target = sameOriginTarget(origin, route);
  const response = await fetch(target, {
    method: "GET",
    redirect: "error",
    signal: AbortSignal.timeout(10_000),
    headers: { accept: "application/json" },
  });
  if (!response.ok) throw new Error(`same-origin API returned HTTP ${String(response.status)}`);
  const value: unknown = await response.json();
  if (!Array.isArray(value) || value.length > 20) {
    throw new Error("same-origin API response must be a bounded JSON array");
  }
  await writeFile(privateOutput, `${JSON.stringify(value)}\n`, {
    encoding: "utf8",
    mode: 0o600,
    flag: "wx",
  });
  await chmod(privateOutput, 0o600);
  return value;
}

export async function fetchTextFromSameOrigin(
  origin: URL,
  route: string,
  privateOutput: string,
): Promise<string> {
  return (await fetchTextResponseFromSameOrigin(origin, route, privateOutput)).body;
}

export type SameOriginTextResponse = {
  readonly body: string;
  readonly contentType: string | null;
  readonly contentDisposition: string | null;
};

export async function fetchTextResponseFromSameOrigin(
  origin: URL,
  route: string,
  privateOutput: string,
): Promise<SameOriginTextResponse> {
  const target = sameOriginTarget(origin, route);
  const response = await fetch(target, {
    method: "GET",
    redirect: "error",
    signal: AbortSignal.timeout(10_000),
    headers: { accept: "text/plain" },
  });
  if (!response.ok) throw new Error(`same-origin API returned HTTP ${String(response.status)}`);
  const body = await response.text();
  if (Buffer.byteLength(body, "utf8") > 1024 * 1024) throw new Error("same-origin text response is too large");
  await writeFile(privateOutput, body, { encoding: "utf8", mode: 0o600, flag: "wx" });
  await chmod(privateOutput, 0o600);
  return {
    body,
    contentType: response.headers.get("content-type"),
    contentDisposition: response.headers.get("content-disposition"),
  };
}

export async function sendSameOriginRequest(
  origin: URL,
  route: string,
  method: "PATCH" | "POST",
  privateOutput: string,
  maybeJsonBody?: unknown,
): Promise<void> {
  const target = sameOriginTarget(origin, route);
  const response = await fetch(target, {
    method,
    redirect: "error",
    signal: AbortSignal.timeout(15_000),
    ...(maybeJsonBody === undefined ? {} : {
      headers: { "content-type": "application/json" },
      body: JSON.stringify(maybeJsonBody),
    }),
  });
  const body = await response.text();
  await writeFile(privateOutput, body, { encoding: "utf8", mode: 0o600, flag: "wx" });
  if (!response.ok) throw new Error(`same-origin API returned HTTP ${String(response.status)}`);
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
