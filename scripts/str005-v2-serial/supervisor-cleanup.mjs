/** Stop admission synchronously, close independent owners, then join requests.
 * Every cleanup is attempted; the first observed failure remains authoritative.
 */
export function createSupervisorCleanup({ stop, closeFixture, closeObserver, settleQueue, fail, settled }) {
  let maybeClosing;
  return function close() {
    if (maybeClosing) return maybeClosing;
    let resolveClose, rejectClose, maybeFirst;
    maybeClosing = new Promise((done, reject) => { resolveClose = done; rejectClose = reject; });
    const capture = (error, code) => { maybeFirst ??= error; fail(error?.code ?? code); };
    try { stop(); } catch (error) { capture(error, "v2_supervisor_stop_failed"); }
    const attempt = async (operation, code) => {
      try { return await operation(); }
      catch (error) { capture(error, code); throw error; }
    };
    (async () => {
      const results = await Promise.allSettled([
        attempt(closeFixture, "v2_fixture_cleanup_failed"),
        attempt(closeObserver, "v2_observer_cleanup_failed"),
      ]);
      for (const result of results) if (result.status === "rejected") maybeFirst ??= result.reason;
      try { await settleQueue(); } catch (error) { capture(error, "v2_request_cleanup_failed"); }
      try { await settled(); } catch (error) { capture(error, "v2_failure_evidence_failed"); }
      if (maybeFirst) throw maybeFirst;
    })().then(resolveClose, rejectClose);
    return maybeClosing;
  };
}
