# Intake

An intake is the protein, carbohydrate, and fat of one day, plus optional water. They are typed on the second row of the `Aujourd'hui` card (P, G, L, Eau) and saved with the same `↑` button. The screen shows energy in kcal. The user never types it. The history row does not repeat the grams.

## Sub-features

- `intake-save` saves the three macros and shows the kcal on that day's row and under the headline.
- `intake-water` adds `eau 1,5 L` on the row when the water field is filled.
- `intake-no-water` omits water when the field is left empty.
- `intake-refuse` shows a French message and writes nothing when a macro is missing or not a number of grams.

## How to get to it (user POV)

- Stay on `Aujourd'hui`. The row is the date chip `Date de l'alimentation`, then `P`, `G`, `L`, and `Eau`.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- Read `#intake-chip`'s `data-date` and use that day, or pick another past or current day from the chip. Do not assume it equals the weigh-in chip.
- That date has no intake yet. A filled date is the replace journey.

- **Enter macros and water.** Run `dietlock-verify eval 'document.getElementById("protein").value = "150"; document.getElementById("carbs").value = "250"; document.getElementById("fat").value = "70"; document.getElementById("water").value = "1,5"; document.getElementById("save").click(); "submitted"'`.
- **See the result.** Poll `dietlock-verify text` until it contains `2 230 kcal` (narrow no-break space before `230`) and `eau 1,5 L`. The subtitle starts with `Aujourd'hui ·` or `Hier ·` and contains that kcal text. The four fields are empty. Energy is `4*150 + 4*250 + 9*70`.
- **Confirm the row.** Run `dietlock-verify db`. The `intake` row stores tenths of a gram: `1500 2500 700`, and `water_ml` `1500`.
- **Leave water empty.** Save `150`, `250`, `70`, and an empty `#water` on another empty day. The row contains the kcal text and does not contain `eau`. `water_ml` is `None`.
- **Refuse an empty macro.** Fill protein only and save. `#intake-error` contains `Les glucides sont obligatoires.` `db` does not gain a row.
- **Proof.** Screenshot once the kcal text is visible, and keep the `db` line.

## Gotchas

- Click `#save`. An empty weight field on the same card does not create a weigh-in.
- On an empty store the proposed intake date is yesterday all day. Before 04:00 the forgotten mark is hidden. From 04:00, including after noon, the chip shows `hier oublié` until yesterday has an intake.
- The history no longer prints `P 150 g, G 250 g, L 70 g`. Grams stay in the form. The row shows kcal and, when typed, water.
- `12,5` is a valid fat amount. The refusal for a word is a French sentence that contains `lipides`.
- A macro above 1000 g, or water above 10 L, is refused in French. Nothing is written.
