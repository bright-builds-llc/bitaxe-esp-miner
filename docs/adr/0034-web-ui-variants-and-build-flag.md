# ADR-0034: Selectable web UI variants and a size budget

Accepted 2026-10-07 under `task-web-ui-variants-and-size-budget`, by the
owner's approval of that date. Updates and supersedes
[ADR-0010](0010-axeos-api-and-asset-compatibility.md) for the web UI.

## Context

ADR-0010 kept the firmware API and asset packaging compatible with AxeOS and
left the web UI out of scope. Since then the project has shipped its own
handwritten operator UI in `firmware/bitaxe/static/www`: plain HTML, CSS and
JavaScript with no build step, about 50 KB raw. UI-004, API-008, FS-001 and
OTA-002 evidence binds that UI.

The owner wants a second implementation in SolidJS, the Bright Builds default
for new frontends, so the two can be compared and debugged side by side. The
3 MiB `www` partition and OTAWWW uploads make bundle size a first-class
concern.

## Decision

1. **Two variants, one per build.** `current` is the handwritten UI, kept where
   it is and still the default. `solid` is a SolidJS port in
   `firmware/bitaxe/web/solid`. Its first goal is a one-to-one match with
   `current`: the same eight pages, routes, API requests, write-only password
   handling, confirmations and accessibility affordances. It shares the
   `current` stylesheet.
2. **Build setting.** The Bazel `string_flag` `//firmware/bitaxe:web_ui`
   (`current|solid`, default `current`) selects which staged variant
   `//firmware/bitaxe:firmware_image` packs into `www.bin`. `just build`,
   `just package`, `just flash` and `just flash-monitor` accept
   `--web-ui <variant>`.
3. **Hermetic JavaScript build.** The Solid variant is locked by its own
   `pnpm-lock.yaml`. `npm_translate_lock` (hub `npm_web_ui`) imports it, and
   Vite with `vite-plugin-solid` runs as a Bazel action. All versions are
   exact. No dependency runs an install script.
4. **Staging.** Each variant is staged into an exact `www` tree plus
   `web-ui.json`, which records the variant and a SHA-256 digest per file.
   Staging generates byte-reproducible `.gz` siblings (zero mtime, no name,
   fixed OS byte) and verifies that each one decompresses to its source.
   Committed `.gz` sources are refused.
   - `current` keeps exactly its evidence-bound representation: a gzip copy of
     `assets/app.css` only.
   - `solid` compresses its content-hashed scripts and styles.
   - `index.html` and `version.txt` stay raw in both variants.
5. **Manifest binding.** The package manifest records `web_ui_variant` and
   `web_ui_assets`, the digest of every file packed into `www.bin`. The flash
   tool requires a recorded variant and refuses a mismatch with `--web-ui`, so
   hardware evidence binds the exact variant.
6. **Cache safety.** Solid script and style names carry Vite content hashes,
   within the 63-byte SPIFFS name limit. A 30-day cached older bundle therefore
   cannot run after an OTAWWW swap. `index.html` stays unhashed, and the
   operator routes (`/`, `/network`, ...) serve it without a cache header.
7. **Size accounting.**
   - `just web-ui-sizes` reports raw, gzip and served bytes per file, totals per
     variant, and SPIFFS pages measured from the generated image.
   - `--append-history` appends rows to `docs/web-ui/size-history.csv`.
   - `//tools/automation:web_ui_budget_test` enforces the hard per-variant gzip
     budget in `firmware/bitaxe/web-ui-budget.json` and the partition fit.
8. **No upstream UI code.** Upstream Angular AxeOS
   (`reference/esp-miner/main/http_server/axe-os`) is a feature reference only.
   It is never built, bundled or copied. Both variants follow it through
   independent implementations, with breadcrumbs.

## Consequences

- The default build is unchanged except for one fix. The committed
  `app.css.gz` was stale (it lacked the scoreboard rules), and the server always
  prefers a `.gz` sibling, so devices served old CSS. It is now generated at
  package time from `app.css`. The bytes served at `/assets/app.css.gz` change,
  so re-projecting UI-004 or API-008 evidence against a new package needs a new
  package identity. Archived evidence is unchanged.
- `MODULE.bazel` gains `bazel_skylib`, `bazel_lib` and the `npm_web_ui` hub,
  and `MODULE.bazel.lock` changes with them.
- Distributing a `solid` package also distributes `solid-js` (MIT). Its notice
  must ship with any published solid-variant release.
- No Bazel CI workflow exists yet, so the budget and the size report run
  locally only, through `just test` and `just web-ui-sizes`.

## Non-claims

This ADR verifies nothing on hardware. Installing a non-default variant needs
its own task contract. Promoting any UI parity row for `solid` needs its own
evidence.
