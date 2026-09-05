import { readFile } from "node:fs/promises";

const [
  packagePath,
  controllerPath,
  usbPath,
  possessionPath,
  trustPath,
  publishedTrustPath,
  publishedCapabilityPath,
  firmwareTrustPath,
  firmwareCapabilityPath,
] = process.argv.slice(2);
if (
  !packagePath || !controllerPath || !usbPath || !possessionPath || !trustPath ||
  !publishedTrustPath || !publishedCapabilityPath || !firmwareTrustPath ||
  !firmwareCapabilityPath
) {
  throw new Error("expected package, four fixtures, and four deployment artifacts");
}

const packageDocument = JSON.parse(await readFile(packagePath, "utf8"));
const expectedExports = new Map([
  [
    "./worker-controller-conformance/fixtures",
    "./conformance/bwg-worker-controller-0.4/fixtures.json",
  ],
  [
    "./worker-serial-conformance/fixtures",
    "./conformance/bwg-worker-serial-0.1/fixtures.json",
  ],
  [
    "./worker-possession-conformance/fixtures",
    "./conformance/bwg-worker-possession-0.2/fixtures.json",
  ],
  [
    "./worker-deployment-trust-conformance/fixtures",
    "./conformance/bwg-worker-deployment-trust-0.2/fixtures.json",
  ],
]);
for (const [subpath, expected] of expectedExports) {
  if (packageDocument.exports?.[subpath]?.default !== expected) {
    throw new Error(`BWG package export mismatch: ${subpath}`);
  }
}

const fixtures = await Promise.all(
  [controllerPath, usbPath, possessionPath, trustPath].map(async (fixturePath) =>
    JSON.parse(await readFile(fixturePath, "utf8"))
  ),
);
const [controller, usb, possession, trust] = fixtures;
if (
  controller.capabilities?.protocolVersion !== "bwg-worker-controller/0.4" ||
  usb.profile !== "bwg-worker-serial/0.1" ||
  possession.profile !== "bwg-worker-possession/0.2" ||
  trust.profile !== "bwg-worker-deployment-trust/0.2" ||
  trust.ultra205?.signedCapability?.board?.revision !== "205"
) {
  throw new Error("BWG fixture profile mismatch");
}

const deploymentArtifacts = await Promise.all(
  [
    publishedTrustPath,
    publishedCapabilityPath,
    firmwareTrustPath,
    firmwareCapabilityPath,
  ].map(async (artifactPath) => JSON.parse(await readFile(artifactPath, "utf8"))),
);
const firmwareTrust = deploymentArtifacts[2];
const firmwareCapability = deploymentArtifacts[3];
if (firmwareTrust.profile !== "bwg-worker-deployment-trust/0.2" ||
    firmwareTrust.workLeaseAuthority?.audience !== "bwg-worker-controller/0.4") {
  throw new Error("firmware deployment profile drift");
}
for (const field of ["protocolVersion", "board", "firmware", "compatibility", "transportProfile"]) {
  if (JSON.stringify(deploymentArtifacts[1][field]) !== JSON.stringify(firmwareCapability[field])) {
    throw new Error(`firmware capability contract drift: ${field}`);
  }
}
// Runtime deployment keys are deliberately separate from disposable conformance keys.
// firmware_config_tests verifies the actual installed Update signature/manifest;
// these checks ensure both roles remain represented without exposing private material.
for (const role of ["updateAuthority", "workLeaseAuthority"]) {
  const keys = firmwareTrust[role]?.keys;
  if (!Array.isArray(keys) || keys.length === 0 || keys.some((key) => "d" in key)) {
    throw new Error("firmware public deployment trust invalid");
  }
}
