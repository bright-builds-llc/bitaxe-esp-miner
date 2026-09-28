import { createShareCoordinator } from '/share-client.mjs';
globalThis.str005CustomPage = true;
const { installPage } = await import('/shared-page.mjs');
installPage({ coordinator: createShareCoordinator });
