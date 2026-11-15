import assert from "node:assert/strict";
import test from "node:test";
import { macroCopy } from "./macros.js";

test("M3: no intake is a dash, never a zero", () => {
  const copy = macroCopy({ kind: "absent" });
  assert.equal(copy.kind, "absent");
  assert.equal(copy.meanText, "—");
  assert.equal(copy.note, "");
  assert.equal(copy.pill, null);
  assert.equal(`${copy.meanText}${copy.note}`.includes("0"), false);
  assert.deepEqual(macroCopy(null), copy);
});

test("M2: a mean of 0 g is a real zero, not a dash", () => {
  const copy = macroCopy({ kind: "present", meanG: 0, days: 1, gapG: null });
  assert.equal(copy.kind, "present");
  assert.equal(copy.meanText, "0 g/j");
  assert.equal(copy.note, "1 jour saisi");
});

test("M4: a missing previous week is a mean without a gap", () => {
  const copy = macroCopy({ kind: "present", meanG: 142, days: 5, gapG: null });
  assert.equal(copy.meanText, "142 g/j");
  assert.equal(copy.note, "5 jours saisis");
  assert.equal(copy.pill, null);
});

test("M4: a positive gap is a signed up pill", () => {
  const copy = macroCopy({ kind: "present", meanG: 142, days: 5, gapG: 12 });
  assert.deepEqual(copy.pill, { text: "+12 vs semaine d'avant", tone: "up" });
});

test("M4: a negative gap uses the minus sign and a down pill", () => {
  const copy = macroCopy({ kind: "present", meanG: 100, days: 2, gapG: -42 });
  assert.deepEqual(copy.pill, { text: "−42 vs semaine d'avant", tone: "down" });
});

test("M4: a real tie is a gap of 0 with a neutral tone, not an absent gap", () => {
  const copy = macroCopy({ kind: "present", meanG: 120, days: 2, gapG: 0 });
  assert.deepEqual(copy.pill, { text: "0 vs semaine d'avant", tone: "even" });
});
