import assert from "node:assert/strict";
import test from "node:test";
import { macroBars, renderMacroChart } from "./macro_chart.js";

const WEEK = ["lun. 2", "mar. 3", "mer. 4", "jeu. 5", "ven. 6", "sam. 7", "dim. 8"];

function week(values) {
  return WEEK.map((label, index) => ({
    date: `2026-11-0${index + 2}`,
    label,
    tenthG: values[index] ?? null,
    text: values[index] == null ? null : `${values[index] / 10} g`.replace(".", ","),
  }));
}

function element(name) {
  return {
    name,
    attrs: {},
    children: [],
    className: "",
    textContent: "",
    setAttribute(key, value) {
      this.attrs[key] = String(value);
    },
    append(...nodes) {
      this.children.push(...nodes);
    },
  };
}

function host() {
  return {
    children: [],
    replaceChildren() {
      this.children = [];
    },
    append(...nodes) {
      this.children.push(...nodes);
    },
  };
}

function withDocument(run) {
  globalThis.document = {
    createElementNS: (_namespace, name) => element(name),
    createElement: (name) => element(name),
  };
  try {
    return run();
  } finally {
    delete globalThis.document;
  }
}

function all(node, found = []) {
  found.push(node);
  for (const child of node.children ?? []) all(child, found);
  return found;
}

function withClass(root, className) {
  return all(root).filter((node) => (node.attrs?.class ?? "").split(/\s+/).includes(className));
}

test("M9: no intake in the period is an empty state, not bars at zero", () => {
  const geometry = macroBars(week([]), null);
  assert.equal(geometry.empty, true);
  assert.deepEqual(geometry.bars, []);
  assert.equal(geometry.mean, null);
});

test("M8: a day without intake is a hole: no bar for it", () => {
  const geometry = macroBars(week([1200, null, 1500]), 135);
  assert.equal(geometry.empty, false);
  assert.equal(geometry.bars.length, 2);
  assert.deepEqual(
    geometry.bars.map((bar) => bar.label),
    ["lun. 2", "mer. 4"],
  );
});

test("M8: a day entered at 0 g is a visible stub with its text, unlike a hole", () => {
  const geometry = macroBars(week([0, null, 1200]), 60);
  const zero = geometry.bars.find((bar) => bar.label === "lun. 2");
  assert.ok(zero, "the 0 g day has a bar");
  assert.ok(zero.height > 0, "the stub has a height");
  assert.equal(zero.text, "0 g");
  assert.equal(
    geometry.bars.some((bar) => bar.label === "mar. 3"),
    false,
  );
});

test("M7: bar heights are proportional to the values", () => {
  const geometry = macroBars(week([1000, 2000]), 150);
  const [low, high] = geometry.bars;
  assert.ok(Math.abs(high.height / low.height - 2) < 1e-9);
  assert.equal(low.y + low.height, high.y + high.height, "bars share a baseline");
});

test("M7: bars run from the oldest day on the left to the newest on the right", () => {
  const geometry = macroBars(week([1000, 1000, 1000, 1000, 1000, 1000, 1000]), 100);
  const xs = geometry.bars.map((bar) => bar.x);
  assert.deepEqual(xs, [...xs].sort((a, b) => a - b));
  assert.equal(new Set(xs).size, 7);
});

test("M9: the mean line sits at the height of the displayed mean", () => {
  const geometry = macroBars(week([1420, 800]), 142);
  const tallest = geometry.bars[0];
  assert.ok(Math.abs(geometry.mean.y - tallest.y) < 1e-9);
});

test("M9: a mean above every bar stays inside the chart", () => {
  const geometry = macroBars(week([100, 100]), 300);
  assert.ok(geometry.mean.y >= 0);
  for (const bar of geometry.bars) assert.ok(bar.y > geometry.mean.y);
});

test("M10: each bar carries its value in grams as a tooltip", () => {
  const box = host();
  withDocument(() => renderMacroChart(box, week([1425, null, 1200]), 132));
  const bars = box.children.flatMap((root) => withClass(root, "mc-bar"));
  assert.equal(bars.length, 2);
  const titles = bars.map((bar) => bar.children.find((child) => child.name === "title").textContent);
  assert.deepEqual(titles, ["lun. 2 · 142,5 g", "mer. 4 · 120 g"]);
});

test("M9: the chart draws the mean line", () => {
  const box = host();
  withDocument(() => renderMacroChart(box, week([1000, 2000]), 150));
  const lines = box.children.flatMap((root) => withClass(root, "mc-mean"));
  assert.equal(lines.length, 1);
});

test("M11: an empty period renders a sentence and no svg", () => {
  const box = host();
  withDocument(() => renderMacroChart(box, week([]), null));
  assert.equal(box.children.length, 1);
  assert.equal(box.children[0].name, "p");
  assert.equal(box.children[0].textContent, "Aucune saisie sur ces 7 jours.");
});

test("A6: no red in the chart", () => {
  const box = host();
  withDocument(() => renderMacroChart(box, week([0, 1200, null, 1500]), 900));
  const nodes = box.children.flatMap((root) => all(root));
  for (const node of nodes) {
    for (const value of Object.values(node.attrs ?? {})) {
      assert.equal(/red|--bad|#b3261e|#ff8a80/i.test(value), false, value);
    }
  }
});
