//! Tauri shell: opens the store at startup and exposes the IPC commands.

mod commands;
mod view;

use std::sync::Mutex;

use dietlock_store::Store;
use tauri::Manager;

/// File name of the database in the app data directory.
const DB_FILE: &str = "dietlock.sqlite";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let store = Store::open(dir.join(DB_FILE))?;
            app.manage(Mutex::new(store));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::save_weigh_in,
            commands::list_weigh_ins
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
