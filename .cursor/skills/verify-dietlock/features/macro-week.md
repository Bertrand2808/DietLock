# Macro week

The `Graphiques` screen shows, under the weight curve, a `Protéines` block: the mean protein per day of the 7 days ending yesterday, how many of those days were entered, and a pill for the signed gap with the 7 days before. A day without macros is left out. A day entered at 0 g counts. No intake in the window is `—`, not 0. No intake in the previous week hides the pill. A gap of 0 is the pill `0 vs semaine d'avant`.

## Sub-features

- `macro-mean` shows `#macro-protein-figure` as `N g/j` and `#macro-protein-note` as `N jours saisis`.
- `macro-gap` shows `#macro-protein-pill` such as `+12 vs semaine d'avant` when the previous week has intakes. Class `up` when positive, `down` when negative, `even` when 0.
- `macro-no-comparison` shows no pill when the previous week is empty.
- `macro-none` shows `—` when the window has no intake. The figure has no digit.
- `macro-today-excluded` ignores today's intake.
- `macro-refresh` updates the block after an add, a replacement or a deletion, once `Graphiques` is shown again.
- `macro-reopen` shows the same figures after the app is closed and opened again on the same data.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- Read today from `#weigh-chip`'s `data-date`. The window is the 7 days ending yesterday.

- **Read an empty window.** Open `Graphiques` with `eval 'document.getElementById("tab-weight").click()'`. `#macro-protein-figure` is `—` and `#macro-protein-note` is empty.
- **Enter days.** Enter intakes the way `energy-week.md` does, with known protein.
- **Read the mean.** `#macro-protein-figure` is the mean in whole grams plus ` g/j`. `#macro-protein-note` starts with `N jours saisis` or `1 jour saisi`.
- **Read the gap.** `#macro-protein-pill` is absent when the previous week is empty.
- **Reopen.** Run `dietlock-verify relaunch`, open `Graphiques`, and read the same figures.
- **Proof.** One screenshot of `Graphiques` plus the `text` output and `db`.
