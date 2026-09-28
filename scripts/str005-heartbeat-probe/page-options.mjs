import { createHeartbeatCoordinator } from './client.mjs';
/** Compose the qualified startup admission/recovery page; only the live observation differs. */
export function pageOptions(suppressHeartbeats) {
  return {
    async beforeFixture({ gate, post, endpoint }) {
      const challenge = await post('/observer/context', {});
      await post('/observer/start', { nonce: challenge.nonce, endpoint, state: gate.state() });
    },
    coordinator(options) {
      const { post } = options; let admitted;
      return createHeartbeatCoordinator({ ...options, gate: { ...options.gate, suppressHeartbeats },
        prepare: async () => { admitted = await options.prepare(); return admitted; },
        readStatus: () => options.gate.stratumV2Status('share', admitted.attemptId, admitted.binding),
        claimStart: () => post('/heartbeat/start-claim', {}),
        recordSuppression: value => post('/heartbeat/suppressed', value),
        record: async value => {
          if (value.dispatchStatus) await post('/heartbeat/observations', { status: value.dispatchStatus });
          return options.record(value);
        },
        observer: { requireArmed: () => post('/heartbeat/alive', {}), requireAlive: () => post('/heartbeat/alive', {}),
          waitTail: () => post('/heartbeat/tail', {}), stop: () => post('/heartbeat/release', {}) },
      });
    },
  };
}
