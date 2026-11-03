# Weigh-in

A weigh-in is the body weight for one day, in kilograms, marked fasted or not. It is typed in the top row of the `Aujourd'hui` card and saved with the `↑` button. The history then shows that day. The greeting stays until four fasted weigh-ins exist in the seven-day window; once any day is saved and there is still no trend, the big line is empty and the subtitle still says the trend is waiting.

## Sub-features

- `weigh-in-save-fasted` saves today's fasted weight and shows `82,6 kg` on that day's row.
- `weigh-in-save-fed` shows `pas à jeun` on the row when the box is unchecked.
- `weigh-in-past` saves another day by picking it on the date chip first.
- `weigh-in-refuse` shows a French message and writes nothing when the weight is not acceptable.

## How to get to it (user POV)

- Stay on `Aujourd'hui`. The row is the date chip `Date de la pesée`, the kg field, and `à jeun`. The `↑` button saves it.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- The chip date is today or earlier. Read it from `#weigh-chip`'s `data-date`.
- That date has no weigh-in yet. A filled date is the replace journey.

- **Enter today's fasted weight.** Leave `à jeun` checked. Run `dietlock-verify eval 'document.getElementById("weight").value = "82,6"; document.getElementById("fasted").checked = true; document.getElementById("save").click(); "submitted"'`.
- **See the result.** Poll `dietlock-verify text` until it contains `82,6 kg` and `✓ Journée enregistrée`. The history row's date is the French label (`mar. 3 nov.`), not `YYYY-MM-DD`. `#weight` is empty again.
- **Confirm the row.** Run `dietlock-verify db`. One `weigh_in` row has that ISO date, `weight_tenth_kg` `826`, and `fasted` `1`.
- **Not fasted.** Uncheck `#fasted`, set another empty day, and save `81.0`. The row's detail contains `pas à jeun`. A dot in the field is accepted; the screen prints `81,0 kg`.
- **Refuse a bad weight.** Save `abc`. `#weigh-error` is visible and contains `n'est pas un poids`. `db` does not gain a row.
- **Proof.** Screenshot after the history row is visible, and keep the `db` line.

## Gotchas

- Click `#save`. There is no `Enregistrer` button and no `#weigh-in-form` submit. Submitting `#entry` does nothing.
- Leaving the weight field empty skips the weigh-in. The click can still save an intake if those fields are filled.
- The expression returns `submitted` before the row exists. Wait for `82,6 kg` or for `#saved` to show `✓ Journée enregistrée`.
- After this first save the headline is empty. The subtitle remains `Pas encore de saisie. La tendance apparaîtra après 4 pesées à jeun.` Do not treat the missing greeting as a failed save.
- A future date is refused in French and nothing is written. Pick the day from the chip (`#weigh-chip`, then a day in `#date-pop`) rather than typing an ISO date into a field that is not there.
