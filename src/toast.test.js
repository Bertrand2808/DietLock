import assert from "node:assert/strict";
import test from "node:test";
import { TOAST_DELAY_MS, createAutoHide } from "./toast.js";

/** A clock the test moves by hand. Timers fire when `advance` reaches them. */
function fakeClock() {
  let time = 0;
  let nextId = 1;
  const timers = new Map();
  return {
    now: () => time,
    setTimer(callback, delay) {
      const id = nextId++;
      timers.set(id, { callback, at: time + delay });
      return id;
    },
    clearTimer(id) {
      timers.delete(id);
    },
    advance(ms) {
      time += ms;
      for (const [id, timer] of [...timers]) {
        if (timer.at > time) continue;
        timers.delete(id);
        timer.callback();
      }
    },
    pending: () => timers.size,
  };
}

function setup() {
  const clock = fakeClock();
  const hidden = [];
  const autoHide = createAutoHide({
    hide: () => hidden.push(clock.now()),
    setTimer: clock.setTimer,
    clearTimer: clock.clearTimer,
    now: clock.now,
  });
  return { clock, hidden, autoHide };
}

test("the toast stays eight seconds, then hides once", () => {
  const { clock, hidden, autoHide } = setup();
  autoHide.start();
  clock.advance(TOAST_DELAY_MS - 1);
  assert.deepEqual(hidden, []);
  clock.advance(1);
  assert.deepEqual(hidden, [TOAST_DELAY_MS]);
  clock.advance(TOAST_DELAY_MS);
  assert.equal(hidden.length, 1);
});

test("the delay is eight seconds", () => {
  assert.equal(TOAST_DELAY_MS, 8000);
});

test("starting again restarts the full delay", () => {
  const { clock, hidden, autoHide } = setup();
  autoHide.start();
  clock.advance(5000);
  autoHide.start();
  clock.advance(5000);
  assert.deepEqual(hidden, []);
  clock.advance(3000);
  assert.deepEqual(hidden, [13000]);
});

test("pausing keeps what is left and resuming continues from there", () => {
  const { clock, hidden, autoHide } = setup();
  autoHide.start();
  clock.advance(6000);
  autoHide.pause();
  clock.advance(60000);
  assert.deepEqual(hidden, [], "a paused toast never hides");
  autoHide.resume();
  clock.advance(1999);
  assert.deepEqual(hidden, []);
  clock.advance(1);
  assert.equal(hidden.length, 1);
});

test("pausing at once and resuming still hides the toast", () => {
  const { clock, hidden, autoHide } = setup();
  autoHide.start();
  autoHide.pause();
  autoHide.resume();
  clock.advance(TOAST_DELAY_MS);
  assert.equal(hidden.length, 1);
});

test("resuming without a pause does nothing", () => {
  const { clock, hidden, autoHide } = setup();
  autoHide.resume();
  clock.advance(TOAST_DELAY_MS * 2);
  assert.deepEqual(hidden, []);
  assert.equal(clock.pending(), 0);
});

test("cancelling stops the countdown", () => {
  const { clock, hidden, autoHide } = setup();
  autoHide.start();
  autoHide.cancel();
  clock.advance(TOAST_DELAY_MS * 2);
  assert.deepEqual(hidden, []);
  assert.equal(clock.pending(), 0);
});

test("a new start after a pause cancels the paused state", () => {
  const { clock, hidden, autoHide } = setup();
  autoHide.start();
  clock.advance(7000);
  autoHide.pause();
  autoHide.start();
  autoHide.resume();
  clock.advance(TOAST_DELAY_MS);
  assert.equal(hidden.length, 1);
  assert.equal(clock.pending(), 0);
});
