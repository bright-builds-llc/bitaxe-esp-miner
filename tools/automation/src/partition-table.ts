type PartitionRow = readonly [
  name: string,
  type: string,
  subtype: string,
  offset: string,
  size: string,
  flags: string,
];

const expectedPartitions: readonly PartitionRow[] = [
  ["nvs", "data", "nvs", "0x9000", "0x6000", ""],
  ["phy_init", "data", "phy", "0xf000", "0x1000", ""],
  ["factory", "app", "factory", "0x10000", "4M", ""],
  ["www", "data", "spiffs", "0x410000", "3M", ""],
  ["ota_0", "app", "ota_0", "0x710000", "4M", ""],
  ["ota_1", "app", "ota_1", "0xb10000", "4M", ""],
  ["otadata", "data", "ota", "0xf10000", "8K", ""],
  ["coredump", "data", "coredump", "", "64K", ""],
] as const;

function normalizedSize(value: string): string | undefined {
  const maybeMatch = /^(0x[0-9a-f]+|[0-9]+)([kKmM]?)$/u.exec(value);
  if (maybeMatch === null) return undefined;
  return `${maybeMatch[1]}${maybeMatch[2]?.toUpperCase() ?? ""}`;
}

function partitionRow(line: string): PartitionRow | undefined {
  const fields = line.split(",").map((field) => field.trim());
  if (fields.length === 5) fields.push("");
  if (fields.length !== 6) return undefined;

  const [name, type, subtype, offset, size, flags] = fields;
  if (
    name === undefined
    || type === undefined
    || subtype === undefined
    || offset === undefined
    || size === undefined
    || flags === undefined
  ) return undefined;

  const maybeSize = normalizedSize(size);
  if (maybeSize === undefined) return undefined;
  return [name, type, subtype, offset, maybeSize, flags];
}

export function canonicalPartitionRows(document: string): boolean {
  const rows = document
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter((line) => line !== "" && !line.startsWith("#"))
    .map(partitionRow);

  return rows.length === expectedPartitions.length
    && rows.every((maybeRow, index) => {
      const expected = expectedPartitions[index];
      return maybeRow !== undefined
        && expected !== undefined
        && maybeRow.every((field, fieldIndex) => field === expected[fieldIndex]);
    });
}

export function requiredPartitionCount(): number {
  return expectedPartitions.length;
}
