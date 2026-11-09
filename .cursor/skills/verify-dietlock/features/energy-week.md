# Energy week

The right column of `Aujourd'hui` shows the mean energy of the 7 days ending yesterday, how many of those days were entered, yesterday's kcal when that day has an intake, and a pill for the signed gap with the 7 days before. A day without macros is left out. A day entered at 0 kcal counts. No intake in the window is a sentence, not 0. No intake in the previous week hides the pill. A gap of 0 stays the words `+0 kcal` and is not a pill.

## Sub-features

- `energy-mean` shows the mean in kcal and `N jours saisis` under the label `Calories · moyenne / jour`.
- `energy-gap` shows a pill such as `−80 vs semaine d'avant` when the previous week has intakes. The pill is green when the gap is negative and amber when it is positive.
- `energy-no-comparison` shows no pill and no gap number when the previous week is empty.
- `energy-yesterday` adds `hier …` only when yesterday has an intake. A missing yesterday is not `hier 0`.
- `energy-skip-empty` leaves a day without macros out of N, and counts a day entered at 0 kcal.
- `energy-none` shows `Pas de moyenne sur 7 jours.` when the window has no intake.
- `energy-refresh` updates the mean and the gap after an add, a replacement or a deletion, without restarting.
- `energy-reopen` shows the same figures after the app is closed and opened again on the same data.

## How to get to it (user POV)

- Stay on `Aujourd'hui`. The mean is the right column. The trend is the left column.
- Enter macros from the intake row. Pick the day on `#intake-chip` before saving.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- Read today from `#weigh-chip`'s `data-date`. The window is the 7 days ending yesterday. The previous week is the 7 days before that.
- A day in that window is chosen from the calendar, not typed into a date input.

- **Read an empty window.** With no intake on those 7 days, `#energy-figure` is `Pas de moyenne sur 7 jours.` and `#energy-note` is empty. The figure contains no digit.
- **Enter a week.** For each past day, open `#intake-chip`, click that day in `#date-pop-grid`, fill P, G and L, clear the weight field, and click `#save`. Poll `dietlock-verify text` until the day's kcal appears in the history.
- **Read the mean.** `#energy-figure` is the grouped kcal and `#energy-note` starts with `N jours saisis` or `1 jour saisi`. N is the number of intakes in the window, including a day at 0 kcal, excluding a day with none.
- **Read no comparison.** When the previous week has no intake, `#energy-pill` is absent. `#energy-note` does not contain `+0`, `−0`, or `vs semaine`.
- **Read the gap.** When the previous week has intakes and the gap is not 0, `#energy-pill` is the signed gap plus `vs semaine d'avant`. A negative gap has class `down`. A positive gap has class `up`. A gap of 0 has no pill, and `#energy-note` contains `+0 kcal`.
- **Read yesterday.** `#energy-note` contains `hier` and yesterday's grouped kcal only when yesterday has an intake. Match that history row by its date.
- **Replace.** Submit new macros on a filled day, then click `#confirm-replace`. The mean and the gap change before the next launch.
- **Delete.** Click `× alim.` on a history row, then `#delete-yes`. N and the mean change.
- **Reopen.** Run `dietlock-verify relaunch`. The same `#energy-figure` and `#energy-note` come back. `dietlock-verify db` still lists the intakes.
- **Proof.** One screenshot per case, taken on `Aujourd'hui`, plus the `text` line and `db`.

## Gotchas

- Today is outside the window. An intake saved on today does not by itself create a mean, and it does not fill the `hier` line.
- A day with neither a weigh-in nor an intake does not appear in the history. To show a skipped day, save a weigh-in on it and leave the macros empty.
- `#save` writes a weigh-in whenever the weight field is not empty. Clear it before saving macros alone.
- The fields stay filled after a save on a hand-picked date. Clear them before the next day.
- `relaunch` keeps the data directory. `cleanup` deletes it.
