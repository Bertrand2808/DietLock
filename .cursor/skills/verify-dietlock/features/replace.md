# Replace a day

A day holds one weigh-in and one intake. Saving again on a date that already has that entry asks before writing. The question shows the current value. Cancel leaves it in place. Replace overwrites it.

## Sub-features

- `replace-ask-weigh-in` asks, and writes nothing, when the date already has a weigh-in.
- `replace-ask-intake` asks, and writes nothing, when the date already has an intake.
- `replace-cancel` hides the question and keeps the stored value.
- `replace-confirm` overwrites that entry and updates the history line.

## How to get to it (user POV)

- Submit the `Pesée` form or the `Alimentation` form for a date that already has that entry.
- The question appears under the forms, with `Remplacer` and `Annuler`.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- The date already has the entry you are about to submit. Save it once through `weigh-in.md` or `intake.md`, and record the `db` row.
- `#confirm` is hidden. Run `dietlock-verify eval 'document.getElementById("confirm").hidden'` and expect `true`.

- **Ask about a weigh-in.** Submit a new weight on that date with `replace` left to the form (do not call the command yourself). Run `dietlock-verify eval 'document.getElementById("weight").value = "81,2"; document.getElementById("weigh-in-form").requestSubmit(); "submitted"'`.
- **See the question.** Poll `dietlock-verify text` until it contains `a déjà une pesée` and `La remplacer par 81,2 kg`. `#confirm` is not hidden. The history line still shows the previous weight.
- **Confirm nothing was written.** Run `dietlock-verify db`. The `weigh_in` row is still the previous `weight_tenth_kg`.
- **Cancel.** Run `dietlock-verify eval 'document.getElementById("confirm-cancel").click(); "cancelled"'`. `#confirm` is hidden. `db` is unchanged.
- **Replace.** Submit `81,2` again, then run `dietlock-verify eval 'document.getElementById("confirm-replace").click(); "replaced"'`. The history line shows `81,2 kg` and `db` shows `weight_tenth_kg` `812`.
- **Ask about an intake.** With an intake already stored, submit a new set of macros. The question contains `a déjà une alimentation` and `La remplacer`. Cancel keeps the previous kcal text. `Remplacer` updates the line and the `intake` row.
- **Proof.** Screenshot while the question is visible, and keep the `db` output from before `Remplacer` showing the old row. Screenshot again after `Remplacer` if the PR claims the overwrite.

## Gotchas

- The first save on an empty date does not ask. It writes immediately. The question only follows a second save of the same entry.
- A weigh-in and an intake on the same date do not block each other. Replacing a weigh-in leaves the intake row as it was.
- `Annuler` only hides the question. It does not clear the weight you typed.
- The question names the previous weight with a comma (`82,6 kg`) and repeats the new text as typed (`81,2`).
- Click `#confirm-replace` or `#confirm-cancel`. Do not submit the form again to mean "yes": another submit while the question is open starts a new ask.
- Leaving the question open and then saving the other form closes it. Read `#confirm` immediately after the submit you are proving.
