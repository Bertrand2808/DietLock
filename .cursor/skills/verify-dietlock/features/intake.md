# Intake

An intake is the protein, carbohydrate, and fat of one day, in whole grams, plus optional water in litres. The screen shows the energy in kcal. The user never types the energy.

## Sub-features

- `intake-save` saves the three macros and shows energy on that day's history line.
- `intake-water` appends the water in litres when the water field is filled.
- `intake-no-water` omits water when the field is left empty.
- `intake-refuse` shows an English message and writes nothing when a macro is empty or not a whole number of grams.

## How to get to it (user POV)

- Use the `Alimentation` form. The fields are `Protéines (g)`, `Glucides (g)`, `Lipides (g)`, `Eau (L, facultatif)`, and `Date`, then `Enregistrer`.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- Read `#intake-date` after launch and use that value, or set it to a past or current `YYYY-MM-DD`. Do not assume it equals `#date`.
- That date has no intake yet. A filled date is the replace journey.

- **Enter macros and water.** Run `dietlock-verify eval 'document.getElementById("protein").value = "150"; document.getElementById("carbs").value = "250"; document.getElementById("fat").value = "70"; document.getElementById("water").value = "1,5"; document.getElementById("intake-form").requestSubmit(); "submitted"'`.
- **See the result.** Poll `dietlock-verify text` until that date's line contains `2230 kcal (P 150 g, G 250 g, L 70 g, eau 1,5 L)`. The four intake fields are empty again. Energy is `4*150 + 4*250 + 9*70`.
- **Confirm the row.** Run `dietlock-verify db`. The `intake` row has `150 250 70` and `water_ml` `1500`.
- **Leave water empty.** Submit `150`, `250`, `70`, and an empty `#water` on another empty day. The line contains `2230 kcal (P 150 g, G 250 g, L 70 g)` and does not contain `eau`. `water_ml` in `db` is `None`.
- **Refuse an empty macro.** Clear the fields and submit. `#error` contains `Protein is required.` `db` does not gain a row.
- **Proof.** Screenshot the window once the kcal text is visible, and keep the `db` line that shows `1500` or `None`.

## Gotchas

- Submit `#intake-form`. The weigh-in form's button has the same label.
- The proposed intake date can be yesterday. Before 04:00 it is yesterday. From 04:00 until noon, if neither yesterday nor today has an intake, it is yesterday too. After a save, relaunch or re-read `#intake-date` before assuming today.
- Nothing on the screen highlights that yesterday was chosen. Only the date field's value changes.
- Macros are whole grams. `12,5` is refused with a message that contains `Fat` and `whole grams` when it was typed in the fat field. Water accepts `1,5` and `1.5`; the screen prints `1,5`.
- Water above `10,0` L is refused. A macro above `1000` g is refused.
- The weigh-in half of the same history line stays `pas de pesée` when no weigh-in was saved.
