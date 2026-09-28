import { check, object } from '../str005-v2-serial/values.mjs';
/** Natural completion is qualification evidence; close/reap and auxiliary release are ownership evidence. */
export function createReleaseOwner({ getFixture, finishFixture, assertFixtureReleased, releaseAuxiliary, persist }) {
  let maybeRelease;
  async function release() {
    const fixture = getFixture(), failures = [];
    const completion = { schema: 'str005-startup-fixture-completion-v1', required: Boolean(fixture && finishFixture), complete: null };
    let fixtureReleased = !fixture, auxiliaryReleased = true;
    if (fixture) {
      if (finishFixture) {
        try { await finishFixture(fixture); completion.complete = true; }
        catch (error) { completion.complete = false; failures.push(error); }
      }
      try { await fixture.close(); await assertFixtureReleased(fixture); fixtureReleased = true; }
      catch (error) { failures.push(error); }
    }
    try { if (releaseAuxiliary) await releaseAuxiliary(); }
    catch (error) { auxiliaryReleased = false; failures.push(error); }
    // Neither persistence failure may prevent the other receipt or any cleanup operation.
    for (const [name, value] of [
      ['fixture-completion.json', completion],
      ['fixture-release.json', { schema: 'str005-startup-fixture-release-v2', fixtureStarted: Boolean(fixture),
        fixtureReleased, auxiliaryReleased, complete: fixtureReleased && auxiliaryReleased }],
    ]) {
      try { await persist(name, value); } catch (error) { failures.push(error); }
    }
    if (failures.length === 1) throw failures[0];
    if (failures.length > 1) throw new AggregateError(failures, 'Fixture completion or ownership release failed');
  }
  return () => { maybeRelease ??= release(); return maybeRelease; };
}
/** Historical v1 is interpreted exactly as emitted; no inferred correction of sealed results. */
export function fixtureReleaseComplete(value) {
  if (value === undefined) return false;
  if (value.schema === 'str005-startup-fixture-release-v1') {
    object(value, ['schema', 'complete']); check(typeof value.complete === 'boolean', 'startup_fixture_release_shape');
    return value.complete;
  }
  object(value, ['schema', 'fixtureStarted', 'fixtureReleased', 'auxiliaryReleased', 'complete']);
  check(value.schema === 'str005-startup-fixture-release-v2' &&
    ['fixtureStarted', 'fixtureReleased', 'auxiliaryReleased', 'complete'].every(key => typeof value[key] === 'boolean') &&
    (value.fixtureStarted || value.fixtureReleased) && value.complete === (value.fixtureReleased && value.auxiliaryReleased),
  'startup_fixture_release_shape');
  return value.complete;
}
export function fixtureCompletionComplete(value) {
  object(value, ['schema', 'required', 'complete']);
  check(value.schema === 'str005-startup-fixture-completion-v1' && typeof value.required === 'boolean' &&
    (value.required ? typeof value.complete === 'boolean' : value.complete === null), 'startup_fixture_completion_shape');
  return value.required && value.complete;
}
