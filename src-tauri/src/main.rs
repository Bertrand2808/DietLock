// No console window besides the app window on Windows, in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    dietlock_lib::run()
}
