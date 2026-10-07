// Dashboard: live summary, device provenance and confirmed operator commands.
// Upstream AxeOS breadcrumb (feature reference only):
// `reference/esp-miner/main/http_server/axe-os/src/app/components/home/`.

import { For, type JSX } from "solid-js";

import { publicError } from "../core/ui-core.js";
import { commandPrompt, maybeValidatedProvenance, type CommandName } from "../core/workflows.js";
import { Bind, PageSection, Status, type Shell } from "./shared.js";

const HERO_STATS = [["Hashrate", "hashRate"], ["Temperature", "temp"], ["Power", "power"]] as const;
const METRICS = [
  ["Shares accepted", "sharesAccepted"],
  ["Shares rejected", "sharesRejected"],
  ["Fan", "fanrpm"],
  ["Uptime", "uptimeSeconds"],
] as const;
const DEVICE_FIELDS = [["Hostname", "hostname"], ["Board", "boardVersion"], ["ASIC", "ASICModel"]] as const;
const COMMAND_BUTTONS: readonly (readonly [CommandName, string, string])[] = [
  ["pause", "secondary", "Pause mining"],
  ["resume", "secondary", "Resume mining"],
  ["restart", "danger", "Restart device"],
];

async function runCommand(shell: Shell, name: CommandName): Promise<void> {
  if (!shell.browser.confirm(commandPrompt(name))) return;
  shell.status.set("commands", "Sending command…");
  try {
    await shell.api.command(name);
    shell.status.set("commands", `Command accepted: ${name}.`, "success");
    if (name !== "restart") void shell.refreshInfo();
  } catch (error) {
    shell.status.set("commands", publicError(error), "error");
  }
}

function DeviceCard(props: { shell: Shell }): JSX.Element {
  const provenance = () => maybeValidatedProvenance(props.shell.info());
  return (
    <article class="card">
      <h2>Device</h2>
      <dl class="details">
        <For each={DEVICE_FIELDS}>
          {([label, field]) => <div><dt>{label}</dt><dd><Bind shell={props.shell} field={field} /></dd></div>}
        </For>
        <div><dt>Firmware</dt><dd id="provenance-version">{provenance()?.version ?? "Unavailable"}</dd></div>
        <div>
          <dt>Source</dt>
          <dd>
            <a id="provenance-commit" href={provenance()?.commitUrl} target="_blank" rel="noopener noreferrer">
              {provenance()?.commitLabel ?? "Unavailable"}
            </a>
          </dd>
        </div>
        <div><dt>Built</dt><dd id="provenance-built">{provenance()?.built ?? "Unavailable"}</dd></div>
      </dl>
    </article>
  );
}

export function Dashboard(props: { shell: Shell }): JSX.Element {
  const refresh = <button class="button secondary" type="button" onClick={() => void props.shell.refreshInfo()}>Refresh</button>;
  return (
    <PageSection shell={props.shell} page="dashboard" eyebrow="Overview" title="Dashboard" action={refresh}>
      <div class="hero card">
        <div><p class="eyebrow">Current state</p><h2><Bind shell={props.shell} field="miningActivity" /></h2></div>
        <For each={HERO_STATS}>
          {([label, field]) => <div class="hero-stat"><span>{label}</span><strong><Bind shell={props.shell} field={field} /></strong></div>}
        </For>
      </div>
      <div class="metric-grid">
        <For each={METRICS}>
          {([label, field]) => <article class="card metric"><span>{label}</span><strong><Bind shell={props.shell} field={field} /></strong></article>}
        </For>
      </div>
      <div class="content-grid">
        <DeviceCard shell={props.shell} />
        <article class="card">
          <h2>Operator controls</h2>
          <p class="muted">Commands require confirmation and report the device response without exposing configuration values.</p>
          <div class="button-row">
            <For each={COMMAND_BUTTONS}>
              {([name, tone, label]) => (
                <button class={`button ${tone}`} type="button" onClick={() => void runCommand(props.shell, name)}>{label}</button>
              )}
            </For>
          </div>
          <Status shell={props.shell} name="commands" />
        </article>
      </div>
    </PageSection>
  );
}
