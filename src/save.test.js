import assert from "node:assert/strict";
import test from "node:test";
import { blockStatus, intakeConfirmText, saveBlocks } from "./save.js";

const filled = {
  weight: "78,2",
  protein: "165",
  carbs: "180",
  fat: "65",
  water: "1,255",
};

test("a refused intake still saves the weigh-in", async () => {
  const out = await saveBlocks(
    filled,
    async () => "saved",
    async () => "error",
  );
  assert.deepEqual(out, { weighIn: "saved", intake: "error" });
});

test("a refused weigh-in does not skip the intake", async () => {
  let intakeCalled = false;
  const out = await saveBlocks(
    { ...filled, weight: "abc", water: "2,15" },
    async () => "error",
    async () => {
      intakeCalled = true;
      return "saved";
    },
  );
  assert.equal(intakeCalled, true);
  assert.deepEqual(out, { weighIn: "error", intake: "saved" });
});

test("each already filled block asks for confirmation on its own", async () => {
  const out = await saveBlocks(
    filled,
    async () => "confirm",
    async () => "confirm",
  );
  assert.deepEqual(out, { weighIn: "confirm", intake: "confirm" });
});

test("an empty block is not saved", async () => {
  let calls = 0;
  const bump = async () => {
    calls += 1;
    return "saved";
  };
  const out = await saveBlocks(
    { weight: "  ", protein: "", carbs: "", fat: "", water: "" },
    bump,
    bump,
  );
  assert.equal(calls, 0);
  assert.deepEqual(out, { weighIn: "skipped", intake: "skipped" });
});

test("an unchanged block is skipped", () => {
  assert.equal(blockStatus("unchanged"), "skipped");
});

test("a filled block still asks and a write stays saved", () => {
  assert.equal(blockStatus("alreadyFilled"), "confirm");
  assert.equal(blockStatus("saved"), "saved");
});

test("overwriting water alone asks about the water", () => {
  const text = intakeConfirmText({
    label: "mar. 3 nov.",
    proteinText: "",
    carbsText: "",
    fatText: "",
    waterText: "2",
    old: {
      energy: "2\u202f190",
      proteinG: "146",
      carbsG: "190",
      fatG: "94",
      water: "3,1",
    },
  });
  assert.equal(text, "Le mar. 3 nov. a déjà une eau : 3,1 L. La remplacer par 2 L ?");
});

test("replacing macros names the stored intake and the typed one", () => {
  const text = intakeConfirmText({
    label: "mar. 3 nov.",
    proteinText: "150",
    carbsText: "200",
    fatText: "60",
    waterText: "1,5",
    old: {
      energy: "2\u202f190",
      proteinG: "146",
      carbsG: "190",
      fatG: "94",
      water: "3,1",
    },
  });
  assert.equal(
    text,
    "Le mar. 3 nov. a déjà une alimentation : 2\u202f190 kcal " +
      "(P 146 g, G 190 g, L 94 g, eau 3,1 L). " +
      "La remplacer par P 150 g, G 200 g, L 60 g, eau 1,5 L ?",
  );
});

test("water alone still attempts the intake block", async () => {
  const out = await saveBlocks(
    { weight: "", protein: "", carbs: "", fat: "", water: "1" },
    async () => "saved",
    async () => "error",
  );
  assert.deepEqual(out, { weighIn: "skipped", intake: "error" });
});
