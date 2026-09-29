// This owner records ready state before Stop. Restoration is mandatory in the later closed receipt.
// The Stop-first share recovery collector has a different, intentionally stricter policy.
export const BEFORE_READ_BASELINE_POLICY = Object.freeze({ allowConfirmedBaseline: false });
