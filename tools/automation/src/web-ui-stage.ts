// Stages one web UI variant into the exact file set packed into `www.bin`
// (ADR-0034). The pure core decides names, gzip siblings and metadata; the
// shell at the bottom reads inputs and writes the staged directory.
import { createHash } from "node:crypto";
import { gunzipSync, gzipSync, constants as zlibConstants } from "node:zlib";

/** Selectable web UI variants; `current` is the default. */
export const webUiVariants = ["current", "solid"] as const;
export type WebUiVariant = (typeof webUiVariants)[number];
export const webUiStageSchema = "bitaxe-web-ui-stage-v1";

/** SPIFFS object names hold 64 bytes including the terminating NUL (`spiffsgen.py --obj-name-len 64`). */
export const spiffsNameLimitBytes = 63;
/** Added by the packager with the build label; never part of a variant. */
const reservedNames = new Set(["version.txt"]);
const gzipOsUnix = 3;

export type StagedFile = { readonly path: string; readonly bytes: Buffer };
export type StagedAsset = { readonly path: string; readonly bytes: number; readonly sha256: string; readonly gzip_of?: string };
export type WebUiStageMetadata = {
  readonly schema_version: typeof webUiStageSchema;
  readonly variant: WebUiVariant;
  readonly files: readonly StagedAsset[];
};

export function maybeWebUiVariant(value: string): WebUiVariant | null {
  return (webUiVariants as readonly string[]).includes(value) ? (value as WebUiVariant) : null;
}

/**
 * Whether a file gets a served `.gz` sibling. `current` keeps exactly the one
 * representation its UI-004, API-008 and FS-001 evidence bound
 * (`/assets/app.css.gz`); `solid` compresses its content-hashed scripts and
 * styles. `index.html` stays raw in both so OTAWWW evidence can digest it.
 */
export function wantsGzipSibling(variant: WebUiVariant, path: string): boolean {
  if (variant === "current") return path === "assets/app.css";
  return /^assets\/[^/]+\.(?:js|css)$/u.test(path);
}

/** Byte-reproducible gzip: maximum compression, no name, zero mtime, fixed OS byte. */
export function deterministicGzip(bytes: Buffer): Buffer {
  const compressed = gzipSync(bytes, { level: zlibConstants.Z_BEST_COMPRESSION });
  compressed.writeUInt32LE(0, 4);
  compressed[9] = gzipOsUnix;
  return compressed;
}

function sha256Hex(bytes: Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}

/** Rejects names the firmware cannot store or serve safely. */
export function assertStageablePath(path: string): void {
  if (!/^[A-Za-z0-9._-]+(?:\/[A-Za-z0-9._-]+)*$/u.test(path) || path.split("/").some((part) => part.startsWith("."))) {
    throw new Error(`web UI asset name is not stageable: ${path}`);
  }
  if (path.endsWith(".gz")) throw new Error(`web UI sources must not contain gzip files: ${path}`);
  if (reservedNames.has(path)) throw new Error(`web UI asset name is reserved: ${path}`);
}

function assertSpiffsName(path: string): void {
  if (Buffer.byteLength(`/${path}`) > spiffsNameLimitBytes) throw new Error(`web UI asset name exceeds the SPIFFS limit: ${path}`);
}

/** Builds the staged file set and its metadata from a variant's source files. */
export function stageWebUi(variant: WebUiVariant, sources: readonly StagedFile[]): { files: StagedFile[]; metadata: WebUiStageMetadata } {
  const sorted = [...sources].sort((left, right) => left.path.localeCompare(right.path));
  const paths = new Set<string>();
  const files: StagedFile[] = [];
  const assets: StagedAsset[] = [];
  for (const source of sorted) {
    assertStageablePath(source.path);
    if (paths.has(source.path)) throw new Error(`duplicate web UI asset: ${source.path}`);
    paths.add(source.path);
    files.push(source);
    assets.push({ path: source.path, bytes: source.bytes.length, sha256: sha256Hex(source.bytes) });
    if (!wantsGzipSibling(variant, source.path)) continue;
    const compressed = deterministicGzip(source.bytes);
    if (!gunzipSync(compressed).equals(source.bytes)) throw new Error(`gzip sibling does not round-trip: ${source.path}`);
    files.push({ path: `${source.path}.gz`, bytes: compressed });
    assets.push({ path: `${source.path}.gz`, bytes: compressed.length, sha256: sha256Hex(compressed), gzip_of: source.path });
  }
  if (!paths.has("index.html")) throw new Error("web UI variant has no index.html");
  for (const file of files) assertSpiffsName(file.path);
  return { files, metadata: { schema_version: webUiStageSchema, variant, files: assets } };
}

/** Every `.gz` file must decompress to exactly its source sibling. */
export function verifyGzipSiblings(files: readonly StagedFile[]): void {
  const byPath = new Map(files.map((file) => [file.path, file.bytes] as const));
  for (const file of files.filter((candidate) => candidate.path.endsWith(".gz"))) {
    const maybeSource = byPath.get(file.path.slice(0, -3));
    if (maybeSource === undefined) throw new Error(`gzip file has no source sibling: ${file.path}`);
    if (!gunzipSync(file.bytes).equals(maybeSource)) throw new Error(`gzip file does not match its source: ${file.path}`);
  }
}
