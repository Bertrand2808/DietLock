# Today's screen

Opening DietLock shows `Aujourd'hui`: two labeled columns, the entry card, and an empty history. The `Poids` tab is in the title bar and is not selected.

## Sub-features

- `today-open` shows `Aujourd'hui` selected and `Poids` not selected.
- `today-empty` shows `Pas encore de pesée.` in the left column and `Pas de moyenne sur 7 jours.` in the right column.
- `today-chips` proposes today for the weigh-in. On an empty store the intake chip is yesterday all day.
- `today-history-empty` shows the invitation instead of a list of days.

## How to get to it (user POV)

- Launch DietLock. `Aujourd'hui` is the selected tab.
- Choose the `Aujourd'hui` tab again after visiting `Poids`.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify launch` printed `ready headline: Pas encore de pesée.`
- `dietlock-verify doctor` printed `doctor PASS`.
- `dietlock-verify db` prints `weigh_in 0` and `intake 0`.

- **Read the empty columns.** Run `dietlock-verify eval 'document.getElementById("headline").textContent'`. It is `Pas encore de pesée.` Run `dietlock-verify eval 'document.getElementById("subhead").textContent'`. It is empty. Run `dietlock-verify eval 'document.getElementById("energy-figure").textContent'`. It is `Pas de moyenne sur 7 jours.` and contains no digit.
- **Read the chips.** Run `dietlock-verify eval 'document.getElementById("weigh-chip").dataset.date + " " + document.getElementById("weigh-chip").innerText'`. The date is the local today and the text contains `Aujourd'hui`. On an empty store the intake chip's `data-date` is yesterday. `#forgot` is hidden before 04:00 and shows `hier oublié` from 04:00 onward.
- **Read the history.** Run `dietlock-verify eval 'document.getElementById("hist").innerText'`. It contains `Ta première journée apparaîtra ici.` and `Commence par ton poids de ce matin`. There is no `.day` row.
- **Proof.** Run `dietlock-verify check-today`. Exit code `0`, and `check-today.txt` starts with `check-today PASS`. The PNG is the `Aujourd'hui` screen at 1080×700 and shows the two empty columns, the card, and the empty history. The same command then opens `Poids` and checks that screen; the PNG is taken before that click.

## Gotchas

- `#headline` is `Pas encore de pesée.` when the store has no weigh-in. It is not a greeting.
- The history does not list thirty empty days. Only days with a weigh-in or an intake appear, under `30 derniers jours`.
- The date a user reads is `Aujourd'hui · mar. 3 nov.`, not an ISO string painted in a widget. Assert `data-date` for the ISO value.
- On an empty store the intake chip stays on yesterday after noon too. It moves to today only once yesterday has an intake. Do not treat `hier oublié` in the afternoon as a wrong date.
- `check-today` fails if a previous drive already saved a row. Run it immediately after launch.
- Serving `src/index.html` in a browser leaves the headline empty, because the commands never run.
