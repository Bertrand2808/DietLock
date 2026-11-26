# Report

`Rapport` is the third tab, after `Aujourd'hui` and `Graphiques`. The page shows the 15-day image at 480 px, the same canvas that is copied. `Copier` writes that PNG to the clipboard. The button reads `✓ Copié` for a few seconds, and a toast `Rapport copié · 15 jours, du … au …` appears. There is no thumbnail. `Télécharger` opens the native save dialog. Closing it writes nothing and shows no toast. The image has a title and date range, the box `Tendance sur 15 jours` only when both ends have a Trend, a summary, the weight and Trend curve, one card per day, and the footer `Case vide = rien saisi. Énergie = 4P+4G+9L.` A missing value is a blank cell. An entered 0 is shown as 0. A weigh-in that is in Alert on `Aujourd'hui` is a normal weight on the image, with no alert mark and no red. The same image is produced after a restart.

## Sub-features

- `report-tab` opens `#tab-report` and shows `#screen-report`, `#report-copy`, `#report-save` and `#report-canvas`.
- `report-copy` turns `#report-copy` into `✓ Copié` and shows `#toast` with `#toast-msg`.
- `report-png` is the generated image, read from `#report-canvas` with `toDataURL`.
- `report-summary` draws the mean per day of energy, protein, carbohydrate, fat and water, each with a counted-day note, and the number of days with an Activity.
- `report-cards` lists 15 days, oldest first, with weight, kcal, P/G/L, water and activities.
- `report-trend` draws the 15-day weight and Trend curve and the box `Tendance sur 15 jours`, including `Écart` on the same line as the two ends.
- `report-gap` leaves a day without a Trend out of the Trend line.
- `report-no-end` hides the box when either end has no Trend.
- `report-empty` leaves a day or a field with no entry blank, and shows an entered 0.
- `report-alert` shows no Alert and no red when the home screen marks a weigh-in in Alert.
- `report-footer` prints `Case vide = rien saisi. Énergie = 4P+4G+9L.`
- `report-save` opens the native dialog. Cancelling it leaves the database unchanged and `#toast` hidden.
- `report-relaunch` shows the same range after `relaunch`.
- `report-clipboard` records whether `image/png` is on the Linux clipboard. macOS is not proved here.

## How to get to it (user POV)

- The tab is `#tab-report`. The title bar has no report button.
- Past days are saved with the same commands as the entry card, then the user opens `Rapport`.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- The store is the empty one `launch` created.

- **Seed.** From `Aujourd'hui`, save weigh-ins, intakes and one activity with `invoke`, which is the command `#save` uses. Dates are ISO and must not be after `#weigh-chip`'s `data-date`.
- **Open.** Run `dietlock-verify eval 'document.getElementById("tab-report").click(); "opened"'`. Poll until `#report-canvas` has a non-zero height and `#report-copy` is enabled. Screenshot the page.
- **Copy.** A synthetic `click()` has no user gesture, so WebKit refuses the clipboard write. Move the pointer onto `#report-copy` and click it. Poll until `#report-copy` reads `✓ Copié` or `#report-error` is visible. On success, `#toast` is shown and `#toast-msg` starts with `Rapport copié · 15 jours, du `. Screenshot the window.
- **PNG.** Run `dietlock-verify eval 'document.getElementById("report-canvas").toDataURL("image/png")'` and write the decoded bytes to a `.png`. The picture shows 15 day labels, oldest first, the summary, and the footer.
- **Gap.** Save a run of fasted weigh-ins that gives a Trend at the start of the 15 days and today, with days in between that have none. Open `Rapport` again. The PNG contains `Tendance sur 15 jours` and a break in the Trend line.
- **Missing end.** On a fresh store, save fewer than four fasted weigh-ins, all near today, so the first day of the 15 has no Trend. The PNG has no `Tendance sur 15 jours`.
- **Empty and alert.** Leave at least one day of the 15 with no row. Save one intake at `0` protein, carbs, fat and water. Save four fasted weigh-ins at `79,0` on the days before today and today at `82,0` fasted. The history row for today shows the alert mark. The PNG shows `82,0 kg` and `0 kcal` and does not show `Alerte` or a red mark. The empty day has a date and no invented `0`.
- **Download.** Click `#report-save`. The native dialog opens. Close it without choosing a file. `#toast` stays hidden, `#report-error` stays hidden, and `db` is unchanged.
- **Restart.** `dietlock-verify relaunch`, then open `Rapport` again. The canvas range repeats. `db` is unchanged.
- **Clipboard.** After the pointer click, run `xclip -selection clipboard -t TARGETS -o` on the run's `DISPLAY`. Record whether `image/png` is listed. A refusal shows `#report-error` with `Le rapport n'a pas pu être copié.` and the button stays `Copier`.
- **Proof.** The report page, the window after `✓ Copié`, the PNG file, the gap in that PNG, the PNG with no trend box, the alert row beside the PNG, the window after a cancelled download, and the window after `relaunch`.

## Gotchas

- `eval` does not wait for the clipboard promise. Poll the button text.
- The page canvas is the exported image. An `<img>` with a `blob:` or `data:` URL is blocked by the page CSP. Do not put a data URL into an `<img>`.
- macOS clipboard behaviour is not proved on this Linux VM. A failure here is not a reason to add `tauri-plugin-clipboard-manager`.
- The save dialog is a native window. Cancelling it is a pointer or keyboard action on that window, not a DOM click.
- Today is the date the Tauri command reads. Do not seed a date after `#weigh-chip`'s `data-date`.
