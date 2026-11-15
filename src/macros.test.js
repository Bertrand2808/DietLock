import assert from "node:assert/strict";
import test from "node:test";
import { MACRO_BLOCKS, macroCopy, macroPeriodCaption } from "./macros.js";

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

test("M13: the gap wording follows the period, not always the week", () => {
  const week = { kind: "present", meanG: 100, days: 5, gapG: 8 };
  assert.equal(macroCopy(week, 7).pill.text, "+8 vs semaine d'avant");
  assert.equal(macroCopy(week).pill.text, "+8 vs semaine d'avant");
  assert.equal(macroCopy(week, 15).pill.text, "+8 vs 15 jours d'avant");
  const down = { kind: "present", meanG: 100, days: 5, gapG: -3 };
  assert.equal(macroCopy(down, 15).pill.text, "−3 vs 15 jours d'avant");
  const tie = { kind: "present", meanG: 100, days: 5, gapG: 0 };
  assert.equal(macroCopy(tie, 15).pill.text, "0 vs 15 jours d'avant");
});

test("M14: the blocks are Protéines, Glucides, Lipides in that order", () => {
  assert.deepEqual(
    MACRO_BLOCKS.map((block) => block.key),
    ["protein", "carbs", "fat"],
  );
  assert.deepEqual(
    MACRO_BLOCKS.map((block) => block.title),
    ["Protéines", "Glucides", "Lipides"],
  );
});

test("M13: the caption names the period", () => {
  assert.equal(macroPeriodCaption(7), "7 derniers jours, hier compris");
  assert.equal(macroPeriodCaption(15), "15 derniers jours, hier compris");
});
