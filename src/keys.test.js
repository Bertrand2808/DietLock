import assert from "node:assert/strict";
import test from "node:test";
import { savesOnEnter } from "./keys.js";

const enter = {
  key: "Enter",
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
};

test("enter alone does not save", () => {
  assert.equal(savesOnEnter(enter), false);
});

test("cmd enter saves", () => {
  assert.equal(savesOnEnter({ ...enter, metaKey: true }), true);
});

test("ctrl enter does not save", () => {
  assert.equal(savesOnEnter({ ...enter, ctrlKey: true }), false);
});

test("cmd enter with shift or alt does not save", () => {
  assert.equal(savesOnEnter({ ...enter, metaKey: true, shiftKey: true }), false);
  assert.equal(savesOnEnter({ ...enter, metaKey: true, altKey: true }), false);
  assert.equal(savesOnEnter({ ...enter, metaKey: true, ctrlKey: true }), false);
});

test("tab does not save", () => {
  assert.equal(savesOnEnter({ ...enter, key: "Tab", metaKey: true }), false);
});
