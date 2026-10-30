import assert from "node:assert/strict";
import test from "node:test";
import {
  WEEKDAY_INITIALS,
  addDays,
  canSelect,
  chipView,
  filledDates,
  frenchDate,
  futureDateMessage,
  monthGrid,
  monthTitle,
  shiftMonth,
  shortcutState,
} from "./calendar.js";

const TODAY = "2026-10-30";

test("the grid starts on Monday", () => {
  assert.deepEqual(WEEKDAY_INITIALS, ["L", "M", "M", "J", "V", "S", "D"]);
});

test("October 2026 starts on Thursday and keeps the 31st", () => {
  const cells = monthGrid(2026, 10);
  assert.equal(cells.length % 7, 0);
  assert.deepEqual(cells.slice(0, 4), [null, null, null, "2026-10-01"]);
  assert.equal(cells[32], "2026-10-30");
  assert.equal(cells[33], "2026-10-31");
  assert.equal(cells.filter(Boolean).length, 31);
});

test("a month that starts on Sunday leads with six empty cells", () => {
  const cells = monthGrid(2026, 11);
  assert.equal(cells[6], "2026-11-01");
  assert.equal(cells.slice(0, 6).every((cell) => cell === null), true);
});

test("February 2028 keeps the leap day", () => {
  const cells = monthGrid(2028, 2).filter(Boolean);
  assert.equal(cells[0], "2028-02-01");
  assert.equal(cells.at(-1), "2028-02-29");
  assert.equal(cells.length, 29);
});

test("month arrows cross the year and stop at the calendar bounds", () => {
  assert.deepEqual(shiftMonth(2026, 10, 1), { year: 2026, month: 11 });
  assert.deepEqual(shiftMonth(2026, 12, 1), { year: 2027, month: 1 });
  assert.deepEqual(shiftMonth(2026, 1, -1), { year: 2025, month: 12 });
  assert.deepEqual(shiftMonth(1000, 1, -1), { year: 1000, month: 1 });
  assert.deepEqual(shiftMonth(9999, 12, 1), { year: 9999, month: 12 });
});

test("day shifts cross month ends and the leap day", () => {
  assert.equal(addDays(TODAY, -2), "2026-10-28");
  assert.equal(addDays("2026-03-01", -1), "2026-02-28");
  assert.equal(addDays("2028-02-28", 1), "2028-02-29");
  assert.equal(addDays("2028-02-29", 1), "2028-03-01");
});

test("french dates match the history labels", () => {
  assert.equal(frenchDate("2026-10-13"), "mar. 13 oct.");
  assert.equal(frenchDate("2026-10-25"), "dim. 25 oct.");
  assert.equal(frenchDate("2026-10-26"), "lun. 26 oct.");
  assert.equal(frenchDate(TODAY), "ven. 30 oct.");
  assert.equal(frenchDate("2026-02-02"), "lun. 2 févr.");
  assert.equal(frenchDate("2026-08-01"), "sam. 1 août");
  assert.equal(monthTitle(2026, 10), "octobre 2026");
  assert.equal(monthTitle(2026, 2), "février 2026");
  assert.equal(monthTitle(2026, 8), "août 2026");
});

test("a future day cannot be selected", () => {
  assert.equal(canSelect("2026-10-31", TODAY), false);
  assert.equal(canSelect(TODAY, TODAY), true);
  assert.equal(canSelect("2026-10-28", TODAY), true);
  assert.equal(frenchDate("2026-10-31"), "sam. 31 oct.");
  assert.equal(
    futureDateMessage("2026-10-31"),
    "sam. 31 oct. est dans le futur, choisis aujourd'hui ou avant.",
  );
});

test("an untouched forgotten yesterday keeps its mark", () => {
  const view = chipView({
    date: "2026-10-29",
    proposed: "2026-10-29",
    today: TODAY,
    forgottenProposed: true,
  });
  assert.equal(view.strong, "Hier");
  assert.equal(view.rest, " · jeu. 29 oct.");
  assert.equal(view.moved, false);
  assert.equal(view.forgotten, true);
});

test("today stays Aujourd'hui when nothing was touched", () => {
  const view = chipView({
    date: TODAY,
    proposed: TODAY,
    today: TODAY,
    forgottenProposed: false,
  });
  assert.equal(view.strong, "Aujourd'hui");
  assert.equal(view.rest, " · ven. 30 oct.");
  assert.equal(view.moved, false);
  assert.equal(view.forgotten, false);
});

test("another day drops the forgotten mark and shows the short date", () => {
  const view = chipView({
    date: "2026-10-28",
    proposed: "2026-10-29",
    today: TODAY,
    forgottenProposed: true,
  });
  assert.equal(view.strong, "mer. 28 oct.");
  assert.equal(view.rest, "");
  assert.equal(view.moved, true);
  assert.equal(view.forgotten, false);
});

test("coming back to the proposed day restores the mark", () => {
  const view = chipView({
    date: "2026-10-29",
    proposed: "2026-10-29",
    today: TODAY,
    forgottenProposed: true,
  });
  assert.equal(view.forgotten, true);
  assert.equal(view.moved, false);
});

test("shortcuts follow the selected day", () => {
  assert.deepEqual(shortcutState(TODAY, TODAY), { today: true, yesterday: false });
  assert.deepEqual(shortcutState("2026-10-29", TODAY), { today: false, yesterday: true });
  assert.deepEqual(shortcutState("2026-10-28", TODAY), { today: false, yesterday: false });
});

test("filled days are kept per entry kind", () => {
  const days = [
    { date: "2026-10-30", weighIn: { weight: "78,2" }, intake: null },
    { date: "2026-10-28", weighIn: null, intake: { energyKcal: 1850 } },
    { date: "2026-10-27", weighIn: null, intake: null },
  ];
  assert.deepEqual(filledDates(days, "weigh"), ["2026-10-30"]);
  assert.deepEqual(filledDates(days, "intake"), ["2026-10-28"]);
});
