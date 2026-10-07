// Network, pool and settings pages: write-only PATCH forms.
// Upstream AxeOS breadcrumbs (feature reference only):
// `reference/esp-miner/main/http_server/axe-os/src/app/components/network-edit/`,
// `.../components/pool/` and `.../components/edit/`.

import { createEffect, type JSX } from "solid-js";

import { buildSettingsPatch, patchFieldNames, publicError, type PatchKind } from "../core/ui-core.js";
import { Bind, PageSection, Status, formValues, type Shell } from "./shared.js";

async function submitSettings(shell: Shell, kind: PatchKind, form: HTMLFormElement): Promise<void> {
  const patch = buildSettingsPatch(kind, formValues(form));
  if (Object.keys(patch).length === 0) {
    shell.status.set(kind, "Enter at least one setting to change.", "error");
    return;
  }
  shell.status.set(kind, "Saving…");
  try {
    await shell.api.patchSettings(patch);
    shell.status.set(kind, "Settings saved. Restart may be required.", "success");
    for (const field of patchFieldNames(kind)) {
      const maybeInput = form.elements.namedItem(field);
      if (field.toLowerCase().includes("pass") && maybeInput instanceof HTMLInputElement) maybeInput.value = "";
    }
    void shell.refreshInfo();
  } catch (error) {
    shell.status.set(kind, publicError(error), "error");
  }
}

function SettingsForm(props: { shell: Shell; kind: PatchKind; class: string; ref?: (form: HTMLFormElement) => void; children: JSX.Element }): JSX.Element {
  const onSubmit = (event: SubmitEvent) => {
    event.preventDefault();
    void submitSettings(props.shell, props.kind, event.currentTarget as HTMLFormElement);
  };
  return (
    <form id={`${props.kind}-form`} class={props.class} ref={props.ref} onSubmit={onSubmit}>
      {props.children}
      <Status shell={props.shell} name={props.kind} />
    </form>
  );
}

export function NetworkPage(props: { shell: Shell }): JSX.Element {
  let maybeForm: HTMLFormElement | undefined;
  // Prefill the visible identity once, without ever loading the stored password.
  createEffect(() => {
    const maybeInfo = props.shell.info();
    if (maybeForm === undefined || maybeInfo === null) return;
    for (const field of ["hostname", "ssid"]) {
      const maybeInput = maybeForm.elements.namedItem(field);
      const value = maybeInfo[field];
      if (maybeInput instanceof HTMLInputElement && maybeInput.value === "" && typeof value === "string") maybeInput.value = value;
    }
  });
  return (
    <PageSection shell={props.shell} page="network" eyebrow="Connectivity" title="Network">
      <div class="content-grid">
        <SettingsForm shell={props.shell} kind="network" class="card form-card" ref={(form) => { maybeForm = form; }}>
          <h2>Wi-Fi configuration</h2>
          <p class="muted">Only fields you enter are changed. The stored password is never loaded into this page.</p>
          <label>Hostname<input name="hostname" maxlength="32" autocomplete="off" placeholder="bitaxe" /></label>
          <label>Wi-Fi name<input name="ssid" maxlength="32" autocomplete="off" placeholder="Network name" /></label>
          <label>Wi-Fi password<input name="wifiPass" type="password" maxlength="63" autocomplete="new-password" placeholder="Leave blank to keep stored password" /></label>
          <button class="button primary" type="submit">Save network settings</button>
        </SettingsForm>
        <article class="card">
          <h2>Connection status</h2>
          <dl class="details">
            <div><dt>Status</dt><dd><Bind shell={props.shell} field="wifiStatus" /></dd></div>
            <div><dt>Signal</dt><dd><Bind shell={props.shell} field="wifiRSSI" /></dd></div>
            <div><dt>IPv4</dt><dd><Bind shell={props.shell} field="ipv4" /></dd></div>
            <div><dt>IPv6</dt><dd><Bind shell={props.shell} field="ipv6" /></dd></div>
          </dl>
        </article>
      </div>
    </PageSection>
  );
}

export function PoolPage(props: { shell: Shell }): JSX.Element {
  return (
    <PageSection shell={props.shell} page="pool" eyebrow="Mining endpoint" title="Pool">
      <SettingsForm shell={props.shell} kind="pool" class="card form-card narrow">
        <h2>Primary Stratum configuration</h2>
        <p class="muted">Pool details are write-only here and are not saved by the browser.</p>
        <label>Protocol<select name="stratumProtocol"><option value="SV1">Stratum V1</option></select></label>
        <label>Pool host<input name="stratumURL" maxlength="3999" autocomplete="off" placeholder="pool.example" /></label>
        <label>Pool port<input name="stratumPort" type="number" min="1" max="65535" inputmode="numeric" placeholder="3333" /></label>
        <label>Worker<input name="stratumUser" maxlength="3999" autocomplete="off" placeholder="account.worker" /></label>
        <label>Pool password<input name="stratumPassword" type="password" maxlength="3999" autocomplete="new-password" placeholder="Leave blank to keep stored password" /></label>
        <button class="button primary" type="submit">Save pool settings</button>
      </SettingsForm>
    </PageSection>
  );
}

export function SettingsPage(props: { shell: Shell }): JSX.Element {
  return (
    <PageSection shell={props.shell} page="settings" eyebrow="Preferences" title="Settings">
      <div class="content-grid">
        <SettingsForm shell={props.shell} kind="settings" class="card form-card">
          <h2>Telemetry history</h2>
          <label>Sample frequency (seconds)<input name="statsFrequency" type="number" min="0" max="65535" inputmode="numeric" placeholder="30" /></label>
          <button class="button primary" type="submit">Save settings</button>
        </SettingsForm>
        <article class="card callout">
          <h2>Hardware controls are protected</h2>
          <p>Frequency, core voltage, fan, and power controls are intentionally absent until their live safety evidence is complete.</p>
        </article>
      </div>
    </PageSection>
  );
}
