import assert from "node:assert/strict";
import test from "node:test";
import { energyCopy } from "./energy.js";

test("E3: no intake is a sentence, not a zero", () => {
  const copy = energyCopy({ kind: "absent" }, 2432);
  assert.equal(copy.kind, "absent");
  assert.equal(copy.kcalText, null);
  assert.equal(copy.sentence, "Pas de moyenne sur 7 jours.");
  assert.equal(copy.note, "");
  assert.equal(copy.pill, null);
  const shown = `${copy.sentence}${copy.note}`;
  assert.equal(shown.includes("0"), false);
  assert.equal(shown.includes("hier"), false);
  assert.equal(shown.includes("2432"), false);
  assert.equal(energyCopy(null).sentence, copy.sentence);
});

test("E2: a mean of 0 kcal stays a zero and the day is counted", () => {
  const copy = energyCopy({ kind: "present", kcal: 0, days: 1, gapKcal: null });
  assert.equal(copy.kcalText, "0");
  assert.equal(copy.note, "1 jour saisi");
  assert.equal(copy.pill, null);
});

test("E4: a missing previous week is not a gap of zero", () => {
  const copy = energyCopy({ kind: "present", kcal: 1320, days: 5, gapKcal: null }, 2432);
  assert.equal(copy.kcalText, "1\u202f320");
  assert.equal(copy.note, "5 jours saisis · hier 2\u202f432");
  assert.equal(copy.pill, null);
  assert.equal(copy.note.includes("+0"), false);
  assert.equal(copy.note.includes("\u22120"), false);
  assert.equal(copy.note.includes("kcal"), false);
});

test("a negative gap is a down pill and a positive gap is an up pill", () => {
  const down = energyCopy({ kind: "present", kcal: 1000, days: 2, gapKcal: -80 });
  assert.deepEqual(down.pill, { text: "\u221280 vs semaine d'avant", tone: "down" });
  assert.equal(down.note, "2 jours saisis");
  const up = energyCopy({ kind: "present", kcal: 1320, days: 5, gapKcal: 120 });
  assert.deepEqual(up.pill, { text: "+120 vs semaine d'avant", tone: "up" });
  assert.equal(up.pill.tone === "down", false);
  const wide = energyCopy({ kind: "present", kcal: 2000, days: 3, gapKcal: -1200 }, 2432);
  assert.deepEqual(wide.pill, { text: "\u22121\u202f200 vs semaine d'avant", tone: "down" });
  assert.equal(wide.note, "3 jours saisis · hier 2\u202f432");
});

test("a gap of 0 keeps +0 kcal and is not a pill", () => {
  const tie = energyCopy({ kind: "present", kcal: 1200, days: 4, gapKcal: 0 }, 800);
  assert.equal(tie.pill, null);
  assert.equal(tie.note, "4 jours saisis · hier 800 · +0 kcal");
  assert.equal(tie.note.includes("+0 kcal"), true);
  assert.equal(tie.note.includes("vs semaine"), false);
});

test("no intake yesterday omits hier, and a real zero stays", () => {
  const week = { kind: "present", kcal: 1700, days: 2, gapKcal: -200 };
  const missing = energyCopy(week, null);
  assert.equal(missing.note.includes("hier"), false);
  assert.equal(missing.note, "2 jours saisis");
  const zero = energyCopy(week, 0);
  assert.equal(zero.note, "2 jours saisis · hier 0");
});
