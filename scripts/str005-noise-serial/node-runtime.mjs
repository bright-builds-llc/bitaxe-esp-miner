/** Preserve only the canonical Node launcher's runfiles configuration, never credentials. */
export function nodeRuntimeEnvironment(environment = process.env) {
  const names = ["JS_BINARY__NODE_BINARY", "JS_BINARY__NODE_PATCHES", "JS_BINARY__NODE_WRAPPER",
    "JS_BINARY__FS_PATCH_ROOTS", "JS_BINARY__PATCH_NODE_FS"];
  return Object.fromEntries(names.filter((name) => environment[name] !== undefined).map((name) => [name, environment[name]]));
}
