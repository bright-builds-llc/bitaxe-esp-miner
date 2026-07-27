(function installFirmwareProvenance(global) {
  "use strict";

  const COMMIT_PATTERN = /^[0-9a-f]{12,40}$/i;
  const TIMESTAMP_PATTERN =
    /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/;

  function validatedProvenance(payload) {
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
      commit: `${payload.sourceCommit.slice(0, 12)}${
        payload.sourceDirty ? " (dirty)" : ""
      }`,
      built: payload.buildTimestampUtc,
    };
  }

  async function hydrate(options) {
    const documentRef = options?.document ?? global.document;
    const fetchRef = options?.fetch ?? global.fetch;
    if (!documentRef || typeof fetchRef !== "function") {
      return false;
    }

    try {
      const response = await fetchRef("/api/system/info", {
        headers: { Accept: "application/json" },
      });
      if (!response.ok) {
        return false;
      }
      const provenance = validatedProvenance(await response.json());
      if (!provenance) {
        return false;
      }

      documentRef.getElementById("provenance-version").textContent =
        provenance.version;
      documentRef.getElementById("provenance-commit").textContent =
        provenance.commit;
      documentRef.getElementById("provenance-built").textContent =
        provenance.built;
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
