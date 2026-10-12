//! Logique métier pure de DietLock : aucune dépendance à Tauri, à SQLite ni à l'horloge.

/// Nom de l'application.
pub fn app_name() -> &'static str {
    return "DietLock";
}

#[cfg(test)]
mod tests {
    use super::app_name;

    #[test]
    fn app_name_is_dietlock() {
        assert_eq!(app_name(), "DietLock");
    }
}
