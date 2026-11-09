import assert from "node:assert/strict";
import test from "node:test";
import { deleteFailureText, deleteQuestion, deleteThenReload } from "./delete.js";

test("S10: a weigh-in question names the date and the weight", () => {
  const text = deleteQuestion("lun. 26 oct.", {
    kind: "weighIn",
    date: "2026-10-26",
    weight: "82,6",
    fasted: true,
  });
  assert.equal(text, "Supprimer la pesée du lun. 26 oct. : 82,6 kg ?");
});

test("S10: an intake question lists macros and water together", () => {
  const text = deleteQuestion("lun. 26 oct.", {
    kind: "intake",
    date: "2026-10-26",
    proteinG: "150",
    carbsG: "200",
    fatG: "70",
    water: "2",
    energyKcal: 1860,
  });
  assert.equal(
    text,
    "Supprimer l'alimentation du lun. 26 oct. : 1\u202f860 kcal (P 150 g, G 200 g, L 70 g, eau 2 L) ?",
  );
});

test("S10: an intake without water does not mention water, and a zero stays a zero", () => {
  const text = deleteQuestion("mar. 27 oct.", {
    kind: "intake",
    date: "2026-10-27",
    proteinG: "0",
    carbsG: "100",
    fatG: "0",
    water: null,
    energyKcal: 400,
  });
  assert.equal(
    text,
    "Supprimer l'alimentation du mar. 27 oct. : 400 kcal (P 0 g, G 100 g, L 0 g) ?",
  );
});

test("S10: a failed deletion reports the error and skips the reload", async () => {
  const boom = new Error("boom");
  let reloaded = false;
  const result = await deleteThenReload(
    async () => {
      throw boom;
    },
    async () => {
      reloaded = true;
    },
  );
  assert.deepEqual(result, { deleted: false, error: boom });
  assert.equal(reloaded, false);
});

test("S10: a reload failure after a successful deletion says the deletion happened", async () => {
  const boom = new Error("boom");
  const result = await deleteThenReload(
    async () => {},
    async () => {
      throw boom;
    },
  );
  assert.deepEqual(result, { deleted: true, error: boom });
});

test("S10: deletion and reload both succeeding leaves no error", async () => {
  const result = await deleteThenReload(async () => {}, async () => {});
  assert.deepEqual(result, { deleted: true, error: null });
});

test("S10: the failure text tells a reload failure apart from a deletion failure", () => {
  assert.equal(deleteFailureText(false, "disque plein"), "disque plein");
  assert.equal(
    deleteFailureText(true, "disque plein"),
    "Supprimé, mais la liste n'a pas pu être rechargée : disque plein",
  );
});
