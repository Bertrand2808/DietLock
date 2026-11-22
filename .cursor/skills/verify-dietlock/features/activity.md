# Activities

Activities are the physical activities of one day, entered together in a modal form opened by the `＋ Activité` button in the title bar. The button is visible on both screens and is separate from the entry card. One save writes every typed row for the chosen date. The weigh-in and the intake of that date are never touched.

A history day that holds at least one activity shows a badge, `1 activité` or `N activités`. A day with none has no badge. Hovering the badge lists each activity (type, then the note when there is one) and `Clic pour gérer`. Clicking the badge opens that day's list under the row; clicking it again folds the list. Only one list is open. `Supprimer` removes that activity at once, with no confirmation. Deleting the last activity of a day removes the badge and folds the list. The trend, the calorie mean and the macro means stay as they were. A deleted activity is still gone after a restart.

## Sub-features

- `activity-open` opens the form with today's date chip and one empty row.
- `activity-save` saves several rows for the chosen date in one click and closes the form.
- `activity-refuse` keeps the form open with a French message when a row has no type.
- `activity-cancel` closes the form without writing, through `Annuler` or Esc.
- `activity-restart` shows the rows are still stored for the right date after `relaunch`.
- `activity-badge` marks a history day that has activities and leaves a day without activities unmarked.
- `activity-tooltip` shows each type and note on hover, then `Clic pour gérer`.
- `activity-list` opens that day's activities under the row. A second click on the same badge folds it. Opening another day folds the previous list.
- `activity-delete` removes the clicked activity at once. The weigh-in and the intake of that date stay.
- `activity-last` removes the badge and folds the list when the last activity of the day is deleted.
- `activity-figures` leaves the trend, the calorie mean and the macro means unchanged when activities are added or deleted.
- `activity-delete-restart` keeps a deleted activity gone after `relaunch`, and keeps the ones that were not deleted.

## How to get to it (user POV)

- Click `＋ Activité` (`#act-open`) in the title bar, on `Aujourd'hui` or on `Graphiques`.
- Date: the chip `#act-chip` (`data-date`) opens the same calendar as the entry card.
- History: on `Aujourd'hui`, each filled day is a `.day` row. The badge is `.act-badge` (`data-date` is the ISO date). The open list is `.act-list`, and each `Supprimer` button is `.act-remove` (`data-id`).

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.

- **Open.** Run `dietlock-verify eval 'document.getElementById("act-open").click(); "opened"'`. `#act-overlay` is no longer hidden and `#act-save` is disabled with the text `Enregistrer`.
- **Fill three rows.** Run `dietlock-verify eval 'const t = (i, v) => { const s = document.querySelectorAll("#act-rows select")[i]; s.value = v; s.dispatchEvent(new Event("change")); }; t(0, "walk"); document.getElementById("act-add").click(); t(1, "strength"); document.getElementById("act-add").click(); t(2, "bike"); document.querySelectorAll("#act-rows .act-note")[0].value = "chien"; document.querySelectorAll("#act-rows .act-note")[0].dispatchEvent(new Event("input")); "filled"'`. `#act-save` reads `Enregistrer 3 activités`.
- **Save.** Run `dietlock-verify eval 'document.getElementById("act-save").click(); "submitted"'`. The overlay hides. `dietlock-verify db` shows `activity 3` and three rows `<today> walk chien`, `<today> strength`, `<today> bike`, and the `weigh_in` and `intake` counts are unchanged.
- **Refuse.** Open the form, type `x` in the first note without choosing a type, click `#act-save`. Wait for `#act-error` to contain `Choisis un type pour chaque activité.` The overlay stays open and `db` gains no row.
- **Cancel.** Open the form, choose a type, click `#act-cancel`. The overlay hides and `db` gains no row. Esc does the same.
- **Other day.** Click `#act-chip`, then a past day in `#date-pop`. `#act-chip`'s `data-date` changes. A future day shows the French future-date message in `#act-error`.
- **Restart.** Run `dietlock-verify relaunch`, then `dietlock-verify db`. The `activity` rows are the same, with the same dates.
- **Badge.** Give today a weigh-in and two activities (Marche with a note, Muscu with none), and give yesterday a weigh-in and no activity. Today's `.day` contains `.act-badge` reading `2 activités`. Yesterday's `.day` has no `.act-badge`.
- **Tooltip.** Move the pointer onto `.act-badge`. The tooltip lists `Marche · chien`, `Muscu`, and `Clic pour gérer`. There is no `·` on the row that has no note.
- **List.** Click `.act-badge`. `.act-list` appears under that row, with the type, the note or `sans note`, and `Supprimer`. Click the same badge again: the list is gone and the badge remains. `#delete-confirm` stays hidden.
- **Delete one.** Open the list and click the first `.act-remove`. That activity disappears from the list and from `db`. The other activity, the weigh-in and the intake of that date stay. The badge reads `1 activité`.
- **Delete the last.** Click the remaining `.act-remove`. The badge and the list are gone. `db` has no activity for that date. The weigh-in row is still there.
- **Figures.** Read `#headline`, `#subhead` and `#energy` before adding activities, after adding them, and after deleting them. The three texts are the same each time. On `Graphiques`, `#macro-protein-figure` is the same as well.
- **Deleted stays gone.** Leave one activity stored and delete another, then `dietlock-verify relaunch`. `db` still has the kept row and not the deleted one. The badge matches what remains.
- **Proof.** Screenshots of the badge against a day with none, the tooltip, the open list, the list after one deletion, the day after the last deletion, the top figures before and after, the same screen after `relaunch`, and the badge and the list in the dark theme (`GTK_THEME=Adwaita:dark`).

## Gotchas

- The title bar is a window drag region on macOS. Whether `＋ Activité` is clickable without starting a drag, and clears the traffic lights, cannot be proved on Linux. Check it on a real Mac.
- `select` changes need a `change` event. Setting `.value` alone leaves the count at 0.
- `Enregistrer` is disabled only while every row is empty. A note without a type enables it, and the click shows the refusal.
- Saving the same date twice appends. Two saves give two sets of rows, not one.
- `Supprimer` on an activity does not open `#delete-confirm`. The click returns before `list_days` has redrawn the row. Wait until the badge text or `db` changes.
- The tooltip is CSS `:hover`. An `eval` click does not show it. Move the pointer onto the badge.
- `relaunch` keeps the theme of the original `launch`. A dark-theme proof is a new `launch` with `GTK_THEME=Adwaita:dark`.
- An activity older than the 30-day history stays in the database and has no badge.
