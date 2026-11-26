// Auto-hide timer of a toast: 8 s by default, paused while the pointer is over it.

export const TOAST_DELAY_MS = 8000;

/**
 * `start` shows for a full delay again. `pause` stops the countdown and keeps
 * what is left, `resume` continues from there, `cancel` stops it for good.
 * `hide` runs once the delay has run out. The clock functions are injectable.
 */
export function createAutoHide({
  hide,
  delayMs = TOAST_DELAY_MS,
  setTimer = setTimeout,
  clearTimer = clearTimeout,
  now = Date.now,
}) {
  let timer = null;
  let startedAt = 0;
  let remaining = delayMs;
  let paused = false;

  function arm() {
    startedAt = now();
    timer = setTimer(() => {
      timer = null;
      hide();
    }, remaining);
  }

  function stop() {
    if (timer === null) return;
    clearTimer(timer);
    timer = null;
  }

  return {
    start() {
      stop();
      paused = false;
      remaining = delayMs;
      arm();
    },
    pause() {
      if (timer === null) return;
      remaining = Math.max(0, remaining - (now() - startedAt));
      paused = true;
      stop();
    },
    resume() {
      if (!paused) return;
      paused = false;
      arm();
    },
    cancel() {
      stop();
      paused = false;
      remaining = delayMs;
    },
  };
}
