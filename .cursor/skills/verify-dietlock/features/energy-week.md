# Energy week

Beside the weight trend, `Aujourd'hui` shows the mean energy of the 7 days ending yesterday, how many of those days were entered, and the signed gap with the 7 days before. A day without macros is left out. A day entered at 0 kcal counts. No intake in the window is a sentence, not 0. No intake in the previous week says there is no comparison.

## Sub-features

- `energy-mean` shows the mean in kcal and `sur N jours saisis` beside the trend.
- `energy-gap` shows a signed gap such as `+120 kcal` when the previous week has intakes.
- `energy-no-comparison` says `pas de comparaison` and shows no gap when the previous week is empty.
- `energy-skip-empty` leaves a day without macros out of N, and counts a day entered at 0 kcal.
- `energy-none` shows `Pas de moyenne sur 7 jours.` when the window has no intake.
- `energy-refresh` updates the mean and the gap after an add, a replacement or a deletion, without restarting.
- `energy-reopen` shows the same figures after the app is closed and opened again on the same data.

## How to get to it (user POV)

- Stay on `Aujourd'hui`. The mean sits in the header, beside the weight trend when a trend exists, and under the greeting otherwise.
- Enter macros from the intake row. Pick the day on `#intake-chip` before saving.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- Read today from `#weigh-chip`'s `data-date`. The window is the 7 days ending yesterday. The previous week is the 7 days before that.
- A day in that window is chosen from the calendar, not typed into a date input.

- **Read an empty window.** With no intake on those 7 days, `#energy-figure` is `Pas de moyenne sur 7 jours.` and `#energy-note` is empty. The figure contains no digit.
- **Enter a week.** For each past day, open `#intake-chip`, click that day in `#date-pop-grid`, fill P, G and L, clear the weight field, and click `#save`. Poll `dietlock-verify text` until the day's kcal appears in the history.
- **Read the mean.** `#energy-figure` is the grouped kcal and `#energy-note` starts with `sur N jours saisis`. N is the number of intakes in the window, including a day at 0 kcal, excluding a day with none.
- **Read no comparison.** When the previous week has no intake, `#energy-note` contains `pas de comparaison` and does not contain `+0` or `−0`.
- **Read the gap.** When the previous week has intakes, `#energy-note` contains a signed kcal gap. The gap is the displayed mean minus the previous week's displayed mean.
- **Replace.** Submit new macros on a filled day, then click `#confirm-replace`. The mean and the gap change before the next launch.
- **Delete.** Click `× alim.` on a history row, then `#delete-yes`. N and the mean change.
- **Reopen.** Run `dietlock-verify relaunch`. The same `#energy-figure` and `#energy-note` come back. `dietlock-verify db` still lists the intakes.
- **Proof.** One screenshot per case, taken on `Aujourd'hui`, plus the `text` line and `db`.

## Gotchas

- Today is outside the window. An intake saved on today changes the subtitle (`Aujourd'hui · … kcal`) and does not by itself create a mean.
- A day with neither a weigh-in nor an intake does not appear in the history. To show a skipped day, save a weigh-in on it and leave the macros empty.
- `#save` writes a weigh-in whenever the weight field is not empty. Clear it before saving macros alone.
- The fields stay filled after a save on a hand-picked date. Clear them before the next day.
- `relaunch` keeps the data directory. `cleanup` deletes it.
