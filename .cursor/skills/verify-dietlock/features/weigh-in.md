# Weigh-in

A weigh-in is the body weight for one day, in kilograms, marked fasted or not. Saving it clears the weight field and shows the weight on that day's history line. The trend sentence does not change until four fasted weigh-ins exist in the seven-day window.

## Sub-features

- `weigh-in-save-fasted` saves today's fasted weight and shows it without the not-fasted note.
- `weigh-in-save-fed` saves a weight that was not fasted and shows `(pas à jeun)`.
- `weigh-in-past` saves a day that is already in the history by changing the date field first.
- `weigh-in-refuse` keeps the form and shows an English message when the weight or the date is not acceptable.

## How to get to it (user POV)

- Use the `Pesée` form at the top of the window. The fields are `Poids (kg)`, `À jeun`, and `Date`, then `Enregistrer`.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- The date you type is today or earlier. Read today from `#date` rather than guessing.
- That date has no weigh-in yet (`dietlock-verify db` does not list it). A filled date is the replace journey.

- **Enter today's fasted weight.** Type `82,6` and leave `À jeun` checked. Run `dietlock-verify eval 'document.getElementById("weight").value = "82,6"; document.getElementById("fasted").checked = true; document.getElementById("weigh-in-form").requestSubmit(); "submitted"'`.
- **See the result.** Poll `dietlock-verify text` until the first history line is `<date> : 82,6 kg | pas d'alimentation` (or the intake text already on that day). `#weight` is empty again. `#trend` is still `pas encore de tendance` when this is the only fasted weigh-in.
- **Confirm the row.** Run `dietlock-verify db`. One `weigh_in` row has that date, `weight_tenth_kg` `826`, and `fasted` `1`.
- **Not fasted.** Set `#fasted` to unchecked and submit `81.0` on an empty day. The history line contains `81,0 kg (pas à jeun)`. A dot in the field is accepted; the screen still prints a comma.
- **Past day.** Set `#date`.value to a `YYYY-MM-DD` that is already a history line and is not in the future, then submit. That line, not necessarily the first, gains the weight.
- **Refuse a bad weight.** Submit `abc`. `#error` is visible and contains `is not a weight`. `db` does not gain a row.
- **Proof.** Run `dietlock-verify screenshot` after the history line has updated. The PNG shows the typed weight on the day line. Keep the `db` output with the screenshot.

## Gotchas

- Submit `#weigh-in-form`, not a button named `Enregistrer`. The intake form has the same button name.
- The expression returns `submitted` before the line updates. Waiting on the history text is the ready signal.
- Displayed weights use a decimal comma (`82,6`) and one decimal. The stored integer is tenths of a kilogram (`826`).
- A future date is refused with `The date <day> is in the future.` The history does not change.
- Replacing an existing weigh-in does not happen in this journey. The confirmation dialog opens and `db` keeps the old row. Follow `replace.md`.
- Four fasted days are required before `#trend` becomes a weight. Do not treat an unchanged trend as a failed save.
