import { createShareCoordinator } from '/share-client.mjs';
globalThis.str005CustomPage = true;
const { installPage } = await import('/shared-page.mjs');
// Must equal the accepted-share contract's MINIMUM_RENEWALS and OBSERVE_WINDOW_MS (checked by share.test.mjs).
installPage({ coordinator: createShareCoordinator, runLabel: 'Run one renewal probe',
  coordinatorOptions: { limits: { minRenewals: 1, observeMs: 80000 } } });
