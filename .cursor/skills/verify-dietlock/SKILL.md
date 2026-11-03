---
name: verify-dietlock
description: "Drive the DietLock Tauri app on a Linux VM and prove a screen or a cargo test. Use when a change needs a screenshot or command output in the PR body, or when checking the weigh-in, intake, trend, or history screen."
---

# Verify DietLock

DietLock is a one-window desktop app (Tauri + Rust). The window is the whole UI: a weigh-in form, an intake form, today's trend, and the last 30 days. There is no router and no second page. The release install target is macOS; this skill proves the same shell on a Linux cloud VM.

The harness is `.cursor/skills/verify-dietlock/scripts/dietlock-verify`. Run it from anywhere. It finds the repo as the parent of `.cursor`.

## Proof in every pull request

Every pull request includes its proof in the PR body before it is ready for review:

- A change the user can see: the PNG this skill writes, plus the passing check output (`check-today.txt` or the `eval` / `text` / `db` output for the journey you drove).
- A change covered by Rust tests, with no UI surface: the `cargo test` output (the full suite, or `cargo test -p <crate> <test>` for the case you changed).

A pull request that contains neither is not ready. Review the proof as well as the diff. One of the two is enough; a UI change still needs the screenshot, because CI never launches the window.

## Launch

From the repo root, on a Linux VM that has the Tauri system libraries (`libwebkit2gtk-4.1`, GTK 3, libsoup 3) plus `Xvfb`, `dbus-launch`, `xwininfo`, `xwd`, `convert` (ImageMagick), and `tesseract` with `fra` and `eng`:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify launch
```

`launch` builds the debug binary with `cargo build -p dietlock` (the dev profile turns WebKit developer extras on; a release binary does not, so the inspector stays closed), starts a private Xvfb and a private D-Bus session, and runs `target/debug/dietlock` with a disposable `HOME` and `XDG_DATA_HOME`. The store is created empty and migrated to schema `user_version` 1. There is no seed data: an empty database is the usable state.

The shell crate is recompiled on every launch. Tauri's build script watches `src-tauri/capabilities`, and that directory is not in the repo, so Cargo marks `dietlock` dirty. Dependencies stay cached; the relink takes a few seconds. Do not add the directory to silence it.

Ready means the process is up, the window is mapped, and the trend paragraph has text. `launch` prints `ready trend: pas encore de tendance` on a fresh data dir, then these lines:

```text
run <run-id>
binary <repo>/target/debug/dietlock
display :<n>
inspector http://127.0.0.1:<port>/
data /tmp/dietlock-verify/<run-id>/data/com.bertrand2808.dietlock/dietlock.sqlite
evidence /tmp/dietlock-verify-evidence/<run-id>
ready
```

The run id is remembered in `/tmp/dietlock-verify/current`. Set `DIETLOCK_VERIFY_RUN` to drive a specific run. Launch refuses when that run's process is still alive: one instance per run, and a second launch would share neither the display nor the database of the first. Clean up first.

WebKit is started with `WEBKIT_DISABLE_COMPOSITING_MODE=1` and `WEBKIT_DISABLE_DMABUF_RENDERER=1` so the window paints under Xvfb. The inspector listens on `WEBKIT_INSPECTOR_HTTP_SERVER`. Do not point a browser at `src/index.html`: `src/main.js` calls `window.__TAURI__` and the forms do not save.

## Doctor

Run this before the first drive, and again after any failure, before driving further:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify doctor
```

Pass means all of the following: the recorded pid is the `dietlock` binary, its Xvfb is alive, the inspector page lists a target named DietLock at `tauri://localhost`, `dietlock.sqlite` is inside this run's data dir and `PRAGMA user_version` is 1, and a viewable window is named DietLock. Anything else prints `doctor FAIL` and the script exits 1. Do not drive a failed doctor. Relaunch instead of reusing a window you did not start.

## Drive

The feature map is `features/README.md`. Follow one feature file. Prefer element ids over the visible labels: both forms have a submit button named `Enregistrer` and a field named `Date`.

Read the screen:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify text
.cursor/skills/verify-dietlock/scripts/dietlock-verify eval 'document.getElementById("trend").textContent'
```

`eval` runs one synchronous JavaScript expression in the page and prints the value. `text` prints `document.body.innerText`. `db` prints the `weigh_in` and `intake` rows (the side effect behind the screen).

Stable handles, from `src/index.html`:

| Handle | What the user sees |
| --- | --- |
| `#weigh-in-form` `#weight` `#fasted` `#date` | Pesée. Weight is text, fasted defaults to checked, date defaults to today. |
| `#intake-form` `#protein` `#carbs` `#fat` `#water` `#intake-date` | Alimentation. Water may be left empty. |
| `#error` | `role="alert"`. Hidden when empty. |
| `#confirm` `#confirm-text` `#confirm-replace` `#confirm-cancel` | Replacement confirmation. Hidden until a date is already filled. |
| `#trend` | `pas encore de tendance`, or a weight such as `82,6 kg`. |
| `#days` | One `li` per day, 30 lines, most recent first. |

Set a field and submit the form the user would submit. The submit listener is the real save path:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify eval 'document.getElementById("weight").value = "82,6"; document.getElementById("fasted").checked = true; document.getElementById("weigh-in-form").requestSubmit(); "submitted"'
```

`requestSubmit()` returns before the command finishes. Poll `text` until the history line changes. Then run `db` and confirm the row. Do not call `window.__TAURI__.core.invoke` from the harness: that skips the form.

## Evidence

Proof files go to `/tmp/dietlock-verify-evidence/<run-id>/`. Put that path, or the picture itself, in the PR body.

- Screenshot of the DietLock window (not the whole X root): `dietlock-verify screenshot` or the PNG written by `check-today`. The window is 800×600.
- DOM text beside it: `today.dom.txt` and `today.dom.json` for the landing check, or the `text` / `eval` output for another journey.
- Side effect: `dietlock-verify db` (and, for the landing check, zero rows).
- Command-line proof, with the app stopped or still up: `dietlock-verify cargo-test` or `dietlock-verify cargo-test -- -p dietlock-core`. The script writes `cargo-test.txt` (command, exit code, stdout, stderr) and prints it. There is no JavaScript test runner.

Landing-screen check, which is the journey to run on a fresh launch:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify check-today
```

It exits 0 only when the window shows the four headings, `#trend` is `pas encore de tendance`, the weigh-in date is the machine's local today, the history has 30 empty days, the alert is hidden, the database is schema 1 with no rows, the PNG is 800×600, and OCR of that PNG sees `Tendance du jour`, `pas encore de tendance`, `30 derniers jours`, and `pas de pesée`. A blank or chrome-only capture fails the OCR lines even if the DOM is right.

Capture the action and the resulting state for any other journey: the command you ran, the history line or confirmation text after it, and `db` showing the row that was or was not written. Do not mock the store. The isolated data dir is the production store, empty because the run created it.

## Cleanup

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify cleanup
```

This kills the pid this run started (the app, its children, that Xvfb, that D-Bus daemon), then deletes `/tmp/dietlock-verify/<run-id>/`. It does not kill by process name. It does not delete `/tmp/dietlock-verify-evidence/<run-id>/`. After cleanup, check that the PNG or `cargo-test.txt` is still at the path `cleanup` prints.

Run `cleanup` after a failed launch too, so a crashed WebKit does not keep the display. `launch` already kills processes if it never reaches `ready`; `cleanup` is still safe to repeat.

`cargo-test` with no current run creates a new evidence directory and prints it. To keep the screenshot and the test log together, run `cargo-test` before `cleanup`.

## Helpers

The only helper is the executable script. Its commands:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify launch
.cursor/skills/verify-dietlock/scripts/dietlock-verify doctor
.cursor/skills/verify-dietlock/scripts/dietlock-verify text
.cursor/skills/verify-dietlock/scripts/dietlock-verify eval 'document.title'
.cursor/skills/verify-dietlock/scripts/dietlock-verify db
.cursor/skills/verify-dietlock/scripts/dietlock-verify screenshot /tmp/dietlock-verify-evidence/<run-id>/screen.png
.cursor/skills/verify-dietlock/scripts/dietlock-verify check-today
.cursor/skills/verify-dietlock/scripts/dietlock-verify cargo-test
.cursor/skills/verify-dietlock/scripts/dietlock-verify cargo-test -- -p dietlock-core
.cursor/skills/verify-dietlock/scripts/dietlock-verify cleanup
```

`screenshot` with no path writes `<evidence>/screen.png`.

## What this cannot verify

- The macOS `.dmg`, code signing, and Gatekeeper prompts. The binary under test is the Linux debug build of the same Tauri shell.
- A release build. Developer extras, and therefore the inspector, are off without the `devtools` feature. Do not flip that feature for a proof; stay on the debug binary.
- A normal browser opened on `src/`. That page is not the app.
- Alerting. v0.1 does not show an alert in the window.
- Pixel-perfect layout. The UI is unstyled; at 800×600 the water label wraps onto a second line. That wrap is the current screen, not a failed launch.
- Two instances at once. Launch refuses while one run is alive.

## Keeping the map honest

When the screen or the commands change, run `/maintain-verification-skill` against `.cursor/skills/verify-dietlock/`. The feature files are the source for later proofs; a screenshot of only the landing screen does not cover a journey the map lists and this run did not drive.
