/**
 * Mini calendar for the entry card (D4–D6).
 * Weekday and month names match `view::french_date` in src-tauri.
 * Monday is the first column. Dates are `YYYY-MM-DD` and compared as text.
 */

const WEEKDAYS = ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."];
const MONTHS = [
  "janv.",
  "févr.",
  "mars",
  "avr.",
  "mai",
  "juin",
  "juil.",
  "août",
  "sept.",
  "oct.",
  "nov.",
  "déc.",
];
const MONTHS_FULL = [
  "janvier",
  "février",
  "mars",
  "avril",
  "mai",
  "juin",
  "juillet",
  "août",
  "septembre",
  "octobre",
  "novembre",
  "décembre",
];

/** Column headers, Monday first. */
export const WEEKDAY_INITIALS = ["L", "M", "M", "J", "V", "S", "D"];

const YEAR_MIN = 1000;
const YEAR_MAX = 9999;

function parts(iso) {
  const [year, month, day] = iso.split("-").map(Number);
  return { year, month, day };
}

function isoFromUtc(date) {
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, "0");
  const day = String(date.getUTCDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

/** Calendar day shifted by `delta` days. */
export function addDays(iso, delta) {
  const { year, month, day } = parts(iso);
  return isoFromUtc(new Date(Date.UTC(year, month - 1, day + delta)));
}

/** `true` when `iso` is today or earlier (D5). */
export function canSelect(iso, today) {
  return iso <= today;
}

/**
 * Sentence of `view::validation_message` for a future date.
 * The screen shows the text returned by the command; this is the same wording.
 */
export function futureDateMessage(iso) {
  return `La date ${iso} est dans le futur.`;
}

/** `ven. 30 oct.` */
export function frenchDate(iso) {
  const { year, month, day } = parts(iso);
  const weekday = new Date(Date.UTC(year, month - 1, day)).getUTCDay();
  return `${WEEKDAYS[weekday]} ${day} ${MONTHS[month - 1]}`;
}

/** `octobre 2026` */
export function monthTitle(year, month) {
  return `${MONTHS_FULL[month - 1]} ${year}`;
}

export function monthOf(iso) {
  const { year, month } = parts(iso);
  return { year, month };
}

/** Next or previous month, kept inside the years `Day::parse` accepts. */
export function shiftMonth(year, month, delta) {
  const index = year * 12 + (month - 1) + delta;
  const nextYear = Math.floor(index / 12);
  const nextMonth = index - nextYear * 12 + 1;
  if (nextYear < YEAR_MIN || nextYear > YEAR_MAX) return { year, month };
  return { year: nextYear, month: nextMonth };
}

/**
 * Cells of one month, Monday first. `null` is an empty cell.
 * The length is a multiple of 7.
 */
export function monthGrid(year, month) {
  const first = new Date(Date.UTC(year, month - 1, 1));
  const leading = (first.getUTCDay() + 6) % 7;
  const count = new Date(Date.UTC(year, month, 0)).getUTCDate();
  const cells = [];
  for (let i = 0; i < leading; i += 1) cells.push(null);
  for (let day = 1; day <= count; day += 1) {
    cells.push(isoFromUtc(new Date(Date.UTC(year, month - 1, day))));
  }
  while (cells.length % 7 !== 0) cells.push(null);
  return cells;
}

/**
 * Chip text for a chosen day.
 * Today and yesterday keep the relative name. Any other day is the short date.
 * The forgotten-yesterday mark stays only while the proposed day is untouched.
 */
export function chipView({ date, proposed, today, forgottenProposed }) {
  const moved = date !== proposed;
  const label = frenchDate(date);
  let strong = label;
  let rest = "";
  if (date === today) {
    strong = "Aujourd'hui";
    rest = ` · ${label}`;
  } else if (date === addDays(today, -1)) {
    strong = "Hier";
    rest = ` · ${label}`;
  }
  return {
    strong,
    rest,
    moved,
    forgotten: Boolean(forgottenProposed) && !moved,
  };
}

/** Which shortcut matches the selected day. */
export function shortcutState(selected, today) {
  return {
    today: selected === today,
    yesterday: selected === addDays(today, -1),
  };
}

/** ISO dates that already hold this kind of entry. `days` is the history payload. */
export function filledDates(days, kind) {
  const key = kind === "weigh" ? "weighIn" : "intake";
  return days.filter((day) => day[key]).map((day) => day.date);
}
