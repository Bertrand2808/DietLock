# Changelog

All notable changes to DietLock are listed here. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed

- The home screen is one card: today's weigh-in, the proposed intake and water, the weight trend, and 30 days of weight, energy and water.
- A fasted weigh-in at least 3 kg from the previous day's trend is marked in the history.
- The screen follows the system light and dark appearance. Colours are CSS variables.
- Error messages shown in the app are in French. The core and the store keep their own error text.
- Macros accept grams with at most one decimal. Water accepts litres with at most two decimals.
- Energy stays a whole number of kcal and rounds half away from zero.
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

[0.1.0]: https://github.com/Bertrand2808/DietLock/releases/tag/v0.1.0
