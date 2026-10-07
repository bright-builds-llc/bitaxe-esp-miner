// Shared view pieces and the shell state handed to every page.

import { createSignal, type Accessor, type JSX } from "solid-js";

import type { ApiClient } from "../api/api-client.js";
import { formatMetric, type Page, type Theme } from "../core/ui-core.js";

export type StatusKind = "" | "success" | "error";
type StatusEntry = Readonly<{ message: string; kind: StatusKind }>;

/** Per-form status lines, keyed by the current variant's `data-status` names. */
export function createStatusBoard() {
  const [entries, setEntries] = createSignal<Readonly<Record<string, StatusEntry>>>({});
  return Object.freeze({
    entry: (name: string): StatusEntry | undefined => entries()[name],
    set: (name: string, message: string, kind: StatusKind = "") =>
      setEntries((previous) => ({ ...previous, [name]: { message, kind } })),
  });
}

export type StatusBoard = ReturnType<typeof createStatusBoard>;

/** Browser effects pages may trigger, injected so they stay explicit. */
export type BrowserEffects = Readonly<{
  confirm: (message: string) => boolean;
  reloadAfter: (delayMs: number) => void;
}>;

/** Shell state shared with every page. */
export type Shell = Readonly<{
  api: ApiClient;
  info: Accessor<Record<string, unknown> | null>;
  refreshInfo: () => Promise<boolean>;
  status: StatusBoard;
  theme: Accessor<Theme>;
  applyTheme: (theme: Theme) => void;
  page: Accessor<Page>;
  browser: BrowserEffects;
}>;

export function Status(props: { shell: Shell; name: string }): JSX.Element {
  return (
    <p class="form-status" data-status={props.name} data-kind={props.shell.status.entry(props.name)?.kind ?? ""} role="status">
      {props.shell.status.entry(props.name)?.message ?? ""}
    </p>
  );
}

/** Text bound to one `/api/system/info` field, formatted like the current variant. */
export function Bind(props: { shell: Shell; field: string }): JSX.Element {
  return <>{formatMetric(props.field, props.shell.info()?.[props.field])}</>;
}

export function PageSection(props: {
  shell: Shell;
  page: Page;
  eyebrow: string;
  title: string;
  action?: JSX.Element;
  children: JSX.Element;
}): JSX.Element {
  const titleId = `${props.page}-title`;
  return (
    <section class="page" data-page={props.page} aria-labelledby={titleId} hidden={props.shell.page() !== props.page}>
      <div class="page-heading">
        <div><p class="eyebrow">{props.eyebrow}</p><h1 id={titleId}>{props.title}</h1></div>
        {props.action}
      </div>
      {props.children}
    </section>
  );
}

/** Text-valued entries of a form, as the current variant reads them. */
export function formValues(form: HTMLFormElement): Record<string, string> {
  const values: Record<string, string> = {};
  new FormData(form).forEach((value, key) => {
    if (typeof value === "string") values[key] = value;
  });
  return values;
}
