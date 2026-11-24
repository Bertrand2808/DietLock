import assert from "node:assert/strict";
import test from "node:test";
import { trendRuns } from "./chart.js";
import { copiedMessage, paintReport, reportClipboardItems } from "./report.js";

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
        if (prop === "fillStyle" || prop === "strokeStyle") calls.push([prop, value]);
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
