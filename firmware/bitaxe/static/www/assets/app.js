(function installBitaxeApplication(global) {
  "use strict";

  const core = global.BitaxeUiCore;
  const api = global.BitaxeApi;
  const documentRef = global.document;
  if (!core || !api || !documentRef) {
    return;
  }

  let logText = "";
  let logPaused = false;
  let maybeCloseLogStream = null;

  function element(selector) {
    return documentRef.querySelector(selector);
  }

  function setStatus(name, message, kind = "") {
    const target = element(`[data-status="${name}"]`);
    if (!target) {
      return;
    }
    target.textContent = message;
    target.dataset.kind = kind;
  }

  function setConnection(message, state) {
    const target = element("#connection-state");
    target.textContent = message;
    target.dataset.state = state;
  }

  function renderInfo(info) {
    for (const target of documentRef.querySelectorAll("[data-bind]")) {
      const field = target.dataset.bind;
      target.textContent = core.formatMetric(field, info?.[field]);
    }
    const hostname = element('#network-form input[name="hostname"]');
    const ssid = element('#network-form input[name="ssid"]');
    if (hostname && hostname.value === "" && typeof info?.hostname === "string") {
      hostname.value = info.hostname;
    }
    if (ssid && ssid.value === "" && typeof info?.ssid === "string") {
      ssid.value = info.ssid;
    }
  }

  async function refreshInfo() {
    setConnection("Connecting", "loading");
    try {
      renderInfo(await api.getInfo());
      setConnection("Device online", "ready");
      return true;
    } catch (error) {
      setConnection("Device unavailable", "error");
      setStatus("commands", core.publicError(error), "error");
      return false;
    }
  }

  function showPage(pathname, pushState) {
    const normalized = core.normalizePath(pathname);
    const page = core.routeFor(normalized);
    for (const section of documentRef.querySelectorAll("[data-page]")) {
      section.hidden = section.dataset.page !== page;
    }
    for (const link of documentRef.querySelectorAll("[data-nav]")) {
      if (link.dataset.nav === page) {
        link.setAttribute("aria-current", "page");
      } else {
        link.removeAttribute("aria-current");
      }
    }
    if (pushState && core.isKnownRoute(normalized)) {
      global.history.pushState({}, "", normalized);
    }
    documentRef.title = `Bitaxe · ${page[0].toUpperCase()}${page.slice(1)}`;
    element("#workspace").focus({ preventScroll: true });
    closeMobileMenu();
    if (page === "logs") {
      void startLogs();
    } else {
      stopLogs();
    }
  }

  function closeMobileMenu() {
    setMobileMenu(false);
  }

  function setMobileMenu(open) {
    const navigation = element("#primary-nav");
    const toggle = element("#menu-toggle");
    const mobile = global.matchMedia("(max-width: 920px)").matches;
    const visible = mobile && open;
    navigation.dataset.open = String(visible);
    navigation.inert = mobile && !visible;
    toggle.setAttribute("aria-expanded", String(visible));
  }

  function syncResponsiveNavigation() {
    const navigation = element("#primary-nav");
    const open = navigation.dataset.open === "true";
    setMobileMenu(open);
  }

  function formValues(form) {
    return Object.fromEntries(new global.FormData(form).entries());
  }

  async function submitSettings(kind, form) {
    const patch = core.buildSettingsPatch(kind, formValues(form));
    if (Object.keys(patch).length === 0) {
      setStatus(kind, "Enter at least one setting to change.", "error");
      return;
    }
    setStatus(kind, "Saving…");
    try {
      await api.patchSettings(patch);
      setStatus(kind, "Settings saved. Restart may be required.", "success");
      for (const field of core.patchFieldNames(kind)) {
        if (field.toLowerCase().includes("pass")) {
          form.elements.namedItem(field).value = "";
        }
      }
      void refreshInfo();
    } catch (error) {
      setStatus(kind, core.publicError(error), "error");
    }
  }

  async function runCommand(name) {
    const prompt = name === "restart"
      ? "Restart the device now?"
      : `${name === "pause" ? "Pause" : "Resume"} mining?`;
    if (!global.confirm(prompt)) {
      return;
    }
    setStatus("commands", "Sending command…");
    try {
      await api.command(name);
      setStatus("commands", `Command accepted: ${name}.`, "success");
      if (name !== "restart") {
        void refreshInfo();
      }
    } catch (error) {
      setStatus("commands", core.publicError(error), "error");
    }
  }

  function boundedLogs(text) {
    const combined = `${logText}${text}`;
    logText = combined.slice(Math.max(0, combined.length - 60000));
    renderLogs();
  }

  function renderLogs() {
    if (logPaused) {
      return;
    }
    const filter = element("#log-filter").value;
    const visible = filter === ""
      ? logText
      : logText.split("\n").filter((line) => line.includes(filter)).join("\n");
    const output = element("#log-output");
    output.textContent = visible || "No matching logs.";
    output.scrollTop = output.scrollHeight;
  }

  async function startLogs() {
    if (maybeCloseLogStream) {
      return;
    }
    try {
      logText = await api.retainedLogs();
      renderLogs();
      setStatus("logs", "Retained logs loaded.", "success");
    } catch (error) {
      setStatus("logs", core.publicError(error), "error");
    }
    maybeCloseLogStream = api.openLogStream(boundedLogs, (state) => {
      setStatus("logs", `Live stream: ${state}.`, state === "connected" ? "success" : "");
    });
  }

  function stopLogs() {
    if (maybeCloseLogStream) {
      maybeCloseLogStream();
      maybeCloseLogStream = null;
    }
  }

  function applyTheme(theme) {
    documentRef.documentElement.dataset.theme = theme.scheme;
    documentRef.documentElement.style.setProperty("--accent", theme.accent);
    const scheme = element('#theme-form select[name="colorScheme"]');
    const accent = element('#theme-form input[name="accentColor"]');
    scheme.value = theme.scheme;
    accent.value = theme.accent;
  }

  async function loadTheme() {
    try {
      applyTheme(core.themeFromPayload(await api.getTheme()));
    } catch {
      applyTheme(core.themeFromPayload(null));
    }
  }

  async function saveTheme(form) {
    const payload = core.themePayload(formValues(form));
    applyTheme(core.themeFromPayload(payload));
    setStatus("theme", "Saving…");
    try {
      await api.saveTheme(payload);
      setStatus("theme", "Theme saved.", "success");
    } catch (error) {
      setStatus("theme", core.publicError(error), "error");
    }
  }

  async function uploadFirmware(form) {
    const input = element("#firmware-file");
    const file = input.files?.[0];
    if (!file || file.name !== "esp-miner.bin") {
      setStatus("update", "Choose a file named esp-miner.bin.", "error");
      return;
    }
    if (!global.confirm("Upload this firmware image and restart the device?")) {
      return;
    }
    setStatus("update", "Uploading firmware. Do not disconnect power.");
    try {
      await api.uploadFirmware(file);
      setStatus("update", "Firmware accepted. The device will restart.", "success");
      form.reset();
      element("#firmware-upload").disabled = true;
    } catch (error) {
      setStatus("update", core.publicError(error), "error");
    }
  }

  function installEvents() {
    documentRef.addEventListener("click", (event) => {
      const route = event.target.closest?.("[data-route]");
      if (route) {
        event.preventDefault();
        showPage(route.getAttribute("href"), true);
        return;
      }
      const command = event.target.closest?.("[data-command]");
      if (command) {
        void runCommand(command.dataset.command);
      }
    });
    global.addEventListener("popstate", () => showPage(global.location.pathname, false));
    global.addEventListener("resize", syncResponsiveNavigation);
    element("#menu-toggle").addEventListener("click", () => {
      const navigation = element("#primary-nav");
      const open = navigation.dataset.open !== "true";
      setMobileMenu(open);
    });
    element('[data-action="refresh-info"]').addEventListener("click", () => void refreshInfo());
    for (const kind of ["network", "pool", "settings"]) {
      element(`#${kind}-form`).addEventListener("submit", (event) => {
        event.preventDefault();
        void submitSettings(kind, event.currentTarget);
      });
    }
    element("#theme-form").addEventListener("submit", (event) => {
      event.preventDefault();
      void saveTheme(event.currentTarget);
    });
    element("#log-filter").addEventListener("input", renderLogs);
    element("#log-pause").addEventListener("click", (event) => {
      logPaused = !logPaused;
      event.currentTarget.textContent = logPaused ? "Resume" : "Pause";
      renderLogs();
    });
    element("#log-download").addEventListener("click", api.downloadLogs);
    element("#log-clear").addEventListener("click", () => { logText = ""; renderLogs(); });
    element("#firmware-file").addEventListener("change", (event) => {
      const file = event.currentTarget.files?.[0];
      element("#firmware-upload").disabled = !file || file.name !== "esp-miner.bin";
    });
    element("#firmware-update-form").addEventListener("submit", (event) => {
      event.preventDefault();
      void uploadFirmware(event.currentTarget);
    });
  }

  installEvents();
  showPage(global.location.pathname, false);
  void Promise.all([refreshInfo(), loadTheme()]);
})(globalThis);
