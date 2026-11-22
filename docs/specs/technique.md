# Technical spec — DietLock
Updated: 2026-11-22

## Stack
- Desktop app: Tauri 2, Rust backend (`dietlock-core`, `dietlock-store`), JavaScript frontend.
- Persistence is one local SQLite file; schema version is `PRAGMA user_version`, currently 3.

## Data model
- A Weigh-in and an Intake are each at most one row per date; an Activity allows any number of rows per date, including duplicates.
- The `activity` table (schema v3) has `id` (INTEGER PRIMARY KEY AUTOINCREMENT), `date`, `kind` (walk, run, mobility, strength, bike, other), and an optional `note` (1–200 characters).
- An Activity being saved has no id; a stored Activity is identified by its `id`.
- Deleting an Activity does not change the Weigh-in or Intake of that date, nor the Trend, Energy mean or Macro means, and does not migrate the schema.

## API / interfaces
- `list_days` includes, on each history day (J−29 to J), the day's stored Activities in entry order, each with `id`, `kind` and `note`; a day without Activities has an empty list.
- `delete_activity(id)` removes that stored Activity without confirmation; an unknown `id` is a no-op; Weigh-in and Intake of that date are left untouched.
- `save_activities` is unchanged: it appends new rows and returns only the count added; the history is reread with `list_days`.

## Constraints
- Activities outside the 30-day history window stay in the database and are not shown in the listing.
- Data stays on the device; there is no network.

## Scheduled / real-time jobs
- None.
