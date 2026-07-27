(function installFirmwareProvenance(global) {
  "use strict";

  const COMMIT_PATTERN = /^[0-9a-f]{12,40}$/i;
  const COMMIT_URL =
    "https://github.com/bright-builds-llc/bitaxe-esp-miner/commit/";
  const TIMESTAMP_PATTERN =
    /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/;

  function maybeValidatedProvenance(payload) {
    if (
      !payload ||
      typeof payload.semanticVersion !== "string" ||
      payload.semanticVersion.trim() === "" ||
      typeof payload.sourceCommit !== "string" ||
      !COMMIT_PATTERN.test(payload.sourceCommit) ||
      typeof payload.buildTimestampUtc !== "string" ||
      !TIMESTAMP_PATTERN.test(payload.buildTimestampUtc) ||
      typeof payload.sourceDirty !== "boolean"
    ) {
      return null;
    }

    return {
      version: payload.semanticVersion,
      commitLabel: `${payload.sourceCommit.slice(0, 12)}${
        payload.sourceDirty ? " (dirty)" : ""
      }`,
      commitUrl: `${COMMIT_URL}${payload.sourceCommit}`,
      built: payload.buildTimestampUtc,
    };
  }

  function reset(documentRef) {
    documentRef.getElementById("provenance-version").textContent =
      "Unavailable";
    const commit = documentRef.getElementById("provenance-commit");
    commit.textContent = "Unavailable";
    commit.removeAttribute("href");
    documentRef.getElementById("provenance-built").textContent = "Unavailable";
  }

  async function hydrate(options) {
    const documentRef = options?.document ?? global.document;
    const fetchRef = options?.fetch ?? global.fetch;
    if (!documentRef || typeof fetchRef !== "function") {
      return false;
    }

    try {
      reset(documentRef);
      const response = await fetchRef("/api/system/info", {
        headers: { Accept: "application/json" },
      });
      if (!response.ok) {
        return false;
      }
      const maybeProvenance = maybeValidatedProvenance(await response.json());
      if (!maybeProvenance) {
        return false;
      }

      documentRef.getElementById("provenance-version").textContent =
        maybeProvenance.version;
      const commit = documentRef.getElementById("provenance-commit");
      commit.textContent = maybeProvenance.commitLabel;
      commit.setAttribute("href", maybeProvenance.commitUrl);
      documentRef.getElementById("provenance-built").textContent =
        maybeProvenance.built;
      return true;
    } catch {
      return false;
    }
  }

  global.BitaxeProvenance = { hydrate };
  if (global.document) {
    void hydrate();
  }
})(globalThis);
