/** Every BWG restoration projection profile the closed validator accepts. */
export const PROJECTION_PROFILES: readonly string[];
/** Validate one published BWG restoration projection under its own profile; throws on any deviation. */
export function validateProjection(value: unknown): unknown;
