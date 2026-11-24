# Functional spec — DietLock
Updated: 2026-11-24

## Goal
- The user records their weight, macros and water in under 30 seconds, then sees whether they are on track: a weight trend, an alert on abnormal deviations, and their mean intake.

## Users
- One person, tracking their own body weight and nutrition, usually entering everything at once in the evening.

## Business rules

### Input
- S1 · The Weigh-in and the Intake are saved separately; either can exist without the other.
- S2 · A date holds at most one Weigh-in and one Intake.
- S3 · Replacing a stored Weigh-in, Macros or water requires confirmation showing the old value next to the new one.
- S3 · Submitting the same value writes nothing and asks nothing.
- S3 · Water added to an Intake that has none is written without confirmation and keeps the Macros.
- S3 · Blank Macros never replace stored Macros.
- S3 · Clearing the water of an Intake removes it after a confirmation stating water goes to 0; it requires the Macros typed, otherwise nothing changes.
- S4 · Protein, carbohydrate and fat are required together; water is optional.
- S5 · No field is pre-filled, except water, which holds the water already stored for the selected Intake date.
- S5 · The last Intake's values are shown as a hint only; an empty field is never saved.
- S6 · Weight in kg with one decimal, macros in g with at most one decimal, water in L with at most two decimals; comma and point are both accepted.
- S7 · Weight 30–250 kg, each macro 0–1,000 g, water 0–10 L; an out-of-range value is refused with a message stating the expected range.
- S8 · Energy = 4 × protein + 4 × carbohydrate + 9 × fat, rounded half away from zero to a whole kcal; it is never entered.
- S9 · The "fasted in the morning" flag is on by default.
- S10 · A Weigh-in or an Intake can be deleted after a confirmation showing its date and stored values.
- S10 · Deleting an Intake removes its Macros and its water together; deleting a Weigh-in recalculates the Trend (T5).

### Dates
- D1 · The Weigh-in is always proposed for today.
- D2 · Before the switch hour, the Intake is proposed for yesterday; from the switch hour to noon, for today unless D8 applies.
- D2 · From noon, the Intake is proposed for yesterday if yesterday has no Intake, otherwise for today.
- D3 · The switch hour defaults to 4:00, can be changed in settings, applies only to Intakes before noon, and never moves the noon boundary.
- D4 · The proposed date can always be changed before saving.
- D5 · No input is accepted for a date after today.
- D6 · Any past date can be entered or corrected, subject to S3.
- D7 · Changing the switch hour only affects future proposals; stored inputs never move.
- D8 · Between the switch hour and noon, if neither yesterday nor today has an Intake, the Intake is proposed for yesterday.
- D8 · From noon to midnight, if yesterday has no Intake, the Intake is proposed for yesterday even if today has one.
- D8 · The user is told whenever D8 proposes yesterday.

### Trend
- T1 · The Trend is computed from Weigh-ins only.
- T2 · The Trend of day J is the simple mean of valid Weigh-ins from J−6 to J, with no interpolation.
- T3 · A day has a Trend only if its window holds at least 4 valid Weigh-ins.
- T4 · After 7 days without a valid Weigh-in there is no Trend; it resumes at the 4th valid Weigh-in within one window.
- T5 · Adding, correcting, deleting a Weigh-in or changing its flag recalculates the Trend of the 7 affected days.
- T6 · A Weigh-in not fasted in the morning is saved and shown but excluded from the Trend and the Alert.
- T7 · A Weigh-in in Alert stays in the Trend.
- T8 · The 7-day and 4-week changes are computed from the Trend; a change is not shown if either end has no Trend.

### Energy mean
- E1 · The Energy mean covers J−7 to J−1 (today excluded); the previous week is J−14 to J−8.
- E2 · A day with an Intake counts, even at 0 kcal; a day without an Intake does not.
- E3 · With no Intake in the window there is no mean, which is not 0.
- E4 · The gap is this week's displayed mean minus last week's, in whole signed kcal; no previous Intake means no gap, a real tie is 0.
- E5 · The mean is the sum of daily Energy divided by N, rounded half away from zero; the gap subtracts the two displayed means.
- E6 · Adding, replacing or deleting an Intake refreshes the mean and the gap at once.

### Macro mean
- M1 · Each macro has its own mean over the selected period ending yesterday, compared with the period before it.
- M2 · A day with an Intake counts, even at 0 g; a day without an Intake is never read as 0 g.
- M3 · With no Intake in the period there is no mean, which is not 0.
- M4 · The gap is the displayed mean minus the previous period's, in whole signed grams; no previous Intake means no gap, a real tie is 0.
- M5 · The mean is the sum in tenths of a gram divided by 10 × N, rounded half away from zero to a whole gram.
- M6 · Adding, replacing or deleting an Intake updates the means the next time they are shown.
- M8 · A day without an Intake is a hole, always distinct from a day entered at 0 g.
- M12 · The macro period is 7 or 15 days, 7 by default, not kept after a restart, and independent of the weight period (15 or 30 days).
- M14 · Protein, carbohydrate and fat are each computed from their own macro only; one period applies to all three.

### Alert
- A1 · The Deviation compares the Weigh-in of day J with the Trend of J−1, which does not contain it.
- A2 · An Alert triggers when the Deviation is ≥ +3 kg or ≤ −3 kg.
- A3 · No Alert without a Trend on the previous day, or on a Weigh-in not fasted in the morning.
- A4 · The Alert is recomputed from current data, never frozen; the home view shows only the latest Weigh-in's Alert, the others stay visible in the history.
- A5 · A Weigh-in deviating ≥ 10 kg from the previous Trend (or from the last valid Weigh-in without a Trend) requires confirmation before saving; it runs before the Alert.

### Activities
- AC1 · An Activity type is one of Marche, Course, Mobilité, Muscu, Vélo, Autre.
- AC2 · The note is optional, trimmed, at most 200 characters; a blank note is no note.
- AC3 · Several Activities are saved at once, all or nothing; a row with neither type nor note is ignored.
- AC4 · A row with a note but no type refuses the whole save; a save with no typed row is refused; cancelling saves nothing.
- AC5 · Saving Activities appends without confirmation, allows duplicates, and never touches the Weigh-in or Intake of that date.
- AC6 · Activities default to today, accept any past date, refuse future dates, and survive a restart.
- AC7 · Each day holding at least one Activity is marked in the history; a day without Activities has no mark.
- AC8 · A marked day shows its Activities, each with its type and note.
- AC9 · An Activity can be deleted without confirmation; deleting the last Activity of a day removes its mark.
- AC10 · Adding or deleting an Activity never changes the Trend, the Energy mean or the Macro means.
- AC11 · A deleted Activity stays deleted after a restart.

### Report
- R1 · The report covers the 15 days ending today, today included, oldest first.
- R2 · Each day shows its Weigh-in with the fasted flag, its Energy and Macros, its water, and its Activities with type and note.
- R3 · A day or value without input shows nothing, never 0 nor an invented value.
- R4 · The report shows the weight and Trend curve over the 15 days, without interpolating a day without Trend.
- R5 · The 15-day change is Trend(J) − Trend(J−14), shown only when both ends have a Trend.
- R6 · The report never shows an Alert.
- R7 · The report includes a note that an empty value means nothing was entered and that Energy = 4P + 4C + 9F.
- R8 · The report is an image that can be copied for pasting into a message, not a file.
- R9 · The user is told when the report has been copied, with its 15-day date range.
- R10 · The user can view the report, exactly as it will be copied, before copying it.
- R11 · The report opens with a summary of the 15 days: the mean per day of Energy, protein, carbohydrate, fat and water, each with the number of days counted.
- R12 · A mean counts only the days with an Intake, today included. A day without Intake is never read as 0. With no Intake in the 15 days there is no mean.
- R13 · The Energy mean is rounded half away from zero to a whole kcal (E5), the macro means to a whole gram (M5), and the water mean keeps up to two decimals in litres (S6).
- R14 · The summary also gives the number of days with at least one Activity.
- R15 · The report is recomputed from the stored data each time it is shown, so it reflects every entry and deletion made since.

## Edge cases
- A missing Intake is never a zero, in any mean or gap.
- A missing previous period is never a zero gap.
- A future date is refused for every input.

## Out of scope
- Editing an Activity, activity duration, calories burned, importing Activities.
- Weight goals and macro targets.
- Import from another tracking app.
- Reminders and notifications.
- PDF and CSV export of the report.
- Choosing a custom report period: the report is always the 15 days ending today.
- Linking a day's macros to the next day's Weigh-in in the analysis.

## Decision journal
- 2026-11-24 · The report becomes a page the user can view, with a 15-day summary (R10–R15); the period stays fixed at 15 days (DEV-85).
