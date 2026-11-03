# Today's screen

Opening DietLock shows one window titled DietLock: the weigh-in form, the intake form, today's trend, and the last 30 days. On a new database the trend is not available yet and every day is empty.

## Sub-features

- `today-open` shows the four sections in one window, with no extra page to open.
- `today-trend-empty` shows `pas encore de tendance` when fewer than four fasted weigh-ins exist in the seven-day window.
- `today-history-empty` lists 30 days, most recent first, each with no weigh-in and no intake.
- `today-weigh-in-date` proposes today in the weigh-in date field.

## How to get to it (user POV)

- Launch DietLock. The window is this screen. There is no menu and no other view.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify launch` printed `ready trend: pas encore de tendance`.
- `dietlock-verify doctor` printed `doctor PASS`.
- `dietlock-verify db` prints `weigh_in 0` and `intake 0`.

- **Open the window.** Launch is the open action. The window title is DietLock and the headings read `Pesée`, `Alimentation`, `Tendance du jour`, and `30 derniers jours`.
- **Read the trend.** Run `dietlock-verify eval 'document.getElementById("trend").textContent'`. The output is `pas encore de tendance`.
- **Read the history.** Run `dietlock-verify eval 'document.getElementById("days").children.length + " " + document.getElementById("days").firstElementChild.textContent'`. The count is `30` and the first line is `<today> : pas de pesée | pas d'alimentation`, where `<today>` is `#date`'s value.
- **Read the proposed weigh-in date.** Run `dietlock-verify eval 'document.getElementById("date").value'`. It equals the machine's local date, `YYYY-MM-DD`.
- **Confirm the store.** Run `dietlock-verify db`. Schema use is implied by doctor (`user_version` 1). Both tables have 0 rows.
- **Proof.** Run `dietlock-verify check-today`. Exit code `0`, and `check-today.txt` starts with `check-today PASS`. The PNG at the path it prints shows the four headings, `pas encore de tendance`, and a first history line with `pas de pesée`.

## Gotchas

- `#intake-date` is not always today. Before 04:00 it is yesterday. From 04:00 to noon, with no intake yesterday or today, it is also yesterday. Do not fail the landing check for that. `check-today` asserts `#date` only.
- The date widget may paint the ISO value in a locale order such as `11/03/2026`. Assert `#date`.value and the history line, which are `YYYY-MM-DD`.
- One fasted weigh-in does not change the trend. The empty sentence stays until four fasted weigh-ins sit in the seven days ending today.
- The water label wraps at 800×600. The wrapped label is still the intake form.
- `check-today` fails if a previous drive already saved a row. It is the empty-store journey. Run it immediately after launch.
- Serving `src/index.html` in a browser is not this screen: the trend stays empty because the Tauri commands never run.
