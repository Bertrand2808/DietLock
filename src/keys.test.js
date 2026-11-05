import assert from "node:assert/strict";
import test from "node:test";
import { savesOnEnter, screenShortcut } from "./keys.js";

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

const chord = {
  key: "1",
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  repeat: false,
};

test("cmd 1 and cmd 2 switch screens on mac", () => {
  assert.equal(screenShortcut({ ...chord, metaKey: true }, true), "today");
  assert.equal(screenShortcut({ ...chord, metaKey: true, key: "2" }, true), "weight");
  assert.equal(screenShortcut({ ...chord, metaKey: true, key: "3" }, true), null);
});

test("ctrl 1 and ctrl 2 switch screens off mac", () => {
  assert.equal(screenShortcut({ ...chord, ctrlKey: true }, false), "today");
  assert.equal(screenShortcut({ ...chord, ctrlKey: true, key: "2" }, false), "weight");
});

test("the other modifier does not switch screens", () => {
  assert.equal(screenShortcut({ ...chord, ctrlKey: true }, true), null);
  assert.equal(screenShortcut({ ...chord, metaKey: true, key: "2" }, false), null);
});

test("shift, alt or a repeated key does not switch screens", () => {
  assert.equal(screenShortcut({ ...chord, metaKey: true, shiftKey: true }, true), null);
  assert.equal(screenShortcut({ ...chord, metaKey: true, altKey: true }, true), null);
  assert.equal(screenShortcut({ ...chord, metaKey: true, repeat: true }, true), null);
});
