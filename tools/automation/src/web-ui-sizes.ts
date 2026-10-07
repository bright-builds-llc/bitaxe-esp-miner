// Pure size accounting for staged web UI variants: per-file raw and gzip
// bytes, variant totals, SPIFFS page use, budgets and the size history.
import { deterministicGzip, type StagedFile, type WebUiVariant } from "./web-ui-stage.js";

export type AssetSize = { readonly path: string; readonly rawBytes: number; readonly gzipBytes: number; readonly servedBytes: number };
export type VariantSize = {
  readonly variant: WebUiVariant;
  readonly assets: readonly AssetSize[];
  readonly rawBytes: number;
  readonly gzipBytes: number;
  readonly servedBytes: number;
  readonly stagedBytes: number;
  readonly estimatedSpiffsPages: number;
};

/** `spiffsgen.py` geometry used by the packager (`--obj-name-len 64`, defaults otherwise). */
export const spiffsGeometry = {
  pageBytes: 256,
  blockBytes: 4096,
  lookupPagesPerBlock: 1,
  dataPageContentBytes: 251,
  firstIndexEntries: 87,
  laterIndexEntries: 124,
} as const;
const pagesPerBlock = spiffsGeometry.blockBytes / spiffsGeometry.pageBytes;
const usablePagesPerBlock = pagesPerBlock - spiffsGeometry.lookupPagesPerBlock;

/** Pages `spiffsgen.py` writes for one object: index pages plus data pages. */
export function spiffsObjectPages(bytes: number): number {
  const dataPages = Math.ceil(bytes / spiffsGeometry.dataPageContentBytes);
  const extraIndexPages = Math.ceil(Math.max(0, dataPages - spiffsGeometry.firstIndexEntries) / spiffsGeometry.laterIndexEntries);
  return 1 + extraIndexPages + dataPages;
}

function sum(values: readonly number[]): number {
  return values.reduce((total, value) => total + value, 0);
}

/** Sizes of a staged variant; `.gz` siblings count as the served bytes of their source. */
export function variantSize(variant: WebUiVariant, files: readonly StagedFile[]): VariantSize {
  const byPath = new Map(files.map((file) => [file.path, file.bytes] as const));
  const assets = files
    .filter((file) => !file.path.endsWith(".gz"))
    .map((file) => ({
      path: file.path,
      rawBytes: file.bytes.length,
      gzipBytes: deterministicGzip(file.bytes).length,
      servedBytes: byPath.get(`${file.path}.gz`)?.length ?? file.bytes.length,
    }));
  return {
    variant,
    assets,
    rawBytes: sum(assets.map((asset) => asset.rawBytes)),
    gzipBytes: sum(assets.map((asset) => asset.gzipBytes)),
    servedBytes: sum(assets.map((asset) => asset.servedBytes)),
    stagedBytes: sum(files.map((file) => file.bytes.length)),
    estimatedSpiffsPages: sum(files.map((file) => spiffsObjectPages(file.bytes.length))),
  };
}

export type SpiffsUsage = {
  readonly imageBytes: number;
  readonly blocks: number;
  readonly usedPages: number;
  readonly usablePages: number;
  readonly usedBytes: number;
  readonly usableBytes: number;
};

/** Counts allocated pages from each block's object lookup page (0xFFFF free, 0x0000 deleted). */
export function spiffsUsage(image: Buffer): SpiffsUsage {
  if (image.length === 0 || image.length % spiffsGeometry.blockBytes !== 0) throw new Error("SPIFFS image size is not a whole number of blocks");
  const blocks = image.length / spiffsGeometry.blockBytes;
  let usedPages = 0;
  for (let block = 0; block < blocks; block += 1) {
    const lookup = block * spiffsGeometry.blockBytes;
    for (let entry = 0; entry < usablePagesPerBlock; entry += 1) {
      const objectId = image.readUInt16LE(lookup + entry * 2);
      if (objectId !== 0xffff && objectId !== 0) usedPages += 1;
    }
  }
  const usablePages = blocks * usablePagesPerBlock;
  return {
    imageBytes: image.length,
    blocks,
    usedPages,
    usablePages,
    usedBytes: usedPages * spiffsGeometry.pageBytes,
    usableBytes: usablePages * spiffsGeometry.pageBytes,
  };
}

export type WebUiBudget = { readonly schema_version: "bitaxe-web-ui-budget-v1"; readonly max_gzip_bytes: Readonly<Record<WebUiVariant, number>> };

/** Human-readable violations of the per-variant gzip budget; empty when every variant fits. */
export function budgetViolations(sizes: readonly VariantSize[], budget: WebUiBudget): string[] {
  return sizes
    .filter((size) => size.gzipBytes > budget.max_gzip_bytes[size.variant])
    .map((size) => `${size.variant} gzip total ${String(size.gzipBytes)} exceeds its budget ${String(budget.max_gzip_bytes[size.variant])}`);
}

function row(cells: readonly (string | number)[]): string {
  const [variant, file, ...numbers] = cells;
  return `${String(variant).padEnd(8)} ${String(file).padEnd(34)} ${numbers.map((value) => String(value).padStart(9)).join(" ")}`;
}

/** Plain-text report: per-file sizes, per-variant totals and measured SPIFFS use. */
export function formatSizeReport(sizes: readonly VariantSize[], usages: ReadonlyMap<WebUiVariant, SpiffsUsage>): string {
  const lines = [row(["variant", "file", "raw", "gzip", "served"])];
  for (const size of sizes) {
    for (const asset of size.assets) lines.push(row([size.variant, asset.path, asset.rawBytes, asset.gzipBytes, asset.servedBytes]));
    lines.push(row([size.variant, `total (${String(size.assets.length)} files)`, size.rawBytes, size.gzipBytes, size.servedBytes]));
    const maybeUsage = usages.get(size.variant);
    if (maybeUsage === undefined) continue;
    const percent = ((100 * maybeUsage.usedBytes) / maybeUsage.usableBytes).toFixed(2);
    lines.push(`${size.variant.padEnd(8)} www.bin SPIFFS: ${String(maybeUsage.usedPages)} of ${String(maybeUsage.usablePages)} pages, ${String(maybeUsage.usedBytes)} of ${String(maybeUsage.usableBytes)} usable bytes (${percent}%) in a ${String(maybeUsage.imageBytes)}-byte image`);
  }
  return `${lines.join("\n")}\n`;
}

export const sizeHistoryHeader = "date_utc,source_commit,source_dirty,variant,files,raw_bytes,gzip_bytes,served_bytes,spiffs_used_bytes";

/** CSV history rows for one measurement. */
export function sizeHistoryRows(dateUtc: string, commit: string, dirty: boolean, sizes: readonly VariantSize[], usages: ReadonlyMap<WebUiVariant, SpiffsUsage>): string[] {
  return sizes.map((size) => [
    dateUtc,
    commit,
    String(dirty),
    size.variant,
    size.assets.length,
    size.rawBytes,
    size.gzipBytes,
    size.servedBytes,
    usages.get(size.variant)?.usedBytes ?? "",
  ].join(","));
}
