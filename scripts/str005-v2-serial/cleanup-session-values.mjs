/** Closed parent-only provenance; never a device or supervisor producer failure. */
export const PARENT_CLEANUP_STAGES = Object.freeze(["prepare", "browser_witness", "supervisor_stop", "record"]);
export const PARENT_CLEANUP_CODES = Object.freeze([
  "v2_parent_cleanup_failed", "v2_parent_cleanup_state", "v2_parent_browser_unproved",
  "v2_parent_supervisor_identity", "v2_parent_supervisor_stop_failed", "v2_parent_supervisor_timeout",
  "v2_parent_supervisor_exit_failed", "v2_parent_owner_remains", "v2_cleanup_server_not_live",
  "v2_cleanup_private_context_unproved", "v2_cleanup_instance_changed", "v2_cleanup_source_changed",
  "v2_cleanup_helper_changed", "v2_pool_listener_present", "v2_pool_listener_unproved",
  "v2_listener_inventory_shape", "v2_listener_inventory_bound",
]);
export function parentCleanupCode(error) {
  if (error?.code === "noise_owner_remains") return "v2_parent_owner_remains";
  return PARENT_CLEANUP_CODES.includes(error?.code) ? error.code : "v2_parent_cleanup_failed";
}
