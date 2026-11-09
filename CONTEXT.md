# DietLock

Daily tracking of body weight, macros and water. It smooths weight into a trend and flags weigh-ins that deviate abnormally from it. Behaviour rules live in `.claude/rules/metier.md`.

## Days and entries

**Day**:
A calendar date. It holds at most one Weigh-in and at most one Intake.
_Avoid_: Journal entry, log

**Weigh-in**:
The body weight entered for a Day, in kg, with a flag for whether it was taken fasted in the morning.
_Avoid_: Weight entry, measurement

**Valid weigh-in**:
A Weigh-in taken fasted in the morning. Only valid Weigh-ins feed the Trend and the Alert.
_Avoid_: Good weigh-in, clean weight

**Intake**:
The macros and optional water entered for a Day.
_Avoid_: Meal, food log

**Macros**:
The protein, carbohydrate and fat amounts of an Intake, in grams with at most one decimal. They are entered together.
_Avoid_: Nutrients, nutrition values

**Energy**:
The kcal of a Day, derived from its Macros. It is never entered.
_Avoid_: Calories

**Energy mean**:
The simple mean of Energy over the Intakes in the 7 calendar days ending yesterday. A day without an Intake is left out. An Intake of 0 kcal counts. The mean does not exist when that window has no Intake.
_Avoid_: Calorie average, weekly calories

**Energy gap**:
The Energy mean minus the Energy mean of the 7 calendar days before that window, in whole kcal. It does not exist when the earlier week has no Intake. A tie is a gap of 0.
_Avoid_: Delta, variance

## Trend and alerts

**Trend**:
The smoothed weight of a Day: the simple mean of the Valid weigh-ins over the 7 calendar days ending that Day. It exists only when that window holds at least 4 Valid weigh-ins.
_Avoid_: Average, moving average, trend line

**Deviation**:
The difference between a Weigh-in and the Trend of the previous Day.
_Avoid_: Gap, delta, variance

**Alert**:
The state of a Weigh-in whose Deviation is at least 3 kg in either direction. It is a property of the Weigh-in, recomputed from current data, not a stored record.
_Avoid_: Warning, anomaly, flag

## Cut-off

**Switch hour**:
The hour before which a nutrition entry is attributed to the previous Day by default. Default 4:00.
_Avoid_: Cutoff, day boundary

## Relationships

- A Day holds at most one Weigh-in and at most one Intake; either can exist without the other.
- A Trend is derived from Weigh-ins only.
- An Energy mean is derived from Intakes only. A missing Intake is not a zero.
- An Energy gap compares two Energy means. A missing previous week is not a zero gap.
- An Alert belongs to one Weigh-in and is measured against the Trend of the previous Day.
