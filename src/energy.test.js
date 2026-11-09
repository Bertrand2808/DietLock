import assert from "node:assert/strict";
import test from "node:test";
import { energyCopy } from "./energy.js";

test("E3: no intake is a sentence, not a zero", () => {
  const copy = energyCopy({ kind: "absent" });
  assert.equal(copy.kind, "absent");
  assert.equal(copy.kcalText, null);
  assert.equal(copy.sentence, "Pas de moyenne sur 7 jours.");
  assert.equal(copy.note, "");
  assert.equal(`${copy.sentence}${copy.note}`.includes("0"), false);
  assert.equal(energyCopy(null).sentence, copy.sentence);
});

test("E2: a mean of 0 kcal stays a zero and the day is counted", () => {
  const copy = energyCopy({ kind: "present", kcal: 0, days: 1, gapKcal: null });
  assert.equal(copy.kcalText, "0");
  assert.equal(copy.note, "sur 1 jour saisi · pas de comparaison");
});

test("E4: a missing previous week is not a gap of zero", () => {
  const copy = energyCopy({ kind: "present", kcal: 1320, days: 5, gapKcal: null });
  assert.equal(copy.kcalText, "1\u202f320");
  assert.equal(copy.note, "sur 5 jours saisis · pas de comparaison");
  assert.equal(copy.note.includes("+0"), false);
  assert.equal(copy.note.includes("\u22120"), false);
});

test("E1: the signed gap is grouped, and a tie keeps its sign", () => {
  const up = energyCopy({ kind: "present", kcal: 1320, days: 5, gapKcal: 120 });
  assert.equal(up.note, "sur 5 jours saisis · +120 kcal");
  const down = energyCopy({ kind: "present", kcal: 1000, days: 2, gapKcal: -80 });
  assert.equal(down.note, "sur 2 jours saisis · \u221280 kcal");
  const tie = energyCopy({ kind: "present", kcal: 1200, days: 4, gapKcal: 0 });
  assert.equal(tie.note, "sur 4 jours saisis · +0 kcal");
  const wide = energyCopy({ kind: "present", kcal: 2000, days: 3, gapKcal: -1200 });
  assert.equal(wide.note, "sur 3 jours saisis · \u22121\u202f200 kcal");
});
