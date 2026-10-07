// Application shell: history routing, responsive navigation, theme and footer.
// Upstream AxeOS breadcrumb (feature reference only):
// `reference/esp-miner/main/http_server/axe-os/src/app/layout/`.

import { For, createEffect, createSignal, on, onCleanup, type JSX } from "solid-js";

import type { ApiClient } from "../api/api-client.js";
import { isKnownRoute, normalizePath, pageTitle, publicError, routeFor, themeFromPayload, type Page, type Theme } from "../core/ui-core.js";
import { Dashboard } from "./dashboard.js";
import { LogsPage, ScoreboardPage } from "./diagnostics.js";
import { ThemePage, UpdatePage } from "./maintenance.js";
import { NetworkPage, PoolPage, SettingsPage } from "./settings-forms.js";
import { createStatusBoard, type BrowserEffects, type Shell } from "./shared.js";

const NAVIGATION: readonly (readonly [string, Page, string])[] = [
  ["/", "dashboard", "Dashboard"],
  ["/network", "network", "Network"],
  ["/pool", "pool", "Pool"],
  ["/settings", "settings", "Settings"],
  ["/scoreboard", "scoreboard", "Scoreboard"],
  ["/logs", "logs", "Logs"],
  ["/update", "update", "Update"],
  ["/design", "theme", "Theme"],
];
const MOBILE_QUERY = "(max-width: 920px)";

type Connection = Readonly<{ message: string; maybeState: "loading" | "ready" | "error" | null }>;

/** Mobile drawer state: open only while mobile, otherwise hidden from assistive tech via `inert`. */
function createResponsiveNavigation() {
  const media = globalThis.matchMedia(MOBILE_QUERY);
  const [mobile, setMobile] = createSignal(media.matches);
  const [open, setOpen] = createSignal(false);
  const setMenu = (requested: boolean) => setOpen(mobile() && requested);
  const onResize = () => { setMobile(media.matches); setMenu(open()); };
  globalThis.addEventListener("resize", onResize);
  onCleanup(() => globalThis.removeEventListener("resize", onResize));
  return { mobile, open, setMenu };
}

export function App(props: { api: ApiClient; browser: BrowserEffects }): JSX.Element {
  // `equals: false`: revisiting the open page refreshes it, like the current variant.
  const [page, setPage] = createSignal<Page>(routeFor(globalThis.location.pathname), { equals: false });
  const [info, setInfo] = createSignal<Record<string, unknown> | null>(null);
  const [theme, setTheme] = createSignal<Theme>(themeFromPayload(null));
  const [connection, setConnection] = createSignal<Connection>({ message: "Connecting", maybeState: null });
  const status = createStatusBoard();
  const navigation = createResponsiveNavigation();
  let maybeWorkspace: HTMLElement | undefined;
  let maybeSidebar: HTMLElement | undefined;

  async function refreshInfo(): Promise<boolean> {
    setConnection({ message: "Connecting", maybeState: "loading" });
    try {
      const payload = await props.api.getInfo();
      setInfo(payload !== null && typeof payload === "object" ? (payload as Record<string, unknown>) : {});
      setConnection({ message: "Device online", maybeState: "ready" });
      return true;
    } catch (error) {
      setConnection({ message: "Device unavailable", maybeState: "error" });
      status.set("commands", publicError(error), "error");
      return false;
    }
  }

  const shell: Shell = { api: props.api, info, refreshInfo, status, theme, applyTheme: setTheme, page, browser: props.browser };

  function showPage(pathname: string, pushState: boolean): void {
    const normalized = normalizePath(pathname);
    if (pushState && isKnownRoute(normalized)) globalThis.history.pushState({}, "", normalized);
    setPage(routeFor(normalized));
    navigation.setMenu(false);
  }

  function onRouteClick(event: MouseEvent): void {
    event.preventDefault();
    showPage((event.currentTarget as HTMLAnchorElement).getAttribute("href") ?? "/", true);
  }

  const onPopState = () => showPage(globalThis.location.pathname, false);
  globalThis.addEventListener("popstate", onPopState);
  onCleanup(() => globalThis.removeEventListener("popstate", onPopState));

  createEffect(on(page, (current) => {
    globalThis.document.title = pageTitle(current);
    maybeWorkspace?.focus({ preventScroll: true });
  }));
  createEffect(() => {
    const root = globalThis.document.documentElement;
    root.dataset["theme"] = theme().scheme;
    root.style.setProperty("--accent", theme().accent);
  });
  createEffect(() => {
    if (maybeSidebar === undefined) return;
    maybeSidebar.dataset["open"] = String(navigation.open());
    maybeSidebar.inert = navigation.mobile() && !navigation.open();
  });

  void Promise.all([
    refreshInfo(),
    props.api.getTheme().then((payload) => setTheme(themeFromPayload(payload)), () => setTheme(themeFromPayload(null))),
  ]);

  return (
    <>
      <a class="skip-link" href="#workspace">Skip to content</a>
      <div class="app-shell">
        <header class="topbar">
          <button id="menu-toggle" class="icon-button mobile-only" type="button" aria-controls="primary-nav" aria-expanded={navigation.open()} onClick={() => navigation.setMenu(!navigation.open())}>
            <span aria-hidden="true">☰</span><span class="sr-only">Toggle navigation</span>
          </button>
          <a class="brand" href="/" onClick={onRouteClick}>
            <span class="brand-mark" aria-hidden="true">₿</span>
            <span><strong>Bitaxe</strong><small>Rust Firmware</small></span>
          </a>
          <p id="connection-state" class="connection-state" role="status" data-state={connection().maybeState ?? undefined}>{connection().message}</p>
        </header>
        <aside id="primary-nav" class="sidebar" aria-label="Primary navigation" ref={maybeSidebar}>
          <nav>
            <For each={NAVIGATION}>
              {([href, target, label]) => (
                <a href={href} aria-current={page() === target ? "page" : undefined} onClick={onRouteClick}><span aria-hidden="true">⌁</span>{label}</a>
              )}
            </For>
          </nav>
          <div class="sidebar-note"><span class="status-dot" aria-hidden="true"></span><span>Ultra 205 operator console</span></div>
        </aside>
        <main id="workspace" class="workspace" tabindex="-1" ref={maybeWorkspace}>
          <Dashboard shell={shell} />
          <NetworkPage shell={shell} />
          <PoolPage shell={shell} />
          <SettingsPage shell={shell} />
          <ScoreboardPage shell={shell} />
          <LogsPage shell={shell} />
          <UpdatePage shell={shell} />
          <ThemePage shell={shell} />
        </main>
        <footer class="footer">
          <span>Independent Rust firmware UI</span>
          <a href="https://github.com/bright-builds-llc/bitaxe-esp-miner" target="_blank" rel="noopener noreferrer">Source on GitHub</a>
          <a href="https://openlinks.us/" target="_blank" rel="noopener noreferrer">By Peter Ryszkiewicz</a>
        </footer>
      </div>
    </>
  );
}
