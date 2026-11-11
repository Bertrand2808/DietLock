import assert from "node:assert/strict";
import test from "node:test";
import {
  ACTIVITY_TYPES,
  addRow,
  emptyRow,
  formKeyAction,
  removeRow,
  rowIsEmpty,
  saveButtonText,
  saveCount,
  validateRows,
} from "./activities.js";

const row = (type, note = "") => ({ ...emptyRow(), type, note });

test("AC1: the type list is the six labels in order, keyed like the core", () => {
  assert.deepEqual(
    ACTIVITY_TYPES.map((t) => t.label),
    ["Marche", "Course", "Mobilité", "Muscu", "Vélo", "Autre"],
  );
  assert.deepEqual(
    ACTIVITY_TYPES.map((t) => t.key),
    ["walk", "run", "mobility", "strength", "bike", "other"],
  );
});

test("AC3: the form starts with one empty row and rows get distinct ids", () => {
  const a = emptyRow();
  const b = emptyRow();
  assert.deepEqual([a.type, a.note], ["", ""]);
  assert.notEqual(a.id, b.id);
  assert.equal(rowIsEmpty(a), true);
  assert.equal(rowIsEmpty(row("run")), false);
  assert.equal(rowIsEmpty(row("", "x")), false);
  assert.equal(rowIsEmpty(row("", "   ")), true);
});

test("AC3: a second and third activity can be added before validating", () => {
  let rows = [row("walk")];
  rows = addRow(rows);
  rows = addRow(rows);
  assert.equal(rows.length, 3);
  assert.equal(rows[0].type, "walk");
  assert.equal(rowIsEmpty(rows[2]), true);
});

test("AC3: removing a row keeps the others, and the last row is never removed", () => {
  const rows = [row("walk"), row("run"), row("bike")];
  assert.deepEqual(
    removeRow(rows, rows[1].id).map((r) => r.type),
    ["walk", "bike"],
  );
  const single = [row("walk")];
  const after = removeRow(single, single[0].id);
  assert.equal(after.length, 1);
  assert.equal(rowIsEmpty(after[0]), true);
});

test("AC4: the button counts the rows that have a type", () => {
  assert.equal(saveCount([row("walk"), row(""), row("run", "x"), row("", "note")]), 2);
  assert.equal(saveCount([emptyRow()]), 0);
});

test("AC4: the button label agrees in number", () => {
  assert.equal(saveButtonText(0), "Enregistrer");
  assert.equal(saveButtonText(1), "Enregistrer 1 activité");
  assert.equal(saveButtonText(2), "Enregistrer 2 activités");
  assert.equal(saveButtonText(3), "Enregistrer 3 activités");
});

test("AC4: validation refuses a form without any type", () => {
  const result = validateRows([emptyRow(), emptyRow()]);
  assert.equal(result.ok, false);
  assert.equal(result.error, "Choisis au moins un type d'activité.");
});

test("AC4: a note without a type blocks the whole form", () => {
  const result = validateRows([row("walk"), row("", "oublié")]);
  assert.equal(result.ok, false);
  assert.equal(result.error, "Choisis un type pour chaque activité.");
});

test("AC3: validation keeps filled rows in order and drops empty ones", () => {
  const result = validateRows([
    row("walk", "  chien "),
    emptyRow(),
    row("strength"),
    row("", "  "),
  ]);
  assert.equal(result.ok, true);
  assert.deepEqual(result.drafts, [
    { kind: "walk", note: "chien" },
    { kind: "strength", note: "" },
  ]);
});

test("AC4: Esc closes the picker first, then the form without saving", () => {
  assert.equal(formKeyAction({ key: "Escape" }, true), "closePicker");
  assert.equal(formKeyAction({ key: "Escape" }, false), "close");
});

test("AC4: only Cmd+Enter saves from the keyboard", () => {
  assert.equal(formKeyAction({ key: "Enter", metaKey: true }, false), "save");
  assert.equal(formKeyAction({ key: "Enter" }, false), null);
  assert.equal(formKeyAction({ key: "a" }, false), null);
});
