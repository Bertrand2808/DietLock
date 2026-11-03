# Weight curve

`Poids` shows the smoothed weight and the daily weigh-ins. With no weigh-in, the screen says the curve is not there yet and counts `aucune pesée` over the last 15 days.

## Sub-features

- `weight-open` opens `Poids` from the title-bar tab. `Aujourd'hui` stays in the window, hidden.
- `weight-empty` shows `Pas encore de courbe.` and the chart sentence `Ta courbe se dessinera ici.`
- `weight-span` switches the window between `15 j` and `30 j` without leaving the screen.

## How to get to it (user POV)

- Choose the `Poids` tab.
- Press `Ctrl+2` on Linux (`⌘2` on macOS) while focus is outside a field. `Ctrl+1` returns to `Aujourd'hui`.

## Driving it with dietlock-verify

Preconditions:

- `dietlock-verify doctor` printed `doctor PASS`.
- For the empty state, `dietlock-verify db` prints `weigh_in 0`.

- **Open the screen.** Run `dietlock-verify eval 'document.getElementById("tab-weight").click(); "opened"'`. Poll `dietlock-verify eval 'document.getElementById("w-headline").textContent'` until it is `Pas encore de courbe.`
- **Read the empty state.** `#w-sub` is `La tendance apparaîtra après 4 pesées à jeun.` `#w-range` is `15 derniers jours`. `#w-count` is `aucune pesée`. `#chartbox` contains `Ta courbe se dessinera ici.` `#tab-weight` has `aria-selected="true"` and `#screen-today` is hidden.
- **Switch to 30 days.** Run `dietlock-verify eval 'document.querySelector("#span-seg button[data-span=\\"30\\"]").click(); document.getElementById("w-range").textContent'`. The text is `30 derniers jours` and the count is still `aucune pesée`.
- **Proof.** `dietlock-verify check-today` opens this screen after it has captured `Aujourd'hui`, and fails if the empty curve text is missing. For a proof that is the curve itself, run `dietlock-verify screenshot` once `#w-headline` reads `Pas encore de courbe.`

## Gotchas

- The headline stays empty for a moment while `weight_curve` loads. Wait for the sentence, not for the click to return.
- One weigh-in replaces `Pas encore de courbe.` with that weight (`82,6 kg ce matin` when it is today) and still says the trend needs four fasted weigh-ins. The empty sentence is only for zero rows.
- `15 j` is the default. A check that expects `30 derniers jours` before the click is looking at the wrong span. The history on `Aujourd'hui` also says `30 derniers jours`, and only after a day is saved.
- The keyboard shortcut is `Ctrl` on this Linux VM. Do not send `⌘`.
