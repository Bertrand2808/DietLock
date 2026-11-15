import assert from "node:assert/strict";
import test from "node:test";
import { pickPeriod } from "./macro_period.js";

const summary = {
  sevenDays: { protein: { kind: "absent" }, proteinDays: new Array(7).fill(null) },
  fifteenDays: { protein: { kind: "present", meanG: 100, days: 1, gapG: null }, proteinDays: new Array(15).fill(null) },
};

test("M12: seven days picks the seven-day period", () => {
  assert.equal(pickPeriod(summary, 7), summary.sevenDays);
});

test("M12: fifteen days picks the fifteen-day period", () => {
  assert.equal(pickPeriod(summary, 15), summary.fifteenDays);
});

test("M12: an unknown period falls back to seven days", () => {
  assert.equal(pickPeriod(summary, 30), summary.sevenDays);
});

test("M12: no summary yet gives no period", () => {
  assert.equal(pickPeriod(null, 7), null);
});
