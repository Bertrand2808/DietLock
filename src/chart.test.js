import assert from "node:assert/strict";
import test from "node:test";
import { renderChart } from "./chart.js";

function element(name) {
  return {
    name,
    attrs: {},
    children: [],
    textContent: "",
    setAttribute(key, value) {
      this.attrs[key] = String(value);
    },
    append(...nodes) {
      this.children.push(...nodes);
    },
    get lastChild() {
      return this.children[this.children.length - 1] ?? null;
    },
  };
}

function host() {
  return {
    children: [],
    replaceChildren() {
      this.children = [];
    },
    append(node) {
      this.children.push(node);
    },
  };
}

function withClass(node, className, found) {
  if (node.attrs?.class?.split(/\s+/).includes(className)) found.push(node);
  for (const child of node.children ?? []) withClass(child, className, found);
}

// Four fasted weigh-ins on the four days ending today. The 7-day mean
// exists only on that last day, so the chart has a single trend day.
function fourFastedDaysEndingToday() {
  const dates = [
    "2026-10-20",
    "2026-10-21",
    "2026-10-22",
    "2026-10-23",
    "2026-10-24",
    "2026-10-25",
    "2026-10-26",
    "2026-10-27",
    "2026-10-28",
    "2026-10-29",
    "2026-10-30",
    "2026-10-31",
    "2026-11-01",
    "2026-11-02",
    "2026-11-03",
  ];
  const weights = {
    "2026-10-31": 780,
    "2026-11-01": 790,
    "2026-11-02": 800,
    "2026-11-03": 784,
  };
  return dates.map((date) => ({
    date,
    label: "",
    weighTenthKg: weights[date] ?? null,
    trendTenthKg: date === "2026-11-03" ? 789 : null,
  }));
}

test("four fasted weigh-ins ending today draw an orange trend point", () => {
  globalThis.document = {
    createElementNS(_namespace, name) {
      return element(name);
    },
  };
  const box = host();
  renderChart(box, fourFastedDaysEndingToday(), null, 15);
  const ends = [];
  for (const child of box.children) withClass(child, "wc-end", ends);
  assert.ok(ends.length >= 1);
});
