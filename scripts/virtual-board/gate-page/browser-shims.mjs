// The browser surface the pinned Gate page and the supervisor client touch, for Bun.
// Only globals are replaced; the page's own runtime (`browserSerialRuntime`) runs unchanged.

/** Elements carry the attributes the page and client use; `#state` notifies observers. */
function element(id, observers) {
  let text = '';
  const listeners = new Map();
  return {
    id, hidden: false, removed: false,
    get textContent() { return text; },
    set textContent(value) {
      text = String(value);
      for (const observer of observers) if (observer.target === this) queueMicrotask(() => observer.callback([]));
    },
    addEventListener(type, listener) { listeners.set(type, [...(listeners.get(type) ?? []), listener]); },
    removeEventListener(type, listener) { listeners.set(type, (listeners.get(type) ?? []).filter(item => item !== listener)); },
    dispatch(type, event) { for (const listener of listeners.get(type) ?? []) listener(event); },
    after() {}, append() {}, remove() { this.removed = true; },
  };
}

/** Installs the shims; `fetch` resolves page-relative paths against `origin` like a browser. */
export function installBrowser({ origin, port }) {
  const observers = [];
  const elements = new Map();
  const byId = id => {
    if (!elements.has(id)) elements.set(id, element(id, observers));
    return elements.get(id);
  };
  let lockHeld = false;
  const nativeFetch = globalThis.fetch;
  globalThis.fetch = (input, init = {}) => nativeFetch(new URL(String(input), origin),
    { ...init, headers: { ...(init.headers ?? {}), origin } });
  Object.assign(globalThis.navigator, {
    serial: { getPorts: async () => [], requestPort: async () => port },
    userActivation: { isActive: true },
    locks: {
      async request(name, options, callback) {
        if (lockHeld && options?.ifAvailable) return callback(null);
        lockHeld = true;
        try { return await callback({ name }); } finally { lockHeld = false; }
      },
    },
  });
  globalThis.MutationObserver = class {
    constructor(callback) { this.callback = callback; }
    observe(target) { this.target = target; observers.push(this); }
    disconnect() { observers.splice(observers.indexOf(this), 1); }
  };
  globalThis.document = {
    visibilityState: 'visible',
    hasFocus: () => true,
    body: element('body', observers),
    getElementById: byId,
    querySelector: selector => selector.startsWith('#') ? byId(selector.slice(1)) : null,
    createElement: () => element('', observers),
    addEventListener() {}, removeEventListener() {},
  };
  globalThis.window = globalThis;
  globalThis.addEventListener ??= () => {};
  globalThis.removeEventListener ??= () => {};
  return { byId, lockHeld: () => lockHeld };
}
