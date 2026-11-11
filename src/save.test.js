import assert from "node:assert/strict";
import test from "node:test";
import {
  blockStatus,
  intakeConfirmText,
  nextWaterField,
  saveBlocks,
  weighInConfirmText,
} from "./save.js";

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

// S3, S9, T6: the confirmation names what changes.
const oldWeigh = { weight: "82,6", fasted: true };

test("s3 weight-only change keeps the kg confirmation", () => {
  const text = weighInConfirmText({
    label: "mar. 3 nov.",
    weightText: "83,1",
    fasted: true,
    old: oldWeigh,
  });
  assert.equal(text, "Le mar. 3 nov. a déjà une pesée : 82,6 kg. La remplacer par 83,1 kg ?");
});

test("s3 unchecking fasted at the same weight names the fasted status", () => {
  const text = weighInConfirmText({
    label: "mar. 3 nov.",
    weightText: "82,6",
    fasted: false,
    old: oldWeigh,
  });
  assert.equal(
    text,
    "Le mar. 3 nov. a déjà une pesée : 82,6 kg, à jeun. La remplacer par 82,6 kg, pas à jeun ?",
  );
});

test("s3 the typed weight is shown as typed beside the fasted status", () => {
  const text = weighInConfirmText({
    label: "mar. 3 nov.",
    weightText: "82.60",
    fasted: false,
    old: oldWeigh,
  });
  assert.equal(
    text,
    "Le mar. 3 nov. a déjà une pesée : 82,6 kg, à jeun. La remplacer par 82.60 kg, pas à jeun ?",
  );
});

test("s3 changing weight and fasted together names both", () => {
  const text = weighInConfirmText({
    label: "mar. 3 nov.",
    weightText: "83,1",
    fasted: false,
    old: oldWeigh,
  });
  assert.equal(
    text,
    "Le mar. 3 nov. a déjà une pesée : 82,6 kg, à jeun. La remplacer par 83,1 kg, pas à jeun ?",
  );
});

test("s3 checking fasted again says à jeun on the new value", () => {
  const text = weighInConfirmText({
    label: "mar. 3 nov.",
    weightText: "82,6",
    fasted: true,
    old: { weight: "82,6", fasted: false },
  });
  assert.equal(
    text,
    "Le mar. 3 nov. a déjà une pesée : 82,6 kg, pas à jeun. La remplacer par 82,6 kg, à jeun ?",
  );
});

const storedIntake = {
  energy: "2 190",
  proteinG: "146",
  carbsG: "190",
  fatG: "94",
  water: "1,5",
};

const typed = { label: "mar. 3 nov.", proteinText: "146", carbsText: "190", fatText: "94" };

test("S3: clearing the water on a filled day says the water goes to 0", () => {
  const text = intakeConfirmText({ ...typed, waterText: "  ", old: storedIntake });
  assert.equal(
    text,
    "Le mar. 3 nov. a déjà une alimentation : 2 190 kcal " +
      "(P 146 g, G 190 g, L 94 g, eau 1,5 L). " +
      "L'eau (1,5 L) passera à 0. La remplacer par P 146 g, G 190 g, L 94 g ?",
  );
});

test("S3: no water warning when the stored intake has no water", () => {
  const text = intakeConfirmText({
    ...typed,
    proteinText: "150",
    waterText: "",
    old: { ...storedIntake, water: null },
  });
  assert.ok(!text.includes("passera à 0"));
});

test("S3: no water warning when the water is kept or replaced", () => {
  for (const waterText of ["1,5", "2"]) {
    const text = intakeConfirmText({ ...typed, proteinText: "150", waterText, old: storedIntake });
    assert.ok(!text.includes("passera à 0"), waterText);
  }
});

test("S5: the water field follows the stored water until the user types in it", () => {
  assert.equal(nextWaterField({ dirty: false, current: "", stored: "1,5" }), "1,5");
  assert.equal(nextWaterField({ dirty: false, current: "2", stored: null }), "");
  assert.equal(nextWaterField({ dirty: true, current: "2", stored: "1,5" }), "2");
  assert.equal(nextWaterField({ dirty: true, current: "", stored: "1,5" }), "");
});
