# Changelog

All notable changes to DietLock are listed here. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- The home screen shows, beside the weight trend, the mean energy of the 7 days ending yesterday, how many of those days were entered, and the signed gap with the 7 days before. A day without an intake is skipped. An intake of 0 kcal counts. No intake in the window shows a sentence instead of 0. No intake in the previous week says there is no comparison (E1–E6).
- A "＋ Activité" button in the title bar opens a form to record all the activities of a day at once: a type (Marche, Course, Mobilité, Muscu, Vélo, Autre) and an optional note per row, for today or a past day. Saving adds to the day's activities and leaves the weigh-in and the intake alone. The database moves to schema 3 (AC1–AC6).

### Changed

- The top of Aujourd'hui is two labeled columns: the fasted trend, and the energy mean with a signed pill against the previous week. No previous week hides the pill. No weigh-in, and no mean, each show a sentence instead of a number. The weight-curve header is centered again.
- A weigh-in or an intake can be deleted from the history after confirmation. The confirmation shows the date and the values. Macros and water go together. The trend and alerts are recalculated (S10, T5).

### Fixed

- Unchecking "à jeun" on a saved weigh-in now asks a confirmation that names the fasted status and shows the stored and the new one, instead of talking only about kilograms (S3, S9).

## [0.2.0] - 2026-11-05

### Added

- The weight curve has its own screen. The trend dot appears on the fourth fasted weigh-in, and the overlay title bar can be dragged.

### Changed

- Each entry row's date can be changed before saving. The chip opens a calendar with today, yesterday and past days. A future date is refused in French, named like the history, and what was typed stays. Today is read again when the calendar opens or the window focuses, and that refusal does not save.
- The home screen is one card: today's weigh-in, the proposed intake and water, the weight trend, and 30 days of weight, energy and water.
- A fasted weigh-in at least 3 kg from the previous day's trend is marked in the history.
- The screen follows the system light and dark appearance. Colours are CSS variables.
- Error messages shown in the app are in French. The core and the store keep their own error text.
- Macros accept grams with at most one decimal. Water accepts litres with at most two decimals.
- Energy stays a whole number of kcal and rounds half away from zero.
- The proposed date of an intake is yesterday from noon when yesterday has no intake (D2, D8).
- Saving writes a block only when its value changes. An unchanged value writes nothing and asks no confirmation (S3).
- The empty subtitle disappears once an entry or a trend exists.
- Opening a v0.1.0 database scales stored macros from grams to tenths of a gram. Water stays in millilitres.

## [0.1.0] - 2026-10-14

First release: installable, unsigned macOS app (Apple Silicon, `.app` and `.dmg`). No design yet, no alert.

### Added

- **Weigh-in (slice 1)**
  - Enter the weight of a day, with the "fasted in the morning" flag.
  - History of weigh-ins.
  - 7-day weight trend (needs at least 4 fasted weigh-ins in the window).
  - Weight in kg with one decimal; comma or point accepted; range 30 to 250 kg.
- **Intake (slice 2)**
  - Enter protein, carbohydrate and fat (g) and water (L) for a day.
  - Energy (kcal) computed from the macros.
  - Daily list showing the weigh-in and the intake of each day.
  - Proposed date: an intake entered before 4 a.m. goes to the previous day.
- Data stored locally in SQLite; saving over an existing day asks for confirmation.
- Packaging for macOS as `.app` and `.dmg`.

[0.2.0]: https://github.com/Bertrand2808/DietLock/releases/tag/v0.2.0
[0.1.0]: https://github.com/Bertrand2808/DietLock/releases/tag/v0.1.0
