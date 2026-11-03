# DietLock verification map

This directory is the maintained source for verifying the user-facing behavior of DietLock. Read the index before driving the app, then use the matching feature file as the recipe.

The window has two screens, `Aujourd'hui` and `Poids`. Nothing leaves the window.

## Baseline preconditions

- Run `dietlock-verify launch` and wait until it prints `ready`.
- The data dir is the empty store that launch created under `/tmp/dietlock-verify/<run-id>/`. Do not point the app at a real `com.bertrand2808.dietlock` directory.
- Run `dietlock-verify doctor` and require `doctor PASS` before the first drive. Schema `user_version` is 2.
- Never drive an instance that this run did not start. Launch refuses a second live instance.
- UI copy is French, including validation messages.
- Frontend tests do not open the window: `dietlock-verify frontend-test`. Rust proof is `dietlock-verify cargo-test`.

## Driving conventions

- Start every recipe from the baseline state unless its preconditions say otherwise.
- Use the element ids in the feature file. Dates live on `#weigh-chip` and `#intake-chip` (`data-date`), not on a date input.
- Treat every command as literal. Keep quoted strings and flags unchanged.
- Run actions through `dietlock-verify eval`. Read the screen with `dietlock-verify text`.
- Read stored rows with `dietlock-verify db` after a save or a cancelled replacement.
- Poll `text` after `#save` or after opening `Poids`. Both finish after the expression returns.
- Every pull request includes its proof in the PR body: the screenshot and check output, or the test output.

## Proof and skip reporting

- Capture the user action and the resulting state, not only the final screen.
- UI proof for the landing journey is `check-today`: a screenshot of `Aujourd'hui` plus the DOM checks, including the `Poids` screen opened afterwards. The PNG stays on `Aujourd'hui`.
- Command-line proof is `cargo-test.txt` or `frontend-test.txt`: the command, stdout, stderr, and the exit code.
- Mutation proof includes `dietlock-verify db` as well as the history row.
- Record the feature id and the entry point with the artifact.
- Report an unreachable path with the command you ran and the unmet precondition.
- Do not report a skipped entry point as verified through a different path.

## Feature entry contract

Each feature file starts with an H1 title and one paragraph describing the user-visible behavior. It then uses exactly four H2 sections in this order.

1. `Sub-features` lists short IDs with one line for each behavior.
2. `How to get to it (user POV)` lists every user entry point.
3. `Driving it with dietlock-verify` starts with `Preconditions:` and uses labeled bullets that pair each user action with an exact command and observable result.
4. `Gotchas` lists traps that can waste or invalidate a verification run.

Keep implementation details out of the map. Name only user paths, stable handles, required state, commands, and observable proof.

## Features

- [Today's screen](./today.md) covers `Aujourd'hui` on an empty store: greeting, chips, empty history, and the screenshot check.
- [Weight curve](./weight.md) covers the `Poids` screen, the 15-day empty state, and the 30-day switch.
- [Weigh-in](./weigh-in.md) covers saving today's fasted weight from the entry card.
- [Intake](./intake.md) covers saving macros and optional water from the same card.
- [Replace a day](./replace.md) covers the confirmation when that date already has a weigh-in or an intake.
