// The SolidJS client must send exactly the requests the current client sends.
// Both run against the same recording fakes and their records must match.
import assert from "node:assert/strict";
import test from "node:test";

import { createApiClient, type ApiServices } from "../src/api/api-client.js";
import { evaluateCurrentScript, plain } from "./support.js";

type Recorded = { path: string; method: string; headers: unknown; body: unknown; credentials: unknown; signal: boolean };
type FakeReply = { ok: boolean; status: number; json?: unknown; text?: string; throwJson?: boolean };
type Harness = {
  readonly requests: Recorded[];
  readonly timers: number[];
  cleared: number;
  readonly downloads: Array<{ href: string; download: string }>;
  readonly sockets: FakeSocket[];
  reply: FakeReply | { throws: unknown };
};
type ClientUnderTest = {
  getInfo(): Promise<unknown>;
  getScoreboard(): Promise<unknown>;
  getTheme(): Promise<unknown>;
  patchSettings(patch: unknown): Promise<unknown>;
  saveTheme(theme: unknown): Promise<unknown>;
  command(name: string): Promise<unknown>;
  retainedLogs(): Promise<string>;
  downloadLogs(): void;
  uploadFirmware(file: unknown): Promise<unknown>;
  uploadWww(file: unknown): Promise<unknown>;
  openLogStream(onText: (text: string) => void, onState: (state: string) => void): () => void;
};

const FILE = { synthetic: "esp-miner.bin" };

type FakeSocket = {
  readonly url: string;
  readonly listeners: Map<string, (event: { data?: unknown }) => void>;
  closed: boolean;
  addEventListener(type: string, listener: (event: { data?: unknown }) => void): void;
  close(): void;
  emit(type: string, data?: unknown): void;
};

/** A WebSocket stand-in that records every socket it constructs. */
function recordingSocketType(sockets: FakeSocket[]): new (url: string) => FakeSocket {
  return class RecordingSocket implements FakeSocket {
    readonly listeners = new Map<string, (event: { data?: unknown }) => void>();
    closed = false;
    constructor(readonly url: string) { sockets.push(this); }
    addEventListener(type: string, listener: (event: { data?: unknown }) => void): void { this.listeners.set(type, listener); }
    close(): void { this.closed = true; }
    emit(type: string, data?: unknown): void { this.listeners.get(type)?.({ data }); }
  };
}

function createHarness(): Harness & { fetch: (path: string, init: RequestInit) => Promise<unknown>; socketType: new (url: string) => FakeSocket } {
  const sockets: FakeSocket[] = [];
  const harness = {
    requests: [] as Recorded[],
    timers: [] as number[],
    cleared: 0,
    downloads: [] as Array<{ href: string; download: string }>,
    sockets,
    reply: { ok: true, status: 200, json: { ok: true }, text: "log line\n" } as Harness["reply"],
    fetch: async (path: string, init: RequestInit): Promise<unknown> => {
      harness.requests.push({
        path,
        method: init.method ?? "GET",
        headers: plain(init.headers),
        body: (init.body as unknown) === FILE ? "<file>" : init.body,
        credentials: init.credentials,
        signal: init.signal !== undefined,
      });
      if ("throws" in harness.reply) throw harness.reply.throws;
      const reply = harness.reply;
      return {
        ok: reply.ok,
        status: reply.status,
        json: async () => { if (reply.throwJson) throw new SyntaxError("bad json"); return reply.json; },
        text: async () => reply.text ?? "",
      };
    },
    socketType: recordingSocketType(sockets),
  };
  return harness;
}

async function currentClient(harness: ReturnType<typeof createHarness>): Promise<ClientUnderTest> {
  const documentRef = {
    createElement: () => {
      const anchor = { href: "", download: "", rel: "", click: () => harness.downloads.push({ href: anchor.href, download: anchor.download }) };
      return anchor;
    },
  };
  return (await evaluateCurrentScript("api-client.js", "BitaxeApi", {
    fetch: harness.fetch,
    setTimeout: (_callback: () => void, delay: number) => { harness.timers.push(delay); return 1; },
    clearTimeout: () => { harness.cleared += 1; },
    AbortController,
    location: { protocol: "https:", host: "miner.invalid" },
    WebSocket: harness.socketType,
    document: documentRef,
  })) as ClientUnderTest;
}

function solidClient(harness: ReturnType<typeof createHarness>): ClientUnderTest {
  const services: ApiServices = {
    fetch: harness.fetch as ApiServices["fetch"],
    setTimeout: (_callback, delay) => { harness.timers.push(delay); return 1; },
    clearTimeout: () => { harness.cleared += 1; },
    location: { protocol: "https:", host: "miner.invalid" },
    maybeWebSocket: harness.socketType,
    clickDownload: (href, download) => harness.downloads.push({ href, download }),
  };
  return createApiClient(services) as unknown as ClientUnderTest;
}

async function exerciseRequests(client: ClientUnderTest): Promise<unknown[]> {
  return [
    await client.getInfo(),
    await client.getScoreboard(),
    await client.getTheme(),
    await client.patchSettings({ hostname: "miner" }),
    await client.saveTheme({ colorScheme: "dark", accentColors: { primary: "#f7931a" } }),
    await client.command("pause"),
    await client.command("restart"),
    await client.retainedLogs(),
    (await client.uploadFirmware(FILE) as { status: number }).status,
    (await client.uploadWww(FILE) as { status: number }).status,
  ];
}

async function failureCategories(client: ClientUnderTest, harness: Harness): Promise<unknown[]> {
  const outcomes: unknown[] = [];
  const replies: Harness["reply"][] = [
    { ok: false, status: 503 },
    { throws: new TypeError("network") },
    { throws: Object.assign(new Error("aborted"), { name: "AbortError" }) },
    { ok: true, status: 200, throwJson: true },
  ];
  for (const reply of replies) {
    harness.reply = reply;
    const error = await client.getInfo().then(() => null, (failure: unknown) => failure as { category: string; status: number });
    outcomes.push({ category: error?.category, status: error?.status });
  }
  const invalid = await client.command("erase").then(() => null, (failure: unknown) => failure as { category: string });
  outcomes.push(invalid?.category);
  return outcomes;
}

async function runContract(build: (harness: ReturnType<typeof createHarness>) => Promise<ClientUnderTest> | ClientUnderTest) {
  const harness = createHarness();
  const client = await build(harness);
  const results = await exerciseRequests(client);
  client.downloadLogs();
  const states: string[] = [];
  const texts: string[] = [];
  const close = client.openLogStream((text) => texts.push(text), (state) => states.push(state));
  const socket = harness.sockets[0];
  socket?.emit("open");
  socket?.emit("message", "line\n");
  socket?.emit("message", 42);
  socket?.emit("error");
  socket?.emit("close");
  close();
  const failures = await failureCategories(client, harness);
  return plain({
    results,
    requests: harness.requests,
    timers: harness.timers,
    cleared: harness.cleared,
    downloads: harness.downloads,
    socket: { url: socket?.url, closed: socket?.closed, states, texts },
    failures,
  });
}

test("the solid API client sends exactly the current client's requests", async () => {
  // Arrange
  const expected = await runContract(currentClient);

  // Act
  const actual = await runContract(solidClient);

  // Assert
  assert.deepEqual(actual, expected);
  assert.equal(expected.requests.length, 14);
  assert.deepEqual(expected.timers.every((delay) => delay === 15000), true);
  assert.equal(expected.socket.url, "wss://miner.invalid/api/ws");
});
