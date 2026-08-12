import dgram from "node:dgram";
import { isIP } from "node:net";

import { internalCommandSpec } from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";

const candidatePattern = /^Bitaxe_[0-9A-F]{4}$/u;
const dnsTransactionId = 0x4e02;
const dnsTtlSeconds = 300;
const captiveBody = "Redirect to the captive portal";

export type HostWifiAdmission = {
  readonly interfaceName: string;
};

export type ProvisioningClientObservation = {
  readonly candidateCount: 1;
  readonly associationObserved: true;
  readonly dhcpObserved: true;
  readonly dnsQueryCount: 1;
  readonly wildcardDnsAnswerMatchesGateway: true;
  readonly dnsTtlSeconds: 300;
  readonly captiveRedirectObserved: true;
  readonly captiveRedirectRoot: true;
  readonly captiveRedirectBodyMatches: true;
  readonly systemInfo: Readonly<Record<string, unknown>>;
};

export type ProvisioningClientBoundary =
  | "configuration_candidate"
  | "association"
  | "dhcp"
  | "wildcard_dns"
  | "captive_redirect"
  | "system_info";

export class ProvisioningClientError extends Error {
  public constructor(public readonly boundary: ProvisioningClientBoundary) {
    super("configuration-network client boundary failed");
    this.name = "ProvisioningClientError";
  }
}

type DnsQuery = (clientIpv4: string, gatewayIpv4: string) => Promise<{
  readonly answerMatchesGateway: boolean;
  readonly ttlSeconds: number;
}>;
type Fetch = typeof globalThis.fetch;

export class MacOsProvisioningClient {
  private joined = false;

  public constructor(
    private readonly processPort: ProcessPort,
    private readonly platform: NodeJS.Platform = process.platform,
    private readonly dnsQuery: DnsQuery = queryWildcardDns,
    private readonly fetch: Fetch = globalThis.fetch,
  ) {}

  public async admit(): Promise<HostWifiAdmission> {
    if (this.platform !== "darwin") throw new Error("host platform is unsupported");
    const inventory = await this.run("networksetup", ["-listallhardwareports"]);
    const interfaces = wifiInterfaces(inventory);
    if (interfaces.length !== 1) throw new Error("host Wi-Fi interface count is ineligible");
    const interfaceName = interfaces[0];
    if (interfaceName === undefined) throw new Error("host Wi-Fi interface is unavailable");
    const power = await this.run("networksetup", ["-getairportpower", interfaceName]);
    if (!/:\s*On\s*$/u.test(power)) throw new Error("host Wi-Fi power state is ineligible");
    if (await this.associatedNetwork(interfaceName) !== undefined) {
      throw new Error("host Wi-Fi is already associated");
    }
    if ((await this.candidates()).length !== 0) {
      throw new Error("configuration-network baseline is ambiguous");
    }
    return { interfaceName };
  }

  public async observe(admission: HostWifiAdmission): Promise<ProvisioningClientObservation> {
    const candidates = await atBoundary(
      "configuration_candidate",
      () => this.waitForSingleCandidate(),
    );
    const candidate = candidates[0];
    if (candidate === undefined) throw new ProvisioningClientError("configuration_candidate");
    await atBoundary("association", async () => {
      await this.run("networksetup", ["-setairportnetwork", admission.interfaceName, candidate]);
      this.joined = true;
      if (await this.associatedNetwork(admission.interfaceName) !== candidate) {
        throw new Error("configuration network association failed");
      }
    });
    const lease = await atBoundary("dhcp", () => this.waitForLease(admission.interfaceName));
    await atBoundary("wildcard_dns", async () => {
      const dns = await this.dnsQuery(lease.clientIpv4, lease.gatewayIpv4);
      if (!dns.answerMatchesGateway || dns.ttlSeconds !== dnsTtlSeconds) {
        throw new Error("configuration-network DNS response is invalid");
      }
    });
    const origin = `http://${lease.gatewayIpv4}`;
    await atBoundary("captive_redirect", async () => {
      const redirect = await this.fetch(`${origin}/net002-captive-check`, {
        redirect: "manual",
        signal: AbortSignal.timeout(10_000),
      });
      const redirectBody = await redirect.text();
      const redirectObserved = redirect.status === 302;
      const redirectRoot = redirect.headers.get("location") === "/";
      const redirectBodyMatches = redirectBody === captiveBody;
      if (!redirectObserved || !redirectRoot || !redirectBodyMatches) {
        throw new Error("captive redirect contract is invalid");
      }
    });
    const systemInfo = await atBoundary("system_info", async () => {
      const response = await this.fetch(`${origin}/api/system/info`, {
        redirect: "error",
        signal: AbortSignal.timeout(10_000),
        headers: { accept: "application/json" },
      });
      if (!response.ok) throw new Error("configuration-network API is unavailable");
      const value: unknown = await response.json();
      if (typeof value !== "object" || value === null || Array.isArray(value)) {
        throw new Error("configuration-network API response is invalid");
      }
      return value as Readonly<Record<string, unknown>>;
    });
    return {
      candidateCount: 1,
      associationObserved: true,
      dhcpObserved: true,
      dnsQueryCount: 1,
      wildcardDnsAnswerMatchesGateway: true,
      dnsTtlSeconds: 300,
      captiveRedirectObserved: true,
      captiveRedirectRoot: true,
      captiveRedirectBodyMatches: true,
      systemInfo,
    };
  }

  public async cleanup(admission: HostWifiAdmission): Promise<boolean> {
    if (!this.joined) return true;
    try {
      await this.run("networksetup", ["-setairportpower", admission.interfaceName, "off"]);
      await this.run("networksetup", ["-setairportpower", admission.interfaceName, "on"]);
      await delay(1_000);
      const power = await this.run("networksetup", ["-getairportpower", admission.interfaceName]);
      const associated = await this.associatedNetwork(admission.interfaceName);
      this.joined = false;
      return /:\s*On\s*$/u.test(power) && associated === undefined;
    } catch {
      return false;
    }
  }

  private async waitForSingleCandidate(): Promise<readonly string[]> {
    for (let attempt = 0; attempt < 6; attempt += 1) {
      const candidates = await this.candidates();
      if (candidates.length === 1) return candidates;
      if (candidates.length > 1) throw new Error("configuration network is ambiguous");
      await delay(1_000);
    }
    throw new Error("configuration network was not observed");
  }

  private async candidates(): Promise<readonly string[]> {
    const profile = await this.run("system_profiler", ["SPAirPortDataType", "-json"]);
    let parsed: unknown;
    try {
      parsed = JSON.parse(profile);
    } catch {
      throw new Error("host Wi-Fi inventory is invalid");
    }
    return configurationCandidates(parsed);
  }

  private async associatedNetwork(interfaceName: string): Promise<string | undefined> {
    const outcome = await this.processPort.run(internalCommandSpec(
      "networksetup",
      ["-getairportnetwork", interfaceName],
      (value) => value,
    ));
    if (outcome.timedOut) throw new Error("host Wi-Fi state query timed out");
    if (outcome.exitCode !== 0 || /not associated/iu.test(outcome.stdout)) return undefined;
    const match = /^Current Wi-Fi Network:\s*(.+)\s*$/mu.exec(outcome.stdout);
    return match?.[1];
  }

  private async waitForLease(interfaceName: string): Promise<{
    readonly clientIpv4: string;
    readonly gatewayIpv4: string;
  }> {
    for (let attempt = 0; attempt < 10; attempt += 1) {
      const client = await this.optionalRun("ipconfig", ["getifaddr", interfaceName]);
      const gateway = await this.optionalRun("ipconfig", ["getoption", interfaceName, "router"]);
      const clientIpv4 = client.trim();
      const gatewayIpv4 = gateway.trim();
      if (isIP(clientIpv4) === 4 && isIP(gatewayIpv4) === 4 && clientIpv4 !== gatewayIpv4) {
        return { clientIpv4, gatewayIpv4 };
      }
      await delay(500);
    }
    throw new Error("configuration-network DHCP lease was not observed");
  }

  private async optionalRun(program: string, args: readonly string[]): Promise<string> {
    const outcome = await this.processPort.run(internalCommandSpec(program, [...args], (value) => value));
    if (outcome.timedOut) throw new Error("host network query timed out");
    return outcome.exitCode === 0 ? outcome.stdout : "";
  }

  private async run(program: string, args: readonly string[]): Promise<string> {
    const outcome = await this.processPort.run(internalCommandSpec(program, [...args], (value) => value));
    if (outcome.timedOut) throw new Error("host network command timed out");
    if (outcome.exitCode !== 0) throw new Error("host network command failed");
    return outcome.stdout;
  }
}

async function atBoundary<T>(
  boundary: ProvisioningClientBoundary,
  operation: () => Promise<T>,
): Promise<T> {
  try {
    return await operation();
  } catch {
    throw new ProvisioningClientError(boundary);
  }
}

export function wifiInterfaces(inventory: string): readonly string[] {
  const interfaces = [...inventory.matchAll(
    /Hardware Port:\s*(?:Wi-Fi|AirPort)\s*\nDevice:\s*([^\s]+)\s*$/gmu,
  )].flatMap((match) => match[1] === undefined ? [] : [match[1]]);
  return [...new Set(interfaces)];
}

export function configurationCandidates(value: unknown): readonly string[] {
  const candidates = new Set<string>();
  const visit = (candidate: unknown): void => {
    if (typeof candidate === "string") {
      if (candidatePattern.test(candidate)) candidates.add(candidate);
      return;
    }
    if (Array.isArray(candidate)) {
      for (const entry of candidate) visit(entry);
      return;
    }
    if (typeof candidate !== "object" || candidate === null) return;
    for (const [key, entry] of Object.entries(candidate)) {
      if (candidatePattern.test(key)) candidates.add(key);
      visit(entry);
    }
  };
  visit(value);
  return [...candidates].sort();
}

async function queryWildcardDns(clientIpv4: string, gatewayIpv4: string): Promise<{
  readonly answerMatchesGateway: boolean;
  readonly ttlSeconds: number;
}> {
  const request = dnsRequest();
  const response = await new Promise<Buffer>((resolve, reject) => {
    const socket = dgram.createSocket("udp4");
    const timeout = setTimeout(() => {
      socket.close();
      reject(new Error("configuration-network DNS query timed out"));
    }, 3_000);
    socket.once("error", (error) => {
      clearTimeout(timeout);
      socket.close();
      reject(error);
    });
    socket.once("message", (message) => {
      clearTimeout(timeout);
      socket.close();
      resolve(message);
    });
    socket.bind(0, clientIpv4, () => socket.send(request, 53, gatewayIpv4));
  });
  return parseDnsResponse(response, gatewayIpv4);
}

function dnsRequest(): Buffer {
  const name = Buffer.from([6, ...Buffer.from("net002"), 7, ...Buffer.from("invalid"), 0]);
  const request = Buffer.alloc(12 + name.length + 4);
  request.writeUInt16BE(dnsTransactionId, 0);
  request.writeUInt16BE(0x0100, 2);
  request.writeUInt16BE(1, 4);
  name.copy(request, 12);
  request.writeUInt16BE(1, 12 + name.length);
  request.writeUInt16BE(1, 14 + name.length);
  return request;
}

export function parseDnsResponse(response: Buffer, gatewayIpv4: string): {
  readonly answerMatchesGateway: boolean;
  readonly ttlSeconds: number;
} {
  if (response.length < 12
    || response.readUInt16BE(0) !== dnsTransactionId
    || (response.readUInt16BE(2) & 0x8000) === 0
    || response.readUInt16BE(4) !== 1
    || response.readUInt16BE(6) !== 1) {
    throw new Error("configuration-network DNS header is invalid");
  }
  let offset = skipDnsName(response, 12);
  if (offset + 4 > response.length) throw new Error("configuration-network DNS question is truncated");
  offset += 4;
  offset = skipDnsName(response, offset);
  if (offset + 14 > response.length
    || response.readUInt16BE(offset) !== 1
    || response.readUInt16BE(offset + 2) !== 1
    || response.readUInt16BE(offset + 8) !== 4) {
    throw new Error("configuration-network DNS answer is invalid");
  }
  const ttlSeconds = response.readUInt32BE(offset + 4);
  const answer = [...response.subarray(offset + 10, offset + 14)].join(".");
  return { answerMatchesGateway: answer === gatewayIpv4, ttlSeconds };
}

function skipDnsName(message: Buffer, start: number): number {
  let offset = start;
  while (offset < message.length) {
    const length = message[offset];
    if (length === undefined) break;
    if (length & 0xc0) {
      if (offset + 2 > message.length) break;
      return offset + 2;
    }
    offset += 1;
    if (length === 0) return offset;
    offset += length;
  }
  throw new Error("configuration-network DNS name is invalid");
}

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
