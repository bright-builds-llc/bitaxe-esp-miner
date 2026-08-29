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
    "./worker-controller-v03-conformance/fixtures",
    "./conformance/bwg-worker-controller-0.3/fixtures.json",
  ],
  [
    "./worker-usb-v02-conformance/fixtures",
    "./conformance/bwg-worker-usb-0.2/fixtures.json",
  ],
  [
    "./worker-possession-conformance/fixtures",
    "./conformance/bwg-worker-possession-0.1/fixtures.json",
  ],
  [
    "./worker-deployment-trust-conformance/fixtures",
    "./conformance/bwg-worker-deployment-trust-0.1/fixtures.json",
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
  controller.capabilities?.protocolVersion !== "bwg-worker-controller/0.3" ||
  usb.profile !== "bwg-worker-usb/0.2" ||
  possession.profile !== "bwg-worker-possession/0.1" ||
  trust.profile !== "bwg-worker-deployment-trust/0.1" ||
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
if (
  JSON.stringify(deploymentArtifacts[0]) !== JSON.stringify(deploymentArtifacts[2]) ||
  JSON.stringify(deploymentArtifacts[1]) !== JSON.stringify(deploymentArtifacts[3])
) {
  throw new Error("firmware BWG deployment artifact drift");
}
