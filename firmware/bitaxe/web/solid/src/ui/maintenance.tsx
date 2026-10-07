// Update and theme pages.
// Upstream AxeOS breadcrumbs (feature reference only):
// `reference/esp-miner/main/http_server/axe-os/src/app/components/update/` and
// `.../components/design/`.

import { createSignal, type JSX } from "solid-js";

import { publicError, themeFromPayload, themePayload } from "../core/ui-core.js";
import { UPLOAD_RULES, isAdmittedUpload, type UploadKind } from "../core/workflows.js";
import { PageSection, Status, formValues, type Shell } from "./shared.js";

// Element ids and input names match the current variant's markup.
const UPLOAD_IDS: Readonly<Record<UploadKind, Readonly<{ form: string; file: string; button: string; input: string }>>> = {
  firmware: { form: "firmware-update-form", file: "firmware-file", button: "firmware-upload", input: "firmware" },
  www: { form: "www-update-form", file: "www-file", button: "www-upload", input: "www" },
};

function UploadForm(props: { shell: Shell; kind: UploadKind; title: string; label: string; button: string; children: JSX.Element }): JSX.Element {
  const rule = UPLOAD_RULES[props.kind];
  const [admitted, setAdmitted] = createSignal(false);
  let maybeInput: HTMLInputElement | undefined;

  async function upload(form: HTMLFormElement): Promise<void> {
    const maybeFile = maybeInput?.files?.[0];
    if (maybeFile === undefined || !isAdmittedUpload(props.kind, maybeFile.name)) {
      props.shell.status.set(rule.statusName, rule.wrongFile, "error");
      return;
    }
    if (!props.shell.browser.confirm(rule.confirm)) return;
    props.shell.status.set(rule.statusName, rule.progress);
    try {
      await (props.kind === "firmware" ? props.shell.api.uploadFirmware(maybeFile) : props.shell.api.uploadWww(maybeFile));
      props.shell.status.set(rule.statusName, rule.success, "success");
      form.reset();
      setAdmitted(false);
      if (rule.reloadAfterMs !== null) props.shell.browser.reloadAfter(rule.reloadAfterMs);
    } catch (error) {
      props.shell.status.set(rule.statusName, publicError(error), "error");
    }
  }

  const ids = UPLOAD_IDS[props.kind];
  return (
    <form id={ids.form} class="card form-card" onSubmit={(event) => { event.preventDefault(); void upload(event.currentTarget); }}>
      <h2>{props.title}</h2>
      {props.children}
      <label>{props.label}<input
        id={ids.file}
        name={ids.input}
        type="file"
        accept=".bin,application/octet-stream"
        ref={maybeInput}
        onChange={(event) => setAdmitted(isAdmittedUpload(props.kind, event.currentTarget.files?.[0]?.name))}
      /></label>
      <button id={ids.button} class="button danger" type="submit" disabled={!admitted()}>{props.button}</button>
      <Status shell={props.shell} name={rule.statusName} />
    </form>
  );
}

export function UpdatePage(props: { shell: Shell }): JSX.Element {
  return (
    <PageSection shell={props.shell} page="update" eyebrow="Software" title="Update">
      <div class="content-grid">
        <UploadForm shell={props.shell} kind="firmware" title="Firmware image" label="Firmware file" button="Upload firmware">
          <p class="muted">Choose an exact <code>esp-miner.bin</code>. Upload starts only after a separate confirmation.</p>
        </UploadForm>
        <UploadForm shell={props.shell} kind="www" title="AxeOS image" label="AxeOS file" button="Upload AxeOS">
          <p class="muted">Choose an exact <code>www.bin</code>. The web interface is erased and rewritten, so an interrupted upload leaves it unavailable until you upload again from <a href="/recovery">recovery</a>.</p>
        </UploadForm>
      </div>
    </PageSection>
  );
}

export function ThemePage(props: { shell: Shell }): JSX.Element {
  async function save(form: HTMLFormElement): Promise<void> {
    const payload = themePayload(formValues(form));
    props.shell.applyTheme(themeFromPayload(payload));
    props.shell.status.set("theme", "Saving…");
    try {
      await props.shell.api.saveTheme(payload);
      props.shell.status.set("theme", "Theme saved.", "success");
    } catch (error) {
      props.shell.status.set("theme", publicError(error), "error");
    }
  }
  return (
    <PageSection shell={props.shell} page="theme" eyebrow="Appearance" title="Theme">
      <form id="theme-form" class="card form-card narrow" onSubmit={(event) => { event.preventDefault(); void save(event.currentTarget); }}>
        <h2>Console appearance</h2>
        <label>Color scheme<select name="colorScheme" value={props.shell.theme().scheme}><option value="dark">Dark</option><option value="light">Light</option></select></label>
        <label>Accent color<input name="accentColor" type="color" value={props.shell.theme().accent} /></label>
        <button class="button primary" type="submit">Save theme</button>
        <Status shell={props.shell} name="theme" />
      </form>
    </PageSection>
  );
}
