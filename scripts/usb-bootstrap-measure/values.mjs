export { check, object, sha256, uint, digest } from "../str005-v2-serial/values.mjs";
export const PREFIX = "usb-bootstrap-measure";
export const TASK = "task-usb-bootstrap-drain-observability";
export const CONTRACT = { path: "docs/hardware/usb-bootstrap-drain-measurement.md", sha256: "f8534c50f47d7c3a210295b5523c46cc51679850281a222af5c97402133e9cc4" };
export const BEFORE = { firmware_commit: "14f6d6d98ad939db065542db66dd8aad77f097d7", app_elf_sha256: "cbe496ec96b5f20cfaeabf10f6e41105de916310e4dc34fc3a77f07d1091e8cf" };
export const PREDECESSOR = { contextSha256: "dcfce9bd9bdaf8ea8fb59d021b0449fec10b58a651a353aa24ce360e4a4ff79a", resultSha256: "1b687eda11fe233677f77c4a77ee462912a6ddb5caf57ad9bfc7567fd4782c2e", sealSha256: "3b77bd5d8b83433ba18b758a3ec579806a91c866e594cb0254656d7a313ea342" };
export const code = error => typeof error?.code === "string" && /^bootstrap_[a-z_]+$/u.test(error.code) ? error.code : "bootstrap_operation_failed";
export const schema = name => `${PREFIX}-${name}-v1`;

export const CONTEXT_V1 = "usb-bootstrap-measure-context-v1";
export const CONTEXT_V2 = "usb-bootstrap-measure-context-v2";
export const PREFLIGHT_AMENDMENT = { path: "docs/hardware/usb-bootstrap-preflight-successor.md", sha256: "8b596d6d2d2ce83f7815c0208c8e58755f7d3c687169168b8f46fde8049233df" };
export const CONTEXT_V3 = "usb-bootstrap-measure-context-v3";
export const CORRECTION_CONTRACT = { path: "docs/hardware/usb-bootstrap-reader-correction.md", sha256: "beae6e0d7028d3c4ac31b9522dc7bfa456940b1bf1908163302e55c4cb085368" };
export const BEFORE_V3 = { firmware_commit: "3951a441606115798ea47eba0de1052d6590b3a0", app_elf_sha256: "0b9d275b5c75c58b3e25f8221d0be7dbb909313c4d9083aba2faf480af7b283f" };
export const MEASUREMENT_002 = { contextSha256: "80cf7437feb6df2efa6130a7c8f463e4b679506b96e911d439c57d7c0c864ee0", resultSha256: "6b3329af0d9188b7618377b18317646b4d24a2b13bcc13725fde2059f932470d", sealSha256: "a8ad2ef03819f3f413edb85c2fb85b8623c556fe0077ff01e7574f0265cb6386" };
export const hostTimingFile = context => `bootstrap-host-timing-v${context.schema === CONTEXT_V3 ? 2 : 1}.json`;
