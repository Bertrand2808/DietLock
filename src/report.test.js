import assert from "node:assert/strict";
import test from "node:test";
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

function paintedTexts(days) {
  const canvas = mockCanvas();
  paintReport(canvas, {
    fromLabel: "mar. 10 nov.",
    toLabel: "mar. 24 nov.",
    days,
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
