# Macro week

The `Graphiques` screen shows, under the weight curve, a `Protéines` block with a bar chart of the 7 days (see `macro-bars` below) and: the mean protein per day of the 7 days ending yesterday, how many of those days were entered, and a pill for the signed gap with the 7 days before. A day without macros is left out. A day entered at 0 g counts. No intake in the window is `—`, not 0. No intake in the previous week hides the pill. A gap of 0 is the pill `0 vs semaine d'avant`.

## Sub-features

- `macro-period` is the selector `#macro-seg` with two buttons, `[data-macro-span="7"]` (`7 j`, pressed on launch) and `[data-macro-span="15"]` (`15 j`). It sits above the macro blocks. Clicking `15 j` presses it, sets `#macro-protein-span` to `15 derniers jours, hier compris`, and redraws the block for 15 days. It never changes the weight selector `#span-seg`, and `#span-seg` never changes it.
- `macro-period-wording` in 15 days names the previous period in the pill: `+12 vs 15 jours d'avant`. In 7 days it stays `vs semaine d'avant`.
- `macro-bars-15` in 15 days draws 15 `text.mc-x` labels that are the day number only, no `text.mc-val`, and a `rect.mc-bar` per entered day. The empty sentence reads `Aucune saisie sur ces 15 jours.`

- `macro-mean` shows `#macro-protein-figure` as `N g/j` and `#macro-protein-note` ending with `N jours saisis`. When a pill is shown, it comes first in the note.
- `macro-gap` shows `#macro-protein-pill` such as `+12 vs semaine d'avant` when the previous week has intakes. Class `up` when positive, `down` when negative, `even` when 0.
- `macro-no-comparison` shows no pill when the previous week is empty.
- `macro-none` shows `—` when the window has no intake. The figure has no digit.
- `macro-bars` shows `#macro-protein-chart` with one `rect.mc-bar` per day entered, oldest on the left, and one `text.mc-x` label per day of the window (`lun. 9`).
- `macro-hole` leaves no bar for a day without intake. A day entered at 0 g has a `rect.mc-bar` of height 2 and a `text.mc-val` reading `0 g`.
- `macro-mean-line` draws one `line.mc-mean` at the displayed mean, with the label `text.mc-mean-lbl` (`moyenne N g`). Both are absent when the window has no intake.
- `macro-hover` gives each bar a `title` such as `lun. 9 · 142,5 g`.
- `macro-chart-empty` replaces the chart by `Aucune saisie sur ces 7 jours.` (a `p.empty`, no `svg`) when the window has no intake.
- `macro-today-excluded` ignores today's intake.
- `macro-refresh` updates the block after an add, a replacement or a deletion, once `Graphiques` is shown again.
- `macro-reopen` shows the same figures after the app is closed and opened again on the same data.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- Read today from `#weigh-chip`'s `data-date`. The window is the 7 days ending yesterday.

- **Read an empty window.** Open `Graphiques` with `eval 'document.getElementById("tab-weight").click()'`. `#macro-protein-figure` is `—` and `#macro-protein-note` is empty.
- **Enter days.** Enter intakes the way `energy-week.md` does, with known protein.
- **Read the mean.** `#macro-protein-figure` is the mean in whole grams plus ` g/j`. `#macro-protein-note` ends with `N jours saisis` or `1 jour saisi`.
- **Read the gap.** `#macro-protein-pill` is absent when the previous week is empty.
- **Reopen.** Run `dietlock-verify relaunch`, open `Graphiques`, and read the same figures.
- **Proof.** One screenshot of `Graphiques` plus the `text` output and `db`.
