export function requiresPhysicalReacquisition(scenario) {
  return ["disconnect", "reboot", "monotonic_uncertainty"].includes(scenario);
}

export function physicalInstruction(scenario) {
  if (scenario === "disconnect") {
    return "Remove only USB while barrel power remains connected. Reconnect USB, then choose Reacquire.";
  }
  if (scenario === "reboot" || scenario === "monotonic_uncertainty") {
    return "Remove USB and barrel power. Restore barrel power first, then USB, then choose Reacquire.";
  }
  return "No physical transition is required.";
}
