# STR-005 TCP connection identity result

- Result: `accepted`
- Diagnostic source: `e0398abb74d710d6a3918f226f1c08fd3203d35f`
- Reference source: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware: detector-admitted Ultra 205
- Projection: `docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-009.json`
- Redaction: `passed`

## Conclusion

Diagnostic-009 proves one exact-peer TCP connection whose private firmware
local port matched the fixture remote port. The standard Rust socket adapter
reported 64 bytes with no pre-send, post-send, or post-shutdown socket error;
the correlated fixture candidate received exact `0x00..0x3f`, observed no
extra byte, and returned the fixed receipt. No competing connection or
candidate overflow occurred.

The inline restore completed snapshot and Wi-Fi writes but its final USB
sampler reported identity drift. Fresh recovery-003 subsequently proved exact
recovery-006 identity/settings, `mineonboot=false`, inactive zero-work/share
state, final device admission, and cleanup. The recovery-aware finalizer joined
that proof to the diagnostic and published the independently validated v2
projection without another hardware effect.

Diagnostic-010 is not eligible. This closes only TCP payload delivery. Noise,
authentication, V2 messages, channel/job/share handling, mining, hardware
control, soak, other boards, and STR-005 parity promotion remain non-claims.
