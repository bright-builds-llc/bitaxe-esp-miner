# Require Explicit Verification Evidence for Parity

Parity checklist items may move to `verified` only with explicit evidence such as pure unit tests against reference-derived fixtures, golden outputs, API comparisons, board-named hardware smoke logs, repeatable hardware regression checks, or an accepted deferred gap. Safety-critical and hardware-control surfaces such as voltage, fan, thermal, power, and ASIC initialization require hardware evidence before they are verified; unit or golden tests can mark those paths implemented but not verified.
