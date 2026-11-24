# Design spec — DietLock
Updated: 2026-11-24

## User journey
- Report: one click on « Rapport » in the title bar copies a 15-day image to the clipboard; the user pastes it into a message to their nutritionist.

## Screens (link to the chosen mockup)
- Report, option B: [DEV-85](https://app.notion.com/p/412449738dcc81e48167f11ba35613af); screenshots in the implementing PR.

## Components
- « Rapport » button: in the title bar, to the left of « ＋ Activité », same style as « ＋ Activité ».
- Report image, top to bottom: title with the date range, « Tendance sur 15 jours » box, weight and Trend curve, one card per day, footer note.
- « Tendance sur 15 jours » box: Trend of J−14 → Trend of today in kg, and « Écart » with its sign (e.g. « 109,7 → 108,3 kg · Écart −1,4 kg »).
- Curve: weight points and the Trend line over the 15 days, sharing the scale of the « Graphiques » screen; a day without Trend breaks the line.
- Day card, at most two lines: date | weight with « pas à jeun » when not fasted | kcal and P/G/L | water on the right; Activities (type and note) on the second line.
- Day cards never show a Trend or a per-day change.
- Footer note: an empty cell means nothing was entered; energy = 4P + 4G + 9L.
- Confirmation: after the copy the button reads « ✓ Copié » briefly, and a toast « Rapport copié · 15 jours, du … au … » shows a thumbnail of the image.

## States (empty, loading, error, disabled)
- A day or value without input leaves its cell empty, never « 0 » nor « — ».
- The 15-day box is hidden when either end has no Trend.
- Copy refused: the button never shows « ✓ Copié », and a failure message is shown instead.

## Style guide (colors, font, spacing)
- The report image is light-only and colourless: no red/green for gains or losses, no Alert colour, whatever the app theme.
- Image width 1080 px; text 10 to 11.6 pt once opened full-width on a phone.

## Accessibility
- Every value in the image is written as text with its unit; nothing is carried by colour alone.

