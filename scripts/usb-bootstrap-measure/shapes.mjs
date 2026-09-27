import { check, object, uint } from "./values.mjs";
export const hex = (value, length = 64) => check(typeof value === "string" && new RegExp(`^[a-f0-9]{${length}}$`, "u").test(value), "bootstrap_digest");
const strings = value => check(Array.isArray(value) && value.every(item => typeof item === "string"), "bootstrap_array");
const records = (value, keys) => { check(Array.isArray(value), "bootstrap_array"); for (const item of value) object(item, keys); };
export function sourceShape(value) {
  records(value, ["path", "sha256", "length"]); let previous = "";
  for (const file of value) { check(typeof file.path === "string" && /^[A-Za-z0-9_./-]+$/u.test(file.path) && !file.path.startsWith("/") &&
    file.path.split("/").every(part => part !== "." && part !== ".." && part.length > 0) && file.path > previous, "bootstrap_source_inventory");
    hex(file.sha256); uint(file.length); previous = file.path; }
}
export function packageShape(value) {
  object(value, ["manifest_sha256", "app_elf_sha256", "reference_commit", "artifacts", "update_segments", "firmware_commit", "manifest"]);
  for (const key of ["manifest_sha256", "app_elf_sha256"]) hex(value[key]); for (const key of ["reference_commit", "firmware_commit"]) hex(value[key], 40);
  records(value.artifacts, ["kind", "sha256", "length"]);
  check(value.artifacts.length === 8 && new Set(value.artifacts.map(row => row.kind)).size === 8 &&
    ["firmware_elf", "firmware_ota_image", "www_spiffs_image", "factory_merged_image", "partition_table", "otadata_initial", "bootloader", "partition_table_binary"].every(kind => value.artifacts.some(row => row.kind === kind)), "bootstrap_package_shape");
  for (const row of value.artifacts) { hex(row.sha256); uint(row.length); }
  check(value.artifacts.find(row => row.kind === "firmware_elf").sha256 === value.app_elf_sha256, "bootstrap_package_shape");
  const geometry = [["bootloader", 0, 0x8000], ["partition_table_binary", 0x8000, 0x1000], ["firmware_ota_image", 0x10000, 0x400000], ["www_spiffs_image", 0x410000, 0x300000], ["otadata_initial", 0xf10000, 0x2000]];
  records(value.update_segments, ["artifact_kind", "offset", "length"]); check(value.update_segments.length === 5, "bootstrap_package_shape");
  for (const [i, [kind, offset, capacity]] of geometry.entries()) { const segment = value.update_segments[i]; uint(segment.length);
    check(segment.artifact_kind === kind && segment.offset === offset && segment.length > 0 && Math.ceil(segment.length / 4096) * 4096 <= capacity &&
      value.artifacts.find(row => row.kind === kind).length === segment.length, "bootstrap_package_shape"); }
}
function paths(value, noise = false) {
  const keys = ["symbol", "ownership", "stackBytes", "addedStackBytes", "selectedPathBytes", "requiredMarginBytes", "remainingStackBytes", "selectedFunctions", "nodes", "completeCallgraphBound"];
  if (noise) keys.push("entryBytes", "entryBudgetBytes", "callbackBinding", "threadEntryBinding", "supplementalRanges", "unresolvedJumps");
  object(value, keys); uint(value.selectedFunctions); records(value.nodes, ["symbol", "entryBytes"]);
  for (const key of ["stackBytes", "addedStackBytes", "selectedPathBytes", "requiredMarginBytes", "remainingStackBytes"]) uint(value[key]);
  check(typeof value.symbol === "string" && typeof value.ownership === "string" && value.completeCallgraphBound === false, "bootstrap_native_shape");
  check(value.selectedPathBytes + value.requiredMarginBytes <= value.stackBytes && value.remainingStackBytes === value.stackBytes - value.selectedPathBytes, "bootstrap_native_budget");
  for (const node of value.nodes) { check(typeof node.symbol === "string", "bootstrap_native_shape"); uint(node.entryBytes); }
  if (noise) { uint(value.entryBytes); uint(value.entryBudgetBytes); ranges(value.supplementalRanges); jumps(value.unresolvedJumps);
    check(typeof value.callbackBinding === "string" && typeof value.threadEntryBinding === "string", "bootstrap_native_shape"); }
}
function ranges(value) { uint(value); }
function address(value) { check(Number.isSafeInteger(value) && value >= 0 || typeof value === "string" && /^(?:0x)?[a-fA-F0-9]{1,16}$/u.test(value), "bootstrap_native_address"); }
function jumps(value) { records(value, ["symbol", "address"]); for (const row of value) { check(typeof row.symbol === "string", "bootstrap_native_shape"); address(row.address); } }
export function nativeShape(value, context) {
  object(value, ["schema", "result", "firmwareCommit", "elfSha256", "packageManifestSha256", "appImageSha256", "appImageBytes", "imageSlotBytes", "sdkconfigSha256", "sourceFiles", "auditorSources", "objdumpSha256", "noise", "production", "telemetry", "v2", "hardwareQualified", "startupHeapQualified", "completeCallgraphBound"]);
  check(value.schema === "str005-v2-native-readiness-v1" && value.result === "selected_native_checks_passed" && value.firmwareCommit === context.package.firmware_commit &&
    value.elfSha256 === context.package.app_elf_sha256 && value.packageManifestSha256 === context.package.manifest_sha256 && value.hardwareQualified === false && value.startupHeapQualified === false && value.completeCallgraphBound === false, "bootstrap_native_shape");
  for (const key of ["elfSha256", "packageManifestSha256", "appImageSha256", "sdkconfigSha256", "objdumpSha256"]) hex(value[key]);
  uint(value.appImageBytes); uint(value.imageSlotBytes); check(value.appImageBytes <= value.imageSlotBytes, "bootstrap_native_shape");
  for (const name of ["sourceFiles", "auditorSources"]) { records(value[name], ["path", "sha256"]); check(value[name].length > 0 && new Set(value[name].map(row => row.path)).size === value[name].length, "bootstrap_native_shape");
    for (const row of value[name]) check(context.sourceInventory.some(file => file.path === row.path && file.sha256 === row.sha256), "bootstrap_native_source"); }
  paths(value.noise, true); object(value.production, ["schema", "stack_bytes", "owner_entry_bytes", "owner_entry_budget_bytes", "required_measured_free_bytes"]);
  check(value.production.schema === "production-owner-stack-audit-v1" && value.production.owner_entry_bytes <= value.production.owner_entry_budget_bytes, "bootstrap_native_shape");
  for (const [key, v] of Object.entries(value.production)) if (key === "schema") check(typeof v === "string", "bootstrap_native_shape"); else uint(v);
  object(value.telemetry, ["schema", "result", "main_stack_budget_bytes", "targeted_path_bytes", "matched_paths", "complete_callgraph_bound", "hardware_safety_verified", "nodes", "edges"]);
  check(value.telemetry.schema === "telemetry-stack-audit-v1" && value.telemetry.result === "targeted_path_fits" && value.telemetry.targeted_path_bytes <= value.telemetry.main_stack_budget_bytes, "bootstrap_native_shape");
  records(value.telemetry.nodes, ["symbol", "address", "entry_bytes"]); records(value.telemetry.edges, ["call_address", "target"]);
  for (const row of value.telemetry.nodes) { check(typeof row.symbol === "string", "bootstrap_native_shape"); address(row.address); uint(row.entry_bytes); }
  for (const row of value.telemetry.edges) { address(row.call_address); address(row.target); }
  uint(value.telemetry.matched_paths); uint(value.telemetry.main_stack_budget_bytes); uint(value.telemetry.targeted_path_bytes);
  check(value.telemetry.complete_callgraph_bound === false && value.telemetry.hardware_safety_verified === false, "bootstrap_native_shape");
  object(value.v2, ["channel", "share", "supplementalRanges", "unresolvedJumps"]); paths(value.v2.channel); paths(value.v2.share); ranges(value.v2.supplementalRanges); jumps(value.v2.unresolvedJumps);
}
