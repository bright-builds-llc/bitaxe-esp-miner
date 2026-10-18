# Ultra 205: local mining soak 001

Local mining soak 001 ran the device on its own, with no Gate lease
(ADR-0038 task 6). After an endpoint handoff it saved the owner's pool as the
primary pool, resumed, and mined for 30 minutes. It then paused, and the ASIC
powered down. Repo-owned commands drove it: `just otawww-endpoint` and
`just local-mining-soak`.

Attempt 001 passed, and an independent review agrees with `complete` after
recomputing the verdict from the journal. Parity is not promoted, and
"unbounded mining" stays a non-claim until the longer soaks.

| Observation      | Result                                                                      |
| ---------------- | --------------------------------------------------------------------------- |
| Firmware commit  | `be69061a42fb834e8d6e33ae23f3601835322e40` (clean; local mining on)         |
| App ELF SHA-256  | `14cd8974311320f7ce2bf18fd5eb7c18ef73e3b73d6f095a776aafd4aded82ca`          |
| Package manifest | `e430a9db64412a353e4c222ecc6a79ccc8acddbf5b3c99eb1ca4ec272d390212`          |
| Gate             | `eacd8147ead746711be6a03341a32e3cfed41a3e` (endpoint handoff only, no lease) |
| Install          | State-preserving `flash-monitor`: `observed_exact_package`                  |
| Pool             | Saved from the owner's input and kept (`pool_config: local-owner-supplied`) |
| Profile          | 485 MHz and 1200 mV on every sample, automatic fan (saved settings)         |
| Baseline         | Paused, mining on boot off, no pause reason, 0.32 W (ASIC off)              |
| Start            | First mining sample 11 s after the resume                                   |
| Active time      | 1,801,120 ms of continuous mining; 804 mining samples of 808               |
| Gaps             | Maximum 2,611 ms between samples (limit 10 s)                               |
| Shares           | 87 accepted, 0 rejected                                                     |
| Mining samples   | 5.24–5.28 V, 10.4–12.4 W, 33–61 C chip, 2,455–3,816 RPM                     |
| Readings         | Power, input, chip temperature and fan fresh on every mining sample         |
| Stops            | None: no reset, device pause, fault or overheat; one boot session           |
| Stop             | ASIC powered down 27 s after the pause (0.44 W), paused, no pause reason    |
| Persistence      | Mining on boot still off; the owner's pool stays saved                      |
| Device           | Same physical device before and after; no serial holder afterwards          |

## Observations, not pass criteria

- **Hashrate:** `hashRate` read 0 on every sample, while shares arrived about
  every 21 s. The hashrate register reads carried only a lease's generation,
  so under local mining they had no permit. A fix follows this soak in its
  own commit: the reads carry the owning generation, and a source guard
  pins it. Hashrate under local mining is not hardware-verified until a
  later soak.
- **VR temperature:** `vrTemp` read 0. The Ultra 205 reports no VR
  temperature, and the judge applies no VR limit.
- **Chip temperature:** it settled at 60–61 C under the automatic fan, well
  below the 75 C limit.
- **Hand-over:** the endpoint handoff ran on the paused device, and its
  result records the endpoint, a closed page and a released host. The soak
  journal does not show when the link went quiet; mining began 11 s after
  the resume.
- **Stopped proof:** the journal records the second of the two powered-down
  samples. The first is enforced by the run and covered by tests, but not
  recorded; later soaks should record both.

## Not shown here

The soak does not show:
- mining past 30 minutes, or across a network or pool outage;
- a Gate lease interrupting local mining and restoring it;
- overheat recovery, the restart guard, the crash-loop guard or the hardware
  fault latch on hardware.

Those are later soaks under their own contracts.
