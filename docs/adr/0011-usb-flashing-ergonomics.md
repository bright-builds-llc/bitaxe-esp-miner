# Make USB Flashing a First-Class Just Workflow

USB flashing should be available through ergonomic commands such as `just flash board=601`, `just flash board=601 port=/dev/cu.usbmodem...`, `just monitor`, and `just flash-monitor`. These commands should route through Bazel-owned build/package targets, discover likely ESP serial ports when possible, fail clearly when the port is ambiguous, print the underlying flashing command for debugging, and prioritize Gamma 601 behavior while keeping other board configs visible but not falsely verified.
