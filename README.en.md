# DietLock

[Français](README.md)

Daily tracking of weight, macros and hydration, with a smoothed weight trend. Desktop app (Tauri + Rust), macOS first.

- Every morning: enter today's weight (fasted or not).
- Every evening: enter the day's macros (protein, carbohydrate, fat) and water; kcal are computed.
- Weight trend: mean of the fasted weigh-ins over 7 days, shown once the window holds at least 4.
- Data stays on your machine (SQLite), no account, no network.

**Status (v0.1.0)**: one screen (trend, entry, 30-day history) that follows light and dark mode. No targets and no chart yet.

## Install (macOS, Apple Silicon)

1. Download the `.dmg` from the [Releases](https://github.com/Bertrand2808/DietLock/releases) page.
2. Open the `.dmg` and drag DietLock into Applications.
3. First launch, since the app is unsigned: right-click DietLock, then Open, then Open. If macOS refuses (macOS 15 and later), go to System Settings, Privacy & Security, and click "Open Anyway".

Data lives in `~/Library/Application Support/com.bertrand2808.dietlock/`.

## Build

```bash
cargo tauri build   # produces the .app and .dmg in target/release/bundle/
```

## License

Dual-licensed, at your option:

- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT ([LICENSE-MIT](LICENSE-MIT))
