import { pageOptions } from '/heartbeat-page-options.mjs';
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  const suppress = window.workerAcceptance.suppressHeartbeats.bind(window.workerAcceptance);
  globalThis.str005CustomPage = true;
  const { installPage } = await import('/shared-page.mjs');
  installPage(pageOptions(suppress));
}
