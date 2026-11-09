/**
 * Pure logic of the activity form (AC1–AC4). Keys match `ActivityKind::key`
 * in dietlock-core. The core validates again when the form is saved.
 */
import { savesOnEnter } from "./keys.js";

export const ACTIVITY_TYPES = [
  { key: "walk", label: "Marche" },
  { key: "run", label: "Course" },
  { key: "mobility", label: "Mobilité" },
  { key: "strength", label: "Muscu" },
  { key: "bike", label: "Vélo" },
  { key: "other", label: "Autre" },
];

let nextId = 1;

/** A form row: no type yet, no note. Ids tell rows apart when one is removed. */
export function emptyRow() {
  const id = nextId;
  nextId += 1;
  return { id, type: "", note: "" };
}

export function rowIsEmpty(row) {
  return row.type === "" && row.note.trim() === "";
}

export function addRow(rows) {
  return [...rows, emptyRow()];
}

/** The form always keeps one row: removing the last one leaves an empty row. */
export function removeRow(rows, id) {
  const kept = rows.filter((row) => row.id !== id);
  return kept.length === 0 ? [emptyRow()] : kept;
}

/** Rows that have a type: the activities the button would save. */
export function saveCount(rows) {
  return rows.filter((row) => row.type !== "").length;
}

export function saveButtonText(count) {
  if (count === 0) return "Enregistrer";
  return count === 1 ? "Enregistrer 1 activité" : `Enregistrer ${count} activités`;
}

/**
 * AC4: a form needs at least one type, and a note never stands without one.
 * Fully empty rows are ignored. Same wording as `view::validation_message`.
 */
export function validateRows(rows) {
  const filled = rows.filter((row) => !rowIsEmpty(row));
  if (filled.some((row) => row.type === "")) {
    return { ok: false, error: "Choisis un type pour chaque activité." };
  }
  if (filled.length === 0) {
    return { ok: false, error: "Choisis au moins un type d'activité." };
  }
  return {
    ok: true,
    drafts: filled.map((row) => ({ kind: row.type, note: row.note.trim() })),
  };
}

/**
 * What a key press does while the form is open. Esc closes the date picker
 * first, then the form without saving. ⌘↩ saves.
 * Returns "closePicker", "close", "save" or null.
 */
export function formKeyAction(event, pickerOpen) {
  if (event.key === "Escape") return pickerOpen ? "closePicker" : "close";
  if (savesOnEnter(event)) return "save";
  return null;
}
