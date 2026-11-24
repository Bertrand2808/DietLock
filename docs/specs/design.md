# Design spec — DietLock
Updated: 2026-11-24

## User journey
- Report: the user opens the « Rapport » tab, checks the 15-day image, clicks « Copier », then pastes the image into a message. On the Mac they send it to their phone and post it in a nutrition app, where their nutritionist opens it full-screen on a phone.

## Screens (link to the chosen mockup)
- Report, option B: [DEV-85](https://app.notion.com/p/412449738dcc81e48167f11ba35613af); screenshots in the implementing PR.

## Components
- « Rapport » tab: third tab of the central selector, after « Aujourd'hui » and « Graphiques ». The title bar has no report button.
- « Copier » button: at the top of the « Rapport » page, always visible while the page scrolls.
- Report page: the image is shown unscaled at 480 px, centred, so what the user sees is what is copied; the page scrolls.
- Report image, top to bottom: title with the date range, « Tendance sur 15 jours » box, summary block, weight and Trend curve, one card per day, footer note. One portrait column, one image.
- Summary block: mean per day of kcal, P, G, L and water, each with « n jours saisis », and the number of days with an Activity. A mean with no counted day reads « — » (the one place « — » is allowed).
- « Tendance sur 15 jours » box: Trend of J−14 → Trend of today in kg, and « Écart » with its sign (e.g. « 109,7 → 108,3 kg · Écart −1,4 kg »).
- Curve: weight points and the Trend line over the 15 days, sharing the scale of the « Graphiques » screen; a day without Trend breaks the line. A kg scale on the left gives three marks (top, middle, bottom) with a light line at each, so the reader can place the points; the scale is absent when no weight was entered in the 15 days.
- Day card, three short lines: date, weight and « pas à jeun » when not fasted; kcal and P/G/L; water and Activities (type and note).
- Day cards never show a Trend or a per-day change.
- Footer note: an empty cell means nothing was entered; energy = 4P + 4G + 9L.
- Confirmation: after the copy the button reads « ✓ Copié » for 8 s, then goes back to « Copier ». A text toast « Rapport copié · 15 jours, du … au … » floats at the bottom right for 8 s, with no thumbnail. Hovering pauses the 8 s, a click closes the toast, a new copy restarts it. The toast never moves the page content.

## States (empty, loading, error, disabled)
- A day or value without input leaves its cell empty, never « 0 » nor « — ».
- The 15-day box is hidden when either end has no Trend.
- Copy refused: the button never shows « ✓ Copié », and a failure message is shown instead.

## Style guide (colors, font, spacing)
- The report image is light-only and colourless: no red/green for gains or losses, no Alert colour, whatever the app theme.
- Image 480 px wide, exported at ×2 (960 px); text at least 15 px (title 22 px), so it stays readable when opened full-width on a phone.

## Accessibility
- Every value in the image is written as text with its unit; nothing is carried by colour alone.

## Decision journal
- 2026-11-24 · The report moves to its own tab, with a phone-first portrait image and a floating text toast (DEV-85).

