# Macro period

The `Graphiques` screen shows, under the weight curve, a `Macros` header (`#macro-title`) with a `7 j` / `15 j` selector (`#macro-seg`), then three blocks for the selected period: `Protéines`, `Glucides`, `Lipides`, in that order. Each block has the mean of its macro per day of the period ending yesterday, how many of those days were entered, a pill for the signed gap with the period before, and a bar chart of the days. A day without macros is left out. A day entered at 0 g counts. No intake in the period is `—`, not 0. No intake in the previous period hides the pill. A gap of 0 is the pill `0 vs semaine d'avant` (7 j) or `0 vs 15 jours d'avant` (15 j). The period is 7 j on launch.

## Sub-features

The ids below are the protein block's. The carbs block uses `#macro-carbs-…` and the fat block `#macro-fat-…`, with the same suffixes (`figure`, `note`, `pill`, `chart`, `span`).

- `macro-three-blocks` shows `#macro-protein`, `#macro-carbs` and `#macro-fat` stacked in that order, under `#macro-seg`. Clicking `15 j` redraws all three. With an intake of 150 / 200 / 60 g on yesterday, the figures read `150 g/j`, `200 g/j` and `60 g/j`, with no mix between the blocks. The screen scrolls to reach the last block.

- `macro-period` is the selector `#macro-seg` with two buttons, `[data-macro-span="7"]` (`7 j`, pressed on launch) and `[data-macro-span="15"]` (`15 j`). Clicking `15 j` presses it, sets `#macro-protein-span` to `15 derniers jours, hier compris`, and redraws the block for 15 days. It never changes the weight selector `#span-seg`, and `#span-seg` never changes it.
- `macro-mean` shows `#macro-protein-figure` as `N g/j` and `#macro-protein-note` ending with `N jours saisis`. When a pill is shown, it comes first in the note.
- `macro-gap` shows `#macro-protein-pill` such as `+12 vs semaine d'avant` (7 j) or `+12 vs 15 jours d'avant` (15 j) when the previous period has intakes. Class `up` when positive, `down` when negative, `even` when 0.
- `macro-no-comparison` shows no pill when the previous period is empty.
- `macro-none` shows `—` when the period has no intake. The figure has no digit.
- `macro-bars` shows `#macro-protein-chart` with one `rect.mc-bar` per day entered, oldest on the left, and one `text.mc-x` label per day of the period. In 7 j the label is `lun. 9`. In 15 j it is the day number only (`9`).
- `macro-hole` leaves no bar for a day without intake. A day entered at 0 g has a `rect.mc-bar` of height 2. In 7 j it also has a `text.mc-val` reading `0 g`. In 15 j there is no `text.mc-val`; the value is in the `title`.
- `macro-mean-line` draws one `line.mc-mean` at the displayed mean, with the label `text.mc-mean-lbl` (`moyenne N g`). Both are absent when the period has no intake.
- `macro-hover` gives each bar a `title` such as `lun. 9 · 142,5 g`, in 7 j and in 15 j.
- `macro-chart-empty` replaces the chart by `Aucune saisie sur ces 7 jours.` or `Aucune saisie sur ces 15 jours.` (a `p.empty`, no `svg`) when the period has no intake.
- `macro-today-excluded` ignores today's intake.
- `macro-refresh` updates the block after an add, a replacement or a deletion, once `Graphiques` is shown again.
- `macro-reopen` shows the same figures after the app is closed and opened again on the same data. The period is back on 7 j.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- Read today from `#weigh-chip`'s `data-date`. The period is the 7 or 15 days ending yesterday, and the previous period is the same number of days before it.

- **Read an empty period.** Open `Graphiques` with `eval 'document.getElementById("tab-weight").click()'`. `#macro-protein-figure` is `—` and `#macro-protein-note` is empty.
- **Enter days.** Enter intakes the way `energy-week.md` does, with known protein.
- **Read the mean.** `#macro-protein-figure` is the mean in whole grams plus ` g/j`. `#macro-protein-note` ends with `N jours saisis` or `1 jour saisi`.
- **Read the gap.** `#macro-protein-pill` is absent when the previous period is empty.
- **Switch to 15 days.** Run `eval 'document.querySelector("[data-macro-span=\"15\"]").click()'`. Then read `#macro-protein-span`, `#macro-protein-figure`, the pill wording, and count `rect.mc-bar` and `text.mc-x` in `#macro-protein-chart`. Check that `#span-seg` still has its own pressed button.
- **Reopen.** Run `dietlock-verify relaunch`, open `Graphiques`, and read the same figures. The selector is back on `7 j`.
- **Proof.** One screenshot of `Graphiques` in 7 j and one in 15 j, plus the `text` output and `db`.
