# Activities

Activities are the physical activities of one day, entered together in a modal form opened by the `＋ Activité` button in the title bar. The button is visible on both screens and is separate from the entry card. One save writes every typed row for the chosen date. The weigh-in and the intake of that date are never touched. Nothing on screen lists saved activities yet, so the proof is `dietlock-verify db`.

## Sub-features

- `activity-open` opens the form with today's date chip and one empty row.
- `activity-save` saves several rows for the chosen date in one click and closes the form.
- `activity-refuse` keeps the form open with a French message when a row has no type.
- `activity-cancel` closes the form without writing, through `Annuler` or Esc.
- `activity-restart` shows the rows are still stored for the right date after `relaunch`.

## How to get to it (user POV)

- Click `＋ Activité` (`#act-open`) in the title bar, on `Aujourd'hui` or on `Poids`.
- Date: the chip `#act-chip` (`data-date`) opens the same calendar as the entry card.

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
- **Proof.** Screenshot of the open form with three rows, the `db` output before and after `relaunch`.

## Gotchas

- The title bar is a window drag region on macOS. Whether `＋ Activité` is clickable without starting a drag, and clears the traffic lights, cannot be proved on Linux. Check it on a real Mac.
- `select` changes need a `change` event. Setting `.value` alone leaves the count at 0.
- `Enregistrer` is disabled only while every row is empty. A note without a type enables it, and the click shows the refusal.
- Saving the same date twice appends. Two saves give two sets of rows, not one.
