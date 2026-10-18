import assert from "node:assert/strict";
import test from "node:test";
import { poolSaved, poolSettings } from "./pool.mjs";

const pool = { endpoint: "stratum+tcp://pool.invalid:3333/", username: "owner.worker", password: "x" };

test("the pool is saved as a bare host with its port", () => {
  // Arrange / Act
  const settings = poolSettings(pool);
  // Assert
  assert.deepEqual(settings, { stratumURL: "pool.invalid", stratumPort: 3333, stratumUser: "owner.worker", stratumPassword: "x" });
});

test("a saved pool matches on host, port and user", () => {
  // Arrange
  const settings = poolSettings(pool);
  // Act / Assert
  assert.equal(poolSaved({ stratumURL: "pool.invalid", stratumPort: 3333, stratumUser: "owner.worker" }, settings), true);
  assert.equal(poolSaved({ stratumURL: "pool.invalid", stratumPort: 3334, stratumUser: "owner.worker" }, settings), false);
  assert.equal(poolSaved({ stratumURL: "other.invalid", stratumPort: 3333, stratumUser: "owner.worker" }, settings), false);
});
