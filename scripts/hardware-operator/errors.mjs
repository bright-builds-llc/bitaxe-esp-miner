/** A typed refusal or failure whose snake_case code is safe to print and to record in evidence. */
export class HardwareOperatorError extends Error {
  constructor(code) {
    super(code);
    this.name = "HardwareOperatorError";
    this.code = code;
  }
}

export function refuse(condition, code) {
  if (!condition) throw new HardwareOperatorError(code);
}
