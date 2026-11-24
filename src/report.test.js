import assert from "node:assert/strict";
import test from "node:test";
import { trendRuns } from "./chart.js";
import {
  REPORT_COLORS,
  REPORT_FOOTER,
  REPORT_SCALE,
  REPORT_WIDTH,
  copiedMessage,
  paintReport,
  reportClipboardItems,
  summaryRows,
  wrapText,
} from "./report.js";

test("the copied message names the fifteen-day range", () => {
  assert.equal(
    copiedMessage({ fromLabel: "mar. 10 nov.", toLabel: "mar. 24 nov." }),
    "Rapport copié · 15 jours, du mar. 10 nov. au mar. 24 nov.",
  );
});

test("the clipboard item keeps the pending png promise", async () => {
  let resolveBlob;
  const pngBlobPromise = new Promise((resolve) => {
    resolveBlob = resolve;
  });
  globalThis.ClipboardItem = class ClipboardItem {
    constructor(items) {
      this.items = items;
    }
  };
  const [item] = reportClipboardItems(pngBlobPromise);
  assert.equal(item.items["image/png"], pngBlobPromise);
  let settled = false;
  pngBlobPromise.then(() => {
    settled = true;
  });
  await Promise.resolve();
  assert.equal(settled, false);
  resolveBlob(new Blob(["png"]));
  await pngBlobPromise;
  assert.equal(settled, true);
});

test("a missing trend splits the line and is not a point of either run", () => {
  const days = [
    { trendTenthKg: 800 },
    { trendTenthKg: 800 },
    { trendTenthKg: null },
    { trendTenthKg: 786 },
  ];
  const runs = trendRuns(days);
  assert.deepEqual(runs, [[0, 1], [3]]);
  assert.equal(runs.some((run) => run.includes(2)), false);
});

test("the trend box shows both ends and the gap, and is absent without a trend", () => {
  const withTrend = paintedTexts([day("2026-11-24", { weight: "78,6 kg" })], {
    trend: { startTenthKg: 800, endTenthKg: 786, variationTenthKg: -14 },
    points: [
      { label: "10 nov.", weighTenthKg: 800, trendTenthKg: 800 },
      { label: "16 nov.", weighTenthKg: null, trendTenthKg: null },
      { label: "24 nov.", weighTenthKg: 786, trendTenthKg: 786 },
    ],
  });
  assert.ok(withTrend.includes("Tendance sur 15 jours"));
  assert.ok(withTrend.includes("80,0 → 78,6 kg"));
  assert.ok(withTrend.includes("\u22121,4 kg"));
  const without = paintedTexts([day("2026-11-24", {})], { trend: null, points: null });
  assert.equal(without.includes("Tendance sur 15 jours"), false);
});

test("an entered zero is drawn and a missing day stays blank, with no alert colour", () => {
  const canvas = mockCanvas();
  paintReport(canvas, {
    fromLabel: "mar. 10 nov.",
    toLabel: "mar. 24 nov.",
    days: [
      day("2026-11-12", {
        label: "jeu. 12 nov.",
        energyKcal: 0,
        proteinG: "0",
        carbsG: "0",
        fatG: "0",
        water: "0",
      }),
      day("2026-11-13", { label: "ven. 13 nov." }),
      day("2026-11-24", { label: "mar. 24 nov.", weight: "82,0 kg" }),
    ],
  });
  const texts = canvas.calls.filter((call) => call[0] === "fillText").map((call) => call[1]);
  const fills = canvas.calls.filter((call) => call[0] === "fillStyle").map((call) => call[1]);
  assert.ok(texts.includes("0 kcal"));
  assert.ok(texts.includes("P 0 · G 0 · L 0"));
  assert.ok(texts.includes("0 L"));
  assert.ok(texts.includes("ven. 13 nov."));
  assert.equal(
    texts.filter((text) => text.includes("kcal")).length,
    1,
  );
  assert.ok(texts.includes(REPORT_FOOTER));
  assert.equal(texts.some((text) => text.includes("Alerte") || text.includes("⚠")), false);
  const allowed = new Set(Object.values(REPORT_COLORS));
  for (const fill of fills) assert.ok(allowed.has(fill), fill);
});

test("a card shows pas à jeun and leaves a missing intake blank", () => {
  const texts = paintedTexts([
    day("2026-11-11", { weight: "81,0 kg", notFasted: true }),
    day("2026-11-12", {}),
  ]);
  assert.ok(texts.includes("81,0 kg · pas à jeun"));
  assert.ok(texts.includes("jeu. 12 nov."));
  assert.equal(
    texts.some((text) => text.includes("kcal")),
    false,
  );
});

const SUMMARY = {
  intakeDays: 7,
  energyKcal: 2310,
  proteinG: 108,
  carbsG: 192,
  fatG: 102,
  waterDays: 6,
  water: "3,16",
  activityDays: 4,
};

const LONG_NOTE = "une très longue note de séance ".repeat(6).trim();

test("the canvas is a 480 px portrait column drawn at double density", () => {
  const canvas = mockCanvas();
  paintReport(canvas, { fromLabel: "a", toLabel: "b", days: [day("2026-11-12", {})] });
  assert.equal(REPORT_WIDTH, 480);
  assert.equal(REPORT_SCALE, 2);
  assert.equal(canvas.width, 960);
  assert.ok(canvas.height > 0);
  assert.ok(canvas.calls.some((call) => call[0] === "scale" && call[1] === 2 && call[2] === 2));
});

test("every font of the image is at least 15 px", () => {
  const canvas = mockCanvas();
  paintReport(canvas, {
    fromLabel: "a",
    toLabel: "b",
    days: [
      day("2026-11-12", {
        weight: "81,0 kg",
        energyKcal: 1,
        proteinG: "1",
        carbsG: "1",
        fatG: "1",
        water: "1",
      }),
    ],
    trend: { startTenthKg: 800, endTenthKg: 786, variationTenthKg: -14 },
    points: [{ label: "10 nov.", weighTenthKg: 800, trendTenthKg: 800 }],
    summary: SUMMARY,
  });
  const sizes = canvas.calls
    .filter((call) => call[0] === "font")
    .map((call) => Number(/(\d+)px/.exec(call[1])[1]));
  assert.ok(sizes.length > 0);
  assert.ok(sizes.every((size) => size >= 15), sizes.join(","));
});

test("the summary shows each mean with its unit and the counted days", () => {
  assert.deepEqual(summaryRows(SUMMARY), [
    { label: "Énergie", note: "7 jours saisis", value: "2 310 kcal" },
    { label: "Protéines", note: "7 jours saisis", value: "108 g" },
    { label: "Glucides", note: "7 jours saisis", value: "192 g" },
    { label: "Lipides", note: "7 jours saisis", value: "102 g" },
    { label: "Eau", note: "6 jours saisis", value: "3,16 L" },
    { label: "Activité", note: "", value: "4 jours" },
  ]);
});

test("a summary without any intake shows a dash, never a zero", () => {
  const rows = summaryRows({
    intakeDays: 0,
    energyKcal: null,
    proteinG: null,
    carbsG: null,
    fatG: null,
    waterDays: 0,
    water: null,
    activityDays: 0,
  });
  assert.deepEqual(
    rows.map((row) => row.value),
    ["—", "—", "—", "—", "—", "0 jour"],
  );
  assert.equal(rows[0].note, "aucun jour saisi");
});

test("one counted day is written in the singular", () => {
  const rows = summaryRows({ ...SUMMARY, intakeDays: 1, waterDays: 1, activityDays: 1 });
  assert.equal(rows[0].note, "1 jour saisi");
  assert.equal(rows[4].note, "1 jour saisi");
  assert.equal(rows[5].value, "1 jour");
});

test("the summary is painted in the image", () => {
  const texts = paintedTexts([day("2026-11-12", {})], { summary: SUMMARY });
  assert.ok(texts.includes("Moyenne par jour"));
  assert.ok(texts.includes("2 310 kcal"));
  assert.ok(texts.includes("3,16 L"));
});

function curveTexts(weighTenths) {
  return paintedTexts([day("2026-11-12", {})], {
    points: weighTenths.map((tenth, index) => ({
      label: `${index + 1} nov.`,
      weighTenthKg: tenth,
      trendTenthKg: null,
    })),
  });
}

test("the curve has a kg scale with a top, a middle and a bottom mark", () => {
  const texts = curveTexts([1086, 1092]);
  assert.ok(texts.includes("110 kg"));
  assert.ok(texts.includes("109 kg"));
  assert.ok(texts.includes("108 kg"));
});

test("a scale whose middle is a half uses a decimal comma", () => {
  const texts = curveTexts([1086, 1102]);
  assert.ok(texts.includes("111 kg"));
  assert.ok(texts.includes("109,5 kg"));
  assert.ok(texts.includes("108 kg"));
});

test("a curve with no weight at all shows no kg scale", () => {
  const texts = curveTexts([null, null]);
  assert.equal(texts.some((text) => /^\d+(,\d)? kg$/.test(text)), false);
});

test("the first box starts below the date range line", () => {
  const canvas = mockCanvas();
  paintReport(canvas, {
    fromLabel: "mar. 10 nov.",
    toLabel: "mar. 24 nov.",
    days: [day("2026-11-12", {})],
    trend: { startTenthKg: 800, endTenthKg: 786, variationTenthKg: -14 },
  });
  const rangeLine = canvas.calls.find((call) => call[0] === "fillText" && call[1].startsWith("du "));
  const firstBox = canvas.calls.find((call) => call[0] === "moveTo");
  const textHeight = 15;
  assert.ok(firstBox[2] >= rangeLine[3] + textHeight, `${firstBox[2]} vs ${rangeLine[3]}`);
});

test("a long note wraps instead of running out of the card", () => {
  const ctx = mockCanvas().getContext();
  const lines = wrapText(ctx, LONG_NOTE, 200);
  assert.ok(lines.length > 1);
  assert.ok(lines.every((line) => ctx.measureText(line).width <= 200), lines.join("|"));
});

test("a card with a long note is taller than the same card with a short one", () => {
  const short = mockCanvas();
  const long = mockCanvas();
  const base = { fromLabel: "a", toLabel: "b" };
  paintReport(short, { ...base, days: [day("2026-11-12", { activities: [{ kind: "walk", note: "ok" }] })] });
  paintReport(long, { ...base, days: [day("2026-11-12", { activities: [{ kind: "walk", note: LONG_NOTE }] })] });
  assert.ok(long.height > short.height);
});

function day(date, fields) {
  return {
    date,
    label: date === "2026-11-11" ? "mer. 11 nov." : "jeu. 12 nov.",
    weight: null,
    notFasted: false,
    energyKcal: null,
    proteinG: null,
    carbsG: null,
    fatG: null,
    water: null,
    activities: [],
    ...fields,
  };
}

function paintedTexts(days, extra = {}) {
  const canvas = mockCanvas();
  paintReport(canvas, {
    fromLabel: "mar. 10 nov.",
    toLabel: "mar. 24 nov.",
    days,
    ...extra,
  });
  return canvas.calls.filter((call) => call[0] === "fillText").map((call) => call[1]);
}

function mockCanvas() {
  const calls = [];
  const ctx = new Proxy(
    {},
    {
      get(_target, prop) {
        if (prop === "measureText") return (text) => ({ width: String(text).length * 7 });
        return (...args) => {
          calls.push([prop, ...args]);
        };
      },
      set(target, prop, value) {
        target[prop] = value;
        if (prop === "fillStyle" || prop === "strokeStyle" || prop === "font") calls.push([prop, value]);
        return true;
      },
    },
  );
  return {
    calls,
    width: 0,
    height: 0,
    getContext() {
      return ctx;
    },
  };
}

export { mockCanvas, paintedTexts };
