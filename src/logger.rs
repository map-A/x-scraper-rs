use std::sync::{Mutex, OnceLock};

static LOG_LEVEL: OnceLock<Mutex<String>> = OnceLock::new();

fn level_store() -> &'static Mutex<String> {
    LOG_LEVEL.get_or_init(|| Mutex::new(load_level_from_env().0))
}

fn level_no(level: &str) -> i64 {
    match level.to_ascii_uppercase().as_str() {
        "TRACE" => 5,
        "DEBUG" => 10,
        "INFO" => 20,
        "WARNING" => 30,
        "ERROR" => 40,
        "CRITICAL" => 50,
        _ => 20,
    }
}

fn load_level_from_env() -> (String, Option<String>) {
    let env = std::env::var("TWS_LOG_LEVEL")
        .unwrap_or_else(|_| "INFO".to_owned())
        .to_ascii_uppercase();

    match env.as_str() {
        "TRACE" | "DEBUG" | "INFO" | "WARNING" | "ERROR" | "CRITICAL" => (env, None),
        _ => ("INFO".to_owned(), Some(env)),
    }
}

pub fn init_log_level() -> (String, Option<String>) {
    let (level, invalid) = load_level_from_env();
    *level_store().lock().expect("log level mutex poisoned") = level.clone();
    (level, invalid)
}

pub fn set_log_level(level: &str) {
    *level_store().lock().expect("log level mutex poisoned") = level.to_ascii_uppercase();
}

pub fn log_filter(record_level_no: i64) -> bool {
    let level = level_store().lock().expect("log level mutex poisoned");
    record_level_no >= level_no(&level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_by_current_level() {
        set_log_level("ERROR");
        assert!(!log_filter(30));
        assert!(log_filter(40));
        assert!(log_filter(50));
        set_log_level("INFO");
        assert!(log_filter(20));
    }
}
