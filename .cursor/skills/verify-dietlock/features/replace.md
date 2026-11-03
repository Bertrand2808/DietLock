# Replace a day

A day holds one weigh-in and one intake. Saving again on a date that already has that entry asks before writing. The question names the day in French (`mar. 3 nov.`) and shows the current value. Cancel leaves it in place. Replace overwrites it.

## Sub-features

- `replace-ask-weigh-in` asks, and writes nothing, when the date already has a weigh-in.
- `replace-ask-intake` asks, and writes nothing, when the date already has an intake.
- `replace-cancel` hides the question and keeps the stored value.
- `replace-confirm` overwrites that entry and updates the history row.

## How to get to it (user POV)

- On `Aujourd'hui`, type a new weight or new macros for a date that already has that entry, then press `↑`.
- The question appears in the card, with `Remplacer` and `Annuler`.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- The date already has the entry you are about to submit. Save it once through `weigh-in.md` or `intake.md`, and record the `db` row.
- `#confirm` is hidden. Run `dietlock-verify eval 'document.getElementById("confirm").hidden'` and expect `true`.

- **Ask about a weigh-in.** Run `dietlock-verify eval 'document.getElementById("weight").value = "81,2"; document.getElementById("save").click(); "submitted"'`.
- **See the question.** Poll `dietlock-verify text` until it contains `a déjà une pesée` and `La remplacer par 81,2 kg`. The day in the sentence is the French label, such as `mar. 3 nov.` `#confirm` is not hidden. The history row still shows the previous weight.
- **Confirm nothing was written.** Run `dietlock-verify db`. The `weigh_in` row is still the previous `weight_tenth_kg`.
- **Cancel.** Run `dietlock-verify eval 'document.getElementById("confirm-cancel").click(); "cancelled"'`. `#confirm` is hidden. `db` is unchanged.
- **Replace.** Submit `81,2` again, then run `dietlock-verify eval 'document.getElementById("confirm-replace").click(); "replaced"'`. The history row shows `81,2 kg` and `db` shows `weight_tenth_kg` `812`.
- **Ask about an intake.** With an intake already stored, submit a new set of macros. The question contains `a déjà une alimentation` and `La remplacer par`. Cancel keeps the previous kcal text. `Remplacer` updates the row and the `intake` row.
- **Proof.** Screenshot while the question is visible, and keep the `db` output from before `Remplacer`. Screenshot again after `Remplacer` if the PR claims the overwrite.

## Gotchas

- The first save on an empty date does not ask. It writes immediately.
- A weigh-in and an intake on the same date do not block each other.
- `#save` does nothing while the question is open. Use `#confirm-replace` or `#confirm-cancel`.
- `Annuler` only dismisses the question. It does not clear what you typed.
- The question uses the French day label from the chip, not the ISO date.
- Saving both a new weigh-in and a new intake when each date is already filled asks twice, one question after the other.
