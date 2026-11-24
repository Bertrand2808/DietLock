# Technical spec — DietLock
Updated: 2026-11-24

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

## Report
- The report image is drawn in the JavaScript frontend on a canvas and exported as a PNG with `canvas.toBlob('image/png')`; it is not rendered in Rust, and no new crate or npm package is added.
- The 15-day window ending today is assembled from `list_days` (day cards: Weigh-in with the fasted flag, Energy and Macros, water, Activities with type and note) and `weight_curve` (weight and Trend points, plus `shortVariation`); days are shown oldest first; the `alert` field is ignored.
- Today's date is `listing.today`, produced by the Tauri command from the system clock; `dietlock-core` stays clock-free, and the report does not use the frontend `localToday()`; a test supplies a fixed `Day` to `Listing::new` and `WeightCurve::build`.
- The 15-day box shows the Trend of J−14, the Trend of today and `shortVariation` (Trend(J) − Trend(J−14), in displayed tenths); the box is omitted when either end has no Trend; a day without Trend has no Trend point and is not interpolated.
- On a click on « Copier » on the « Rapport » page, the PNG is written to the system clipboard with `navigator.clipboard.write([new ClipboardItem({ 'image/png': pngBlobPromise })])`, keeping the blob promise inside the `ClipboardItem` so WebKit retains the user gesture; no `tauri-plugin-clipboard-manager` and no new Tauri permission.
- If that write is refused, the button does not show « ✓ Copié », a failure message is shown, and no plugin is added; if macOS refuses the Web API, work stops and Bertrand is asked.
- The « Rapport » page shows the canvas it exports, at 480 px. The current CSP (`default-src 'self'`) would block an `<img>` with a `blob:` or `data:` URL, so no `<img>` is used. The PNG is never sent over the network.
- The canvas has a logical width of 480 px and is drawn at ×2 (960 px) for display and export; fonts are at least 15 px logical.
- `invoke("report")` is called and the canvas repainted each time the « Rapport » tab is shown; nothing is cached or stored.
- The summary (means of Energy, P, G, L and water, counted-day counts, days with an Activity) is a field of `Report`, computed in `Report::assemble` in `view.rs` as a pure tested function. Its rounding reuses E5 and M5, and its window includes today.
- The toast is a fixed-position text element, hidden after 8 s; hover pauses the timer and a click closes it.

## Decision journal
- 2026-11-24 · The report is repainted at each tab display, drawn at ×2, with the summary computed in `Report::assemble` (DEV-85).

## Scheduled / real-time jobs
- None.
