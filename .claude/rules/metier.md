# Business rules v1 — Weight & macro tracking

Source of truth for DietLock's behaviour. Cite the IDs (S, D, T, A) in code comments and test names. Terms are defined in `CONTEXT.md`; use those names in code.

## Purpose and vocabulary

Every morning, in under 30 seconds, the user enters today's weight, and yesterday's macros if they were forgotten. Every evening, after the last meal, they enter the day's macros. They then see their weight trend and an alert when the deviation is abnormal.

| Term | Definition |
| --- | --- |
| Day | A calendar date. It holds at most one Weigh-in and one Intake. |
| Weigh-in | The weight entered for a date, in kg, with the flag "fasted in the morning" or not. |
| Valid weigh-in | A Weigh-in with the "fasted in the morning" flag set. Every saved Weigh-in already respects the range in S7. |
| Intake | The protein, carbohydrate and fat (in g) and water (in L) entered for a date. |
| Energy | The kcal of the day, computed from the macros. |
| Trend | The smoothed weight, computed from valid weigh-ins. |
| Deviation | The difference between a Weigh-in and the previous day's Trend. |
| Switch hour | The hour before which an Intake is attributed to the previous day by default. |

## Input

A day receives two independent inputs, and no existing data is replaced without confirmation.

- **S1 · Two independent inputs.** The Weigh-in and the Intake are saved separately. Either can exist without the other.
- **S2 · Uniqueness.** A date holds at most one Weigh-in and one Intake.
- **S3 · Confirmed replacement.** Entering a Weigh-in or an Intake on a date that already has one requires confirmation. The old value is shown next to the new one.
- **S4 · Complete macros.** Protein, carbohydrate and fat are required together. Water is optional.
- **S5 · No pre-fill.** No field is pre-filled. The values of the last Intake appear greyed out in empty fields, as a hint only. An empty field is never saved (S4).
- **S6 · Formats.** Weight in kg with one decimal. Macros in grams with at most one decimal. Water in litres with at most two decimals. Comma and point are both accepted as decimal separator.
- **S7 · Valid ranges.** Weight from 30 to 250 kg. Protein, carbohydrate and fat from 0 to 1,000 g each. Water from 0 to 10 L. An out-of-range value is refused with a message stating the expected range.
- **S8 · Computed energy.** Energy = 4 × protein + 4 × carbohydrate + 9 × fat, rounded half away from zero to a whole kcal. It is never entered.
- **S9 · Weigh-in conditions.** The "fasted in the morning" flag is checked by default.
- **S10 · Deletion.** A Weigh-in or an Intake can be deleted after confirmation. Deleting a Weigh-in triggers the recalculation in T5.

## Dates

The Weigh-in is always proposed for today's date. The Intake rolls over to the previous day before 4 a.m., and from 21:00 when yesterday has no Intake.

- **D1 · Weigh-in date.** The proposed date is always today.
- **D2 · Switch hour.** Before the switch hour, the proposed date for an Intake is the previous day. After it, the proposed date is today, except in case D8: the morning window, and from 21:00 when yesterday has no Intake.
- **D3 · Adjustable switch hour.** The switch hour defaults to 4 a.m. and can be changed in settings. It applies only to Intakes, and only to the morning boundary. Changing it never moves the 21:00 evening proposal.
- **D4 · Editable date.** The proposed date can always be changed before saving.
- **D5 · No future.** No input is possible for a date after today.
- **D6 · Catch-up.** Any past date can be entered or corrected, subject to S3.
- **D7 · Switch hour change.** Changing the switch hour only changes the proposed date for future inputs. Existing inputs are never moved.
- **D8 · Forgotten yesterday.** Between the switch hour and noon, if yesterday has no Intake and today does not either, the proposed date for the Intake is yesterday. From 21:00, if yesterday has no Intake, the proposed date is yesterday, whether or not today has an Intake; if yesterday already has an Intake, the proposed date is today. The proposed day's name is highlighted whenever this rule chooses yesterday. From noon until 21:00, D2 applies.

Example: at 0:30 on Tuesday, the Intake is proposed for Monday. At 7:00 on Tuesday, with no Intake on Monday or Tuesday, the Intake is proposed for Monday, and the Weigh-in is proposed for Tuesday. At 21:00 on Tuesday, with no Intake on Monday, the Intake is proposed for Monday; if Monday already has one, it is proposed for Tuesday.

## Trend

The Trend of a day is the simple mean of the valid weigh-ins over the last 7 calendar days. It exists only if that window contains at least 4 valid weigh-ins.

- **T1 · Source.** The Trend is computed from weigh-ins only. Intakes play no role.
- **T2 · Method.** The Trend of day J is the simple mean of the valid weigh-ins from J−6 to J inclusive (7 calendar days). No interpolation of missing days.
- **T3 · Minimum.** The Trend is computed only if the window contains at least 4 valid weigh-ins. Otherwise that day has no Trend.
- **T4 · Resuming after a gap.** Follows from T2 and T3: after 7 days without a valid weigh-in, there is no Trend. It resumes at the 4th valid weigh-in within one window, ignoring weigh-ins from before the gap. Replaces the former 10-day rule.
- **T5 · Recalculation.** Adding, correcting or deleting a Weigh-in, or changing its "fasted in the morning" flag, recalculates the Trend of the 7 affected days.
- **T6 · Weigh-in outside conditions.** A Weigh-in without the "fasted in the morning" flag is saved and displayed, but excluded from the Trend and the Alert.
- **T7 · Weigh-ins in alert.** A Weigh-in that triggered an Alert stays in the Trend.
- **T8 · Changes.** Displayed changes (7 days, 4 weeks) are computed from the Trend, never from raw weigh-ins. If J or J−7 (or J−28) has no Trend, the corresponding change is not displayed.

## Alert

An Alert triggers when a Weigh-in deviates by 3 kg or more from the previous day's Trend, upward or downward.

- **A1 · Reference.** The Deviation is computed between the Weigh-in of day J and the Trend of day J−1 (window J−7 to J−1), which does not contain that Weigh-in.
- **A2 · Threshold.** An Alert triggers if the Deviation is greater than or equal to +3 kg, or less than or equal to −3 kg.
- **A3 · Conditions.** No Alert if the previous day's Trend does not exist (T3), or on a Weigh-in outside conditions (T6).
- **A4 · Recalculated alert.** The Alert is a state of the Weigh-in, computed from current data and never frozen. Correcting a past Weigh-in can make an Alert appear or disappear. The home screen shows only the Alert of the latest Weigh-in. The others remain visible in the history.
- **A5 · Typo guard.** Before saving, a Weigh-in that deviates by at least 10 kg from the previous day's Trend requires confirmation (example: 8.26 kg instead of 82.6 kg). Without a Trend, the comparison is made against the last valid Weigh-in. Without any Weigh-in, only S7 applies. This check runs before the Alert: a confirmed Weigh-in can then trigger an Alert.
- **A6 · Colour.** Red is reserved for Alerts and is used nowhere else.

## Out of scope for v1 (do not implement)

- Weight goals and macro targets.
- Import from another tracking app.
- Reminders and notifications.
- Attaching a day's macros to the next day's Weigh-in in the analysis.

## Open points

- [ ] Verify in use the ranges in S7, the 4-weigh-in threshold (T3) and the 3 kg alert threshold (A2).
