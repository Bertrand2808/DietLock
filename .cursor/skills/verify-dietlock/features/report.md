# Report

`Rapport` sits in the title bar, immediately left of `＋ Activité`. A click copies a PNG of the 15 days ending today, oldest first, and the button becomes `✓ Copié`. A line `Rapport copié · 15 jours, du … au …` appears with a canvas thumbnail. The image has one card per day (date, weight, `pas à jeun` when the weigh-in was not fasted, kcal, P/G/L, activities with type and note, water), the weight and Trend curve, the box `Tendance sur 15 jours` when both ends have a Trend, and the footer `Case vide = rien saisi. Énergie = 4P+4G+9L.` A missing value is a blank cell. An entered 0 is shown as 0. A weigh-in that is in Alert on `Aujourd'hui` is a normal weight on the image, with no alert mark and no red. The same image is produced after a restart.

## Sub-features

- `report-copy` turns `#report` into `✓ Copié` and shows `#report-msg` plus `#report-thumb`.
- `report-png` is the generated image, read from `#report-source` with `toDataURL`.
- `report-cards` lists 15 days, oldest first, with weight, kcal, P/G/L, water and activities.
- `report-trend` draws the 15-day weight and Trend curve and the box `Tendance sur 15 jours`.
- `report-gap` leaves a day without a Trend out of the Trend line.
- `report-empty` leaves a day or a field with no entry blank, and shows an entered 0.
- `report-alert` shows no Alert and no red when the home screen marks a weigh-in in Alert.
- `report-footer` prints `Case vide = rien saisi. Énergie = 4P+4G+9L.`
- `report-relaunch` copies the same range after `relaunch`.
- `report-clipboard` records whether `image/png` is on the Linux clipboard. macOS is not proved here.

## How to get to it (user POV)

- The button is `#report` in the title bar, on `Aujourd'hui` or on `Graphiques`.
- Past days are saved with the same commands as the entry card, then the user clicks `Rapport`.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- The store is the empty one `launch` created.

- **Seed.** From `Aujourd'hui`, save weigh-ins, intakes and one activity with `invoke`, which is the command `#save` uses. Poll `#report` until it exists. Dates are ISO and must not be after `#weigh-chip`'s `data-date`.
- **Copy.** Run `dietlock-verify eval 'document.getElementById("report").click(); "clicked"'`. Poll until `#report` reads `✓ Copié` or `#report-error` is visible. On success, `#report-msg` starts with `Rapport copié · 15 jours, du ` and `#report-thumb` is a canvas with a non-zero height. Screenshot the window.
- **PNG.** Run `dietlock-verify eval 'document.getElementById("report-source").toDataURL("image/png")'` and write the decoded bytes to a `.png`. The picture shows 15 day labels, oldest first, and the footer.
- **Gap.** Save a run of fasted weigh-ins that gives a Trend at the start of the 15 days and today, with days in between that have none. Click `Rapport` again. The PNG contains `Tendance sur 15 jours` and a break in the Trend line.
- **Empty and alert.** Leave at least one day of the 15 with no row. Save one intake at `0` protein, carbs, fat and water. Save four fasted weigh-ins at `79,0` on the days before today and today at `82,0` fasted. The history row for today shows the alert mark. The new PNG shows `82,0 kg` and `0 kcal` and does not show `Alerte` or a red mark. The empty day has a date and no invented `0`.
- **Restart.** `dietlock-verify relaunch`, then click `Rapport` again. `#report-msg` repeats the same range. `db` is unchanged.
- **Clipboard.** After a click, run `xclip -selection clipboard -t TARGETS -o` on the run's `DISPLAY`. Record whether `image/png` is listed. A refusal shows `#report-error` with `Le rapport n'a pas pu être copié.` and the button stays `Rapport`.
- **Proof.** The window after `✓ Copié`, the PNG file, the gap PNG, the alert PNG beside the history row, and the window after `relaunch`.

## Gotchas

- `eval` does not wait for the clipboard promise. Poll the button text.
- The thumbnail is a canvas. An `<img>` with a `blob:` or `data:` URL is blocked by the page CSP.
- `#report-source` is off-screen. `toDataURL` still reads it. Do not put that URL into an `<img>`.
- macOS clipboard behaviour is not proved on this Linux VM. A failure here is not a reason to add `tauri-plugin-clipboard-manager`.
- The title bar is a drag region on macOS. Whether `Rapport` clears the traffic lights is not proved here.
- Today is the date the Tauri command reads. Do not seed a date after `#weigh-chip`'s `data-date`.
