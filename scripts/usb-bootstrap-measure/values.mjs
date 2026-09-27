export { check, object, sha256, uint, digest } from "../str005-v2-serial/values.mjs";
export const PREFIX = "usb-bootstrap-measure";
export const TASK = "task-usb-bootstrap-drain-observability";
export const CONTRACT = { path: "docs/hardware/usb-bootstrap-drain-measurement.md", sha256: "f8534c50f47d7c3a210295b5523c46cc51679850281a222af5c97402133e9cc4" };
export const BEFORE = { firmware_commit: "14f6d6d98ad939db065542db66dd8aad77f097d7", app_elf_sha256: "cbe496ec96b5f20cfaeabf10f6e41105de916310e4dc34fc3a77f07d1091e8cf" };
export const PREDECESSOR = { contextSha256: "dcfce9bd9bdaf8ea8fb59d021b0449fec10b58a651a353aa24ce360e4a4ff79a", resultSha256: "1b687eda11fe233677f77c4a77ee462912a6ddb5caf57ad9bfc7567fd4782c2e", sealSha256: "3b77bd5d8b83433ba18b758a3ec579806a91c866e594cb0254656d7a313ea342" };
export const code = error => typeof error?.code === "string" && /^bootstrap_[a-z_]+$/u.test(error.code) ? error.code : "bootstrap_operation_failed";
export const schema = name => `${PREFIX}-${name}-v1`;
