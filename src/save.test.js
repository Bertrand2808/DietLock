import assert from "node:assert/strict";
import test from "node:test";
import { saveBlocks } from "./save.js";

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

test("water alone still attempts the intake block", async () => {
  const out = await saveBlocks(
    { weight: "", protein: "", carbs: "", fat: "", water: "1" },
    async () => "saved",
    async () => "error",
  );
  assert.deepEqual(out, { weighIn: "skipped", intake: "error" });
});
