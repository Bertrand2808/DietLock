---
name: verify-dietlock
description: "Drive the DietLock Tauri app on a Linux VM and prove a screen or a test run. Use when a change needs a screenshot or command output in the PR body, or when checking Aujourd'hui, the weight curve, a weigh-in, an intake, or a replacement."
---

# Verify DietLock

DietLock is a desktop app (Tauri + Rust) with two screens in one window: `Aujourd'hui` (the entry card and the history) and `Poids` (the weight curve). The tabs are `#tab-today` and `#tab-weight`. The release install target is macOS; this skill proves the same shell on a Linux cloud VM.

The harness is `.cursor/skills/verify-dietlock/scripts/dietlock-verify`. Run it from anywhere. It finds the repo as the parent of `.cursor`.

## Proof in every pull request

Every pull request includes its proof in the PR body before it is ready for review:

- A change the user can see: the PNG this skill writes, plus the passing check output (`check-today.txt` or the `eval` / `text` / `db` output for the journey you drove).
- A change covered by tests, with no UI surface: the `cargo test` output, or `node --test` for `src/*.test.js` (the full suite, or the crate or file you changed).

A pull request that contains neither is not ready. Review the proof as well as the diff. One of the two is enough; a UI change still needs the screenshot, because CI never launches the window.

## Launch

From the repo root, on a Linux VM that has the Tauri system libraries (`libwebkit2gtk-4.1`, GTK 3, libsoup 3) plus `Xvfb`, `dbus-launch`, `xwininfo`, `xwd`, `convert` (ImageMagick), `tesseract` with `fra`, and Node for the frontend tests:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify launch
```

`launch` builds the debug binary with `cargo build -p dietlock` (the dev profile turns WebKit developer extras on; a release binary does not, so the inspector stays closed), starts a private Xvfb at 1280×800 and a private D-Bus session, and runs `target/debug/dietlock` with a disposable `HOME` and `XDG_DATA_HOME`. The window is 1080×700. The store is created empty and migrated to schema `user_version` 2. There is no seed data: an empty database is the usable state.

Ready means the process is up, the window is mapped, and `#headline` has text. On a fresh data dir between 05:00 and 18:00, `launch` prints `ready headline: Bonjour.` (`Bonsoir.` outside that range), then:

```text
run <run-id>
binary <repo>/target/debug/dietlock
display :<n>
inspector http://127.0.0.1:<port>/
data /tmp/dietlock-verify/<run-id>/data/com.bertrand2808.dietlock/dietlock.sqlite
evidence /tmp/dietlock-verify-evidence/<run-id>
ready
```

The run id is remembered in `/tmp/dietlock-verify/current`. Set `DIETLOCK_VERIFY_RUN` to drive a specific run. Launch refuses when that run's process is still alive. Clean up first.

WebKit is started with `WEBKIT_DISABLE_COMPOSITING_MODE=1` and `WEBKIT_DISABLE_DMABUF_RENDERER=1` so the window paints under Xvfb. The inspector listens on `WEBKIT_INSPECTOR_HTTP_SERVER`. Do not point a browser at `src/index.html`: `src/main.js` calls `window.__TAURI__` and nothing is saved.

## Doctor

Run this before the first drive, and again after any failure, before driving further:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify doctor
```

Pass means all of the following: the recorded pid is the `dietlock` binary, its Xvfb is alive, the inspector page lists a target named DietLock at `tauri://localhost`, `dietlock.sqlite` is inside this run's data dir and `PRAGMA user_version` is 2, and a viewable window is named DietLock. Anything else prints `doctor FAIL` and the script exits 1. Do not drive a failed doctor. Relaunch instead of reusing a window you did not start.

## Drive

The feature map is `features/README.md`. Follow one feature file. Prefer element ids. The date controls are chips (`#weigh-chip`, `#intake-chip`), not `<input type="date">`. One button, `#save`, records whichever block the user started.

Read the screen:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify text
.cursor/skills/verify-dietlock/scripts/dietlock-verify eval 'document.getElementById("headline").textContent'
```

`eval` runs one synchronous JavaScript expression in the page and prints the value. `text` prints `document.body.innerText`. `db` prints the `weigh_in` and `intake` rows.

Stable handles, from `src/index.html`:

| Handle | What the user sees |
| --- | --- |
| `#tab-today` `#screen-today` | L'onglet `Aujourd'hui`. Selected on launch. |
| `#tab-weight` `#screen-weight` | L'onglet `Poids`. Hidden until chosen. |
| `#headline` `#subhead` | Greeting (`Bonjour.` / `Bonsoir.`) or `82,6` plus `kg de tendance`, and the line under it. |
| `#weigh-chip` `#weight` `#fasted` | Date de la pesée, weight in kg, `à jeun` checked by default. The chip's `data-date` is `YYYY-MM-DD`. |
| `#intake-chip` `#protein` `#carbs` `#fat` `#water` `#forgot` | Date de l'alimentation, P / G / L / Eau. On an empty store the chip is yesterday. `#forgot` shows `hier oublié` from 04:00. |
| `#weigh-error` `#intake-error` | `role="alert"`. Hidden when empty. Messages are French. |
| `#confirm` `#confirm-text` `#confirm-replace` `#confirm-cancel` | Replacement question. Hidden until a date is already filled. |
| `#save` `#saved` | The `↑` button. `#saved` says `✓ Journée enregistrée` after a write. |
| `#hist` | Empty sentence, or `30 derniers jours` and one `.day` row per filled day. |
| `#w-headline` `#w-sub` `#w-range` `#w-count` `#chartbox` | Weight screen. Empty copy is `Pas encore de courbe.` |

Open `Poids` the way the user does:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify eval 'document.getElementById("tab-weight").click(); "opened"'
```

The curve loads after the click. Poll `#w-headline` until it has text.

Save by clicking `#save`, not by calling `invoke`. The click returns before the write finishes:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify eval 'document.getElementById("weight").value = "82,6"; document.getElementById("fasted").checked = true; document.getElementById("save").click(); "submitted"'
```

An empty weight field does not save a weigh-in. An empty macro row does not save an intake. Filling one block and leaving the other empty saves only the filled block.

## Evidence

Proof files go to `/tmp/dietlock-verify-evidence/<run-id>/`. Put that path, or the picture itself, in the PR body.

- Screenshot of the DietLock window (not the whole X root): `dietlock-verify screenshot` or the PNG written by `check-today`. The window is 1080×700. `check-today` captures `Aujourd'hui` before it opens `Poids`.
- DOM text beside it: `today.dom.txt`, `today.dom.json`, and `weight.dom.json`.
- Side effect: `dietlock-verify db` (zero rows on the landing check).
- Rust proof: `dietlock-verify cargo-test` or `dietlock-verify cargo-test -- -p dietlock-core`. The script writes `cargo-test.txt`.
- Frontend proof, no browser: `dietlock-verify frontend-test`, which runs `node --test` on `src/calendar.test.js`, `src/chart.test.js`, `src/keys.test.js`, `src/save.test.js`, and `src/energy.test.js`, and writes `frontend-test.txt`.

Landing check, on a fresh launch:

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify check-today
```

It exits 0 only when `Aujourd'hui` shows the greeting for the local hour, the empty subtitle, the weigh-in chip on today, the intake chip on yesterday (with `hier oublié` from 04:00, hidden before that), the empty history sentence, no errors, schema 2 with no rows, a 1080×700 PNG whose OCR sees `Pas encore de saisie`, `première journée`, `Aujourd`, and `Poids`, and, after the `Poids` tab is opened, `Pas encore de courbe.`, `15 derniers jours`, and `aucune pesée`. The large greeting is asserted from the page text; OCR of that serif line is unreliable. A blank capture fails the OCR lines.

Capture the action and the resulting state for any other journey: the command you ran, the history row or the confirmation text after it, and `db`. Do not mock the store.

## Cleanup

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify cleanup
```

This kills the pid this run started (the app, its children, that Xvfb, that D-Bus daemon), then deletes `/tmp/dietlock-verify/<run-id>/`. It does not kill by process name. It does not delete `/tmp/dietlock-verify-evidence/<run-id>/`. After cleanup, check that the PNG or the test log is still at the path `cleanup` prints.

Run `cleanup` after a failed launch too. `launch` already kills processes if it never reaches `ready`; `cleanup` is still safe to repeat.

`cargo-test` and `frontend-test` with no current run create a new evidence directory. To keep the screenshot and the logs together, run them before `cleanup`.

## Helpers

```bash
.cursor/skills/verify-dietlock/scripts/dietlock-verify launch
.cursor/skills/verify-dietlock/scripts/dietlock-verify relaunch
.cursor/skills/verify-dietlock/scripts/dietlock-verify doctor
.cursor/skills/verify-dietlock/scripts/dietlock-verify text
.cursor/skills/verify-dietlock/scripts/dietlock-verify eval 'document.title'
.cursor/skills/verify-dietlock/scripts/dietlock-verify db
.cursor/skills/verify-dietlock/scripts/dietlock-verify screenshot /tmp/dietlock-verify-evidence/<run-id>/screen.png
.cursor/skills/verify-dietlock/scripts/dietlock-verify check-today
.cursor/skills/verify-dietlock/scripts/dietlock-verify cargo-test
.cursor/skills/verify-dietlock/scripts/dietlock-verify cargo-test -- -p dietlock-core
.cursor/skills/verify-dietlock/scripts/dietlock-verify frontend-test
.cursor/skills/verify-dietlock/scripts/dietlock-verify cleanup
```

`screenshot` with no path writes `<evidence>/screen.png`.

`relaunch` closes the DietLock process of the current run and starts the same binary again on that run's data directory, Xvfb and D-Bus session. The database is kept. Use it to prove a figure survives a restart. It refuses when the app or its Xvfb is already gone. `cleanup` is still what deletes the data directory.

## What this cannot verify

- The macOS `.dmg`, code signing, Gatekeeper, and the overlay title bar (`titleBarStyle` and `hiddenTitle` are macOS window chrome). On Linux the check uses the page content and the 1080×700 window.
- A release build. Developer extras, and therefore the inspector, are off without the `devtools` feature. Stay on the debug binary.
- A normal browser opened on `src/`. That page is not the app.
- The system light or dark theme. The page follows `prefers-color-scheme`. The check reads text, so either theme passes.
- Two instances at once. Launch refuses while one run is alive.

## Keeping the map honest

When the screen or the commands change, run `/maintain-verification-skill` against `.cursor/skills/verify-dietlock/`. The feature files are the source for later proofs; a screenshot of `Aujourd'hui` does not cover a journey the map lists and this run did not drive.
