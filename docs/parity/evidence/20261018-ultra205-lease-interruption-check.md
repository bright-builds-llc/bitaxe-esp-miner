# Ultra 205: lease interruption check

This check shows on hardware that a Gate Work Lease interrupts local mining
and that local mining comes back afterwards (ADR-0038 task 6, handover).
Two repo-owned commands ran it together:
- `just local-mining-soak --soak lease-001` for the local phases;
- `just ultra205-soak --interrupt-local-mining` for the lease.

Attempt 004 passed both halves. An independent review recomputed both
verdicts from the private journals and agrees with `complete`. Attempts 001
to 003 failed, as recorded in `TASKS.md`:
- 001 failed on a judge rule in our tooling (fixed);
- 002 met an intermittent native panic, still undiagnosed (below);
- 003 was cut short by an operator process-group signal (fixed).
Parity is not promoted.

| Observation      | Result                                                                      |
| ---------------- | --------------------------------------------------------------------------- |
| Firmware commit  | `a69b27037371cec87c7fc328411ec22db316562e` (clean package)                  |
| App ELF SHA-256  | `2455b491ddd1b9de1ae162ba3b67930706232dbc1812ad16cb0ceb60314aeb02`          |
| Gate             | `eacd8147ead746711be6a03341a32e3cfed41a3e`                                  |
| Install          | State-preserving `flash-monitor`: `observed_exact_package`                  |
| Pool             | The owner's saved pool, and the same pool through the grant                 |
| Local mining     | 301,160 ms accumulated before the lease                                     |
| Refused Connect  | One, at stage `hello`; local mining stopped 40 s after the 5 minutes        |
| Admitted Connect | After the ASIC read 0.33 W; local mining stayed off through the 71 s proof  |
| Lease            | 600,000 ms, ended by its budget; safe-stop complete at `fan_paused`         |
| Lease windows    | 20 of 20 credited on HTTP and WebSocket; gaps at most 2,393 ms and 923 ms   |
| Lease shares     | 39 accepted, 0 rejected, counted from the handover's counter restart        |
| After the lease  | The run intent on both transports; pool settings retained; ledger ordinal 8 |
| Restored         | Local mining back 20 s after the Gate soak's result (bound 180 s)           |
| Local mining     | 300,947 ms accumulated after the restore, excluding the outage              |
| Local shares     | 25 accepted, 0 rejected over the local phases                               |
| Outages          | One, 47 s, 265 s into the after phase, recovered on its own (bound 60 s)    |
| Limits           | At most 12.49 W and 61 C; input 5,238–5,473 mV; readings fresh              |
| Stop             | Paused; powered down 21 s and 23 s after the pause (0.54 W, 0.49 W)         |
| Persistence      | Mining on boot off, no pause reason, one boot session                       |
| Device           | Same physical device before and after; `lsof` found no serial holder after  |

## How the handover looked

1. After 5 minutes of local mining, a Gate Connect was refused. The
   firmware ignored the Hello and stopped local mining through its ordered
   safe-stop.
2. Once the ASIC was down, a second Connect was admitted. The idle proof
   then observed 71 s with no mining, and the page signed one 600 s soak
   lease.
3. The lease mined the owner's pool through its grant. The share counters
   restarted with its first mining sample, which the window judge allows
   once.
4. The lease ended by its budget and safe-stopped. The page closed the
   link, and 20 s later local mining resumed with the saved pool.
5. After 5 more minutes, the run paused the device and confirmed the ASIC
   powered down.

## The open panic

In attempt 002 the device panicked about 22 s after the admitted Connect,
before any page operation, and came back paused. It was a native panic, with
no Rust panic receipt. The stored core dump predated that firmware, because
the partition keeps an older dump. It was archived privately and cleared.
Attempts 003 and 004 did not reproduce the panic, including a 90 s idle hold
after admission. The cleared partition will keep the next dump. Until a dump
names the cause, the panic stays an open defect.

## The after-phase outage

Mining stopped and restarted once, 265 s into the after phase:
- HTTP returned no samples for 22.8 s;
- then 11 `safe_blocked` samples followed, with the ASIC down (0.8–1.3 W)
  and the chip cooling from 52 to 44 C;
- the share counters and hashrate restarted from zero;
- mining came back 47 s after the last mining sample.

The boot session and uptime continued, so there was no reset. The pattern
matches soak 002's outages: the device unreachable for about 20 s, then a
new pool session. It points to a network or pool interruption followed by
local mining's own recovery. The cause is not established.

## Not shown here

The check does not show:
- the cause of the attempt 002 panic, or that it cannot recur;
- the cause of the after-phase outage;
- a lease that starts while local mining runs without an operator's
  refused Connect first;
- longer local mining after the lease, or across longer outages;
- overheat recovery, the restart guard, the crash-loop guard or the
  hardware fault latch on hardware.
