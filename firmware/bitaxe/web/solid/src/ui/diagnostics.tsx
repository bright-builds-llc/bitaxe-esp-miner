// Scoreboard and logs pages.
// Upstream AxeOS breadcrumbs (feature reference only):
// `reference/esp-miner/main/http_server/axe-os/src/app/components/scoreboard/`
// and `.../components/logs/`.

import { For, createEffect, createSignal, on, onCleanup, type JSX } from "solid-js";

import { publicError, scoreboardCells, scoreboardRows, type ScoreboardRow } from "../core/ui-core.js";
import { appendBoundedLog, visibleLogText } from "../core/workflows.js";
import { PageSection, Status, type Shell } from "./shared.js";

export function ScoreboardPage(props: { shell: Shell }): JSX.Element {
  const [rows, setRows] = createSignal<readonly ScoreboardRow[]>([]);
  async function refresh(): Promise<void> {
    props.shell.status.set("scoreboard", "Loading…");
    try {
      setRows(scoreboardRows(await props.shell.api.getScoreboard()));
      props.shell.status.set("scoreboard", "Scoreboard refreshed.", "success");
    } catch (error) {
      setRows([]);
      props.shell.status.set("scoreboard", publicError(error), "error");
    }
  }
  createEffect(on(props.shell.page, (page) => { if (page === "scoreboard") void refresh(); }));
  const action = <button class="button secondary" type="button" onClick={() => void refresh()}>Refresh</button>;
  return (
    <PageSection shell={props.shell} page="scoreboard" eyebrow="Best valid nonces" title="Scoreboard" action={action}>
      <article class="card scoreboard-card">
        <div class="scoreboard-scroll">
          <table id="scoreboard-table" hidden={rows().length === 0}>
            <caption class="sr-only">Top valid nonce scoreboard</caption>
            <thead><tr><th>Rank</th><th>Difficulty</th><th>Job</th><th>Extranonce</th><th>Time</th><th>Nonce</th><th>Version bits</th></tr></thead>
            <tbody id="scoreboard-rows">
              <For each={rows()}>{(row, index) => <tr><For each={scoreboardCells(row, index())}>{(cell) => <td>{cell}</td>}</For></tr>}</For>
            </tbody>
          </table>
        </div>
        <p id="scoreboard-empty" class="muted" hidden={rows().length !== 0}>No valid nonce results recorded yet.</p>
        <Status shell={props.shell} name="scoreboard" />
      </article>
    </PageSection>
  );
}

export function LogsPage(props: { shell: Shell }): JSX.Element {
  let logText = "";
  let maybeCloseStream: (() => void) | null = null;
  let maybeOutput: HTMLPreElement | undefined;
  const [filter, setFilter] = createSignal("");
  const [paused, setPaused] = createSignal(false);
  const [shown, setShown] = createSignal("Waiting for logs…");

  // Paused views keep their last text while new lines keep arriving.
  function render(): void {
    if (paused()) return;
    setShown(visibleLogText(logText, filter()));
    if (maybeOutput !== undefined) maybeOutput.scrollTop = maybeOutput.scrollHeight;
  }

  async function start(): Promise<void> {
    if (maybeCloseStream !== null) return;
    try {
      logText = await props.shell.api.retainedLogs();
      render();
      props.shell.status.set("logs", "Retained logs loaded.", "success");
    } catch (error) {
      props.shell.status.set("logs", publicError(error), "error");
    }
    maybeCloseStream = props.shell.api.openLogStream(
      (text) => { logText = appendBoundedLog(logText, text); render(); },
      (state) => props.shell.status.set("logs", `Live stream: ${state}.`, state === "connected" ? "success" : ""),
    );
  }

  function stop(): void {
    maybeCloseStream?.();
    maybeCloseStream = null;
  }

  createEffect(on(props.shell.page, (page) => { if (page === "logs") void start(); else stop(); }));
  onCleanup(stop);

  const togglePause = () => { setPaused(!paused()); render(); };
  const clear = () => { logText = ""; render(); };
  return (
    <PageSection shell={props.shell} page="logs" eyebrow="Diagnostics" title="Logs">
      <article class="card logs-card">
        <div class="logs-toolbar">
          <label>Filter<input id="log-filter" type="search" autocomplete="off" placeholder="Filter visible lines" onInput={(event) => { setFilter(event.currentTarget.value); render(); }} /></label>
          <div class="button-row">
            <button class="button secondary" id="log-pause" type="button" onClick={togglePause}>{paused() ? "Resume" : "Pause"}</button>
            <button class="button secondary" id="log-download" type="button" onClick={() => props.shell.api.downloadLogs()}>Download</button>
            <button class="button secondary" id="log-clear" type="button" onClick={clear}>Clear view</button>
          </div>
        </div>
        <pre id="log-output" tabindex="0" aria-live="polite" ref={maybeOutput}>{shown()}</pre>
        <Status shell={props.shell} name="logs" />
      </article>
    </PageSection>
  );
}
