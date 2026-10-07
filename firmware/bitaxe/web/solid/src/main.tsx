// Entry point of the SolidJS operator UI variant.
// The stylesheet is shared with the `current` variant so both render identically.
import "../../../static/www/assets/app.css";

import { render } from "solid-js/web";

import { browserServices, createApiClient } from "./api/api-client.js";
import { App } from "./ui/app.js";

const maybeMount = document.getElementById("app");
if (maybeMount !== null) {
  const browser = Object.freeze({
    confirm: (message: string) => globalThis.confirm(message),
    reloadAfter: (delayMs: number) => { globalThis.setTimeout(() => globalThis.location.reload(), delayMs); },
  });
  render(() => <App api={createApiClient(browserServices())} browser={browser} />, maybeMount);
}
