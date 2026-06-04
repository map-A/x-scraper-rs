use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env;

use crate::utils;

pub const AUTH_TOKEN: &str = "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA";

pub fn json_string_map(value: Option<&Value>) -> Map<String, Value> {
    value
        .and_then(Value::as_str)
        .and_then(|text| serde_json::from_str::<Value>(text).ok())
        .and_then(|value| match value {
            Value::Object(obj) => Some(obj),
            _ => None,
        })
        .unwrap_or_default()
}

pub fn stats_map(value: Option<&Value>) -> Map<String, Value> {
    json_string_map(value)
        .into_iter()
        .filter(|(_, value)| value.is_i64() || value.is_u64())
        .collect()
}

pub fn guess_delim_rs(line: &str) -> Result<String, String> {
    let parts = line.split("username").collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(format!("Invalid line format: {line}"));
    }

    let left = parts[0].trim();
    let right = parts[1].trim();
    let delimiter = if left.is_empty() {
        right.chars().next()
    } else {
        left.chars().last()
    };
    delimiter
        .map(|x| x.to_string())
        .ok_or_else(|| format!("Invalid line format: {line}"))
}

pub fn parse_account_file_lines_rs(
    text: &str,
    line_format: &str,
) -> Result<Vec<HashMap<String, String>>, String> {
    let delimiter = guess_delim_rs(line_format)?;
    let tokens = line_format
        .split(&delimiter)
        .map(str::trim)
        .collect::<Vec<_>>();

    for required in ["username", "password", "email", "email_password"] {
        if !tokens.contains(&required) {
            return Err(format!("Invalid line format: {line_format}"));
        }
    }

    let mut accounts = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let data = line.split(&delimiter).map(str::trim).collect::<Vec<_>>();
        if data.len() < tokens.len() {
            return Err(format!("Invalid line: {line}"));
        }

        let mut item = HashMap::new();
        for (key, value) in tokens.iter().zip(data.iter()) {
            if *key != "_" {
                item.insert(key.to_string(), value.to_string());
            }
        }
        accounts.push(item);
    }

    Ok(accounts)
}

pub fn selected_proxy(call_proxy: Option<&str>, account_proxy: Option<String>) -> Option<String> {
    call_proxy
        .map(str::to_owned)
        .or_else(|| env::var("TWS_PROXY").ok())
        .or(account_proxy)
        .and_then(|proxy| utils::parse_proxy(Some(&proxy)))
}

pub fn account_save_sql(cols: Vec<String>) -> String {
    format!(
        "\n        INSERT INTO accounts ({}) VALUES ({})\n        ON CONFLICT(username) DO UPDATE SET {}\n        ",
        cols.join(","),
        cols.iter()
            .map(|col| format!(":{col}"))
            .collect::<Vec<_>>()
            .join(","),
        cols.iter()
            .map(|col| format!("{col}=excluded.{col}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub fn account_lock_until_sql(queue: &str, unlock_at: i64, req_count: i64, now_ts: i64) -> String {
    format!(
        "\n        UPDATE accounts SET\n            locks = json_set(locks, '$.{queue}', datetime({unlock_at}, 'unixepoch')),\n            stats = json_set(stats, '$.{queue}', COALESCE(json_extract(stats, '$.{queue}'), 0) + {req_count}),\n            last_used = datetime({now_ts}, 'unixepoch')\n        WHERE username = :username\n        "
    )
}

pub fn account_unlock_sql(queue: &str, req_count: i64, now_ts: i64) -> String {
    format!(
        "\n        UPDATE accounts SET\n            locks = json_remove(locks, '$.{queue}'),\n            stats = json_set(stats, '$.{queue}', COALESCE(json_extract(stats, '$.{queue}'), 0) + {req_count}),\n            last_used = datetime({now_ts}, 'unixepoch')\n        WHERE username = :username\n        "
    )
}

pub fn account_get_for_queue_sql(queue: &str, order_by: &str) -> String {
    format!(
        "\n        SELECT username FROM accounts\n        WHERE active = true AND (\n            locks IS NULL\n            OR json_extract(locks, '$.{queue}') IS NULL\n            OR json_extract(locks, '$.{queue}') < datetime('now')\n        )\n        ORDER BY {order_by}\n        LIMIT 1\n        "
    )
}

fn quoted_usernames(usernames: Vec<String>) -> String {
    usernames
        .into_iter()
        .map(|username| format!("\"{username}\""))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn account_delete_sql(usernames: Vec<String>) -> String {
    format!(
        "DELETE FROM accounts WHERE username IN ({})",
        quoted_usernames(usernames)
    )
}

pub fn account_select_by_username_sql() -> &'static str {
    "SELECT * FROM accounts WHERE username = :username"
}

pub fn account_select_all_sql() -> &'static str {
    "SELECT * FROM accounts"
}

pub fn account_delete_inactive_sql() -> &'static str {
    "DELETE FROM accounts WHERE active = false"
}

pub fn account_login_select_sql(usernames: Option<Vec<String>>) -> String {
    match usernames {
        None => "SELECT * FROM accounts WHERE active = false AND error_msg IS NULL".to_owned(),
        Some(usernames) => format!(
            "SELECT * FROM accounts WHERE username IN ({})",
            quoted_usernames(usernames)
        ),
    }
}

pub fn account_relogin_sql(usernames: Vec<String>, user_agent: &str) -> String {
    format!(
        "\n        UPDATE accounts SET\n            active = false,\n            locks = json_object(),\n            last_used = NULL,\n            error_msg = NULL,\n            headers = json_object(),\n            cookies = json_object(),\n            user_agent = \"{user_agent}\"\n        WHERE username IN ({})\n        ",
        quoted_usernames(usernames)
    )
}

pub fn account_relogin_failed_sql() -> &'static str {
    "SELECT username FROM accounts WHERE active = false AND error_msg IS NOT NULL"
}

pub fn account_reset_locks_sql() -> &'static str {
    "UPDATE accounts SET locks = json_object()"
}

pub fn account_set_active_sql() -> &'static str {
    "UPDATE accounts SET active = :active WHERE username = :username"
}

fn lock_condition(condition: &str) -> String {
    if condition.contains(' ') {
        format!("({condition})")
    } else {
        format!("'{condition}'")
    }
}

pub fn account_get_and_lock_sql(
    queue: &str,
    condition: &str,
    now_ts: i64,
    sqlite_minor: i64,
    tx: Option<&str>,
) -> String {
    let condition = lock_condition(condition);
    if sqlite_minor >= 35 {
        format!(
            "\n            UPDATE accounts SET\n                locks = json_set(locks, '$.{queue}', datetime('now', '+15 minutes')),\n                last_used = datetime({now_ts}, 'unixepoch')\n            WHERE username = {condition}\n            RETURNING *\n            "
        )
    } else {
        let tx = tx.unwrap_or_default();
        format!(
            "\n            UPDATE accounts SET\n                locks = json_set(locks, '$.{queue}', datetime('now', '+15 minutes')),\n                last_used = datetime({now_ts}, 'unixepoch'),\n                _tx = '{tx}'\n            WHERE username = {condition}\n            "
        )
    }
}

pub fn account_select_by_tx_sql(tx: &str) -> String {
    format!("SELECT * FROM accounts WHERE _tx = '{tx}'")
}

pub fn account_next_available_sql(queue: &str) -> String {
    format!(
        "\n        SELECT json_extract(locks, '$.\"{queue}\"') as lock_until\n        FROM accounts\n        WHERE active = true AND json_extract(locks, '$.\"{queue}\"') IS NOT NULL\n        ORDER BY lock_until ASC\n        LIMIT 1\n        "
    )
}

pub fn account_mark_inactive_sql() -> &'static str {
    "\n        UPDATE accounts SET active = false, error_msg = :error_msg\n        WHERE username = :username\n        "
}

fn locks_count_sql(queue: &str) -> String {
    format!(
        "\n            SELECT COUNT(*) FROM accounts\n            WHERE json_extract(locks, '$.{queue}') IS NOT NULL\n                AND json_extract(locks, '$.{queue}') > datetime('now')\n            "
    )
}

pub fn account_stats_sql(queues: Vec<String>) -> String {
    let mut config = vec![
        (
            "total".to_owned(),
            "SELECT COUNT(*) FROM accounts".to_owned(),
        ),
        (
            "active".to_owned(),
            "SELECT COUNT(*) FROM accounts WHERE active = true".to_owned(),
        ),
        (
            "inactive".to_owned(),
            "SELECT COUNT(*) FROM accounts WHERE active = false".to_owned(),
        ),
    ];
    for queue in queues {
        config.push((format!("locked_{queue}"), locks_count_sql(&queue)));
    }

    format!(
        "SELECT {}",
        config
            .into_iter()
            .map(|(key, query)| format!("({query}) as {key}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guesses_delimiters_around_username() {
        assert_eq!(
            guess_delim_rs("username:password:email:email_password").unwrap(),
            ":"
        );
        assert_eq!(
            guess_delim_rs("password|username|email|email_password").unwrap(),
            "|"
        );
    }

    #[test]
    fn invalid_line_format_is_rejected() {
        assert!(guess_delim_rs("password,email,email_password").is_err());
    }

    #[test]
    fn parses_account_lines_pure_rust() {
        let items = parse_account_file_lines_rs(
            "alice:pass:a@example.test:ep\nbob:pass2:b@example.test:ep2",
            "username:password:email:email_password",
        )
        .unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["username"], "alice");
        assert_eq!(items[0]["password"], "pass");
        assert_eq!(items[0]["email"], "a@example.test");
        assert_eq!(items[0]["email_password"], "ep");
        assert_eq!(items[1]["username"], "bob");
        assert_eq!(items[1]["password"], "pass2");
    }

    #[test]
    fn parses_account_lines_with_pipe_delimiter() {
        let items = parse_account_file_lines_rs(
            "pass|alice|a@example.test|ep",
            "password|username|email|email_password",
        )
        .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["username"], "alice");
        assert_eq!(items[0]["password"], "pass");
    }

    #[test]
    fn parses_account_lines_with_underscore_placeholder() {
        let items = parse_account_file_lines_rs(
            "alice:pass:ignored:a@example.test:ep",
            "username:password:_:email:email_password",
        )
        .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["username"], "alice");
        assert!(!items[0].contains_key("_"));
    }

    #[test]
    fn rejects_invalid_line_too_few_fields() {
        let err =
            parse_account_file_lines_rs("alice:pass", "username:password:email:email_password");
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("Invalid line"));
    }

    #[test]
    fn selected_proxy_uses_first_available_source() {
        assert_eq!(
            selected_proxy(Some("1.2.3.4:8080"), Some("5.6.7.8:9090".to_owned())),
            Some("http://1.2.3.4:8080".to_owned())
        );
        assert_eq!(
            selected_proxy(None, Some("5.6.7.8:9090".to_owned())),
            Some("http://5.6.7.8:9090".to_owned())
        );
    }

    #[test]
    fn account_sql_helpers_include_expected_fragments() {
        let save = account_save_sql(vec!["username".to_owned(), "password".to_owned()]);
        assert!(save.contains("INSERT INTO accounts (username,password)"));
        assert!(save.contains(":username,:password"));
        assert!(save.contains("password=excluded.password"));

        let lock = account_lock_until_sql("SearchTimeline", 123, 2, 456);
        assert!(lock.contains("$.SearchTimeline"));
        assert!(lock.contains("datetime(123, 'unixepoch')"));
        assert!(lock.contains("+ 2"));
        assert!(lock.contains("datetime(456, 'unixepoch')"));

        let stats = account_stats_sql(vec!["SearchTimeline".to_owned()]);
        assert!(stats.contains("as total"));
        assert!(stats.contains("as locked_SearchTimeline"));
        assert!(stats.contains("json_extract(locks, '$.SearchTimeline')"));
    }

    #[test]
    fn account_more_sql_helpers_include_expected_fragments() {
        let delete = account_delete_sql(vec!["alice".to_owned(), "bob".to_owned()]);
        assert_eq!(
            delete,
            "DELETE FROM accounts WHERE username IN (\"alice\",\"bob\")"
        );
        assert_eq!(
            account_select_by_username_sql(),
            "SELECT * FROM accounts WHERE username = :username"
        );
        assert_eq!(account_select_all_sql(), "SELECT * FROM accounts");
        assert_eq!(
            account_delete_inactive_sql(),
            "DELETE FROM accounts WHERE active = false"
        );

        let login_all = account_login_select_sql(None);
        assert_eq!(
            login_all,
            "SELECT * FROM accounts WHERE active = false AND error_msg IS NULL"
        );

        let login_some = account_login_select_sql(Some(vec!["alice".to_owned()]));
        assert_eq!(
            login_some,
            "SELECT * FROM accounts WHERE username IN (\"alice\")"
        );

        let relogin = account_relogin_sql(vec!["alice".to_owned()], "ua");
        assert!(relogin.contains("user_agent = \"ua\""));
        assert!(relogin.contains("WHERE username IN (\"alice\")"));

        let returning = account_get_and_lock_sql(
            "SearchTimeline",
            "SELECT username FROM accounts",
            123,
            35,
            None,
        );
        assert!(returning.contains("RETURNING *"));
        assert!(returning.contains("WHERE username = (SELECT username FROM accounts)"));

        let fallback = account_get_and_lock_sql("SearchTimeline", "alice", 123, 34, Some("tx1"));
        assert!(fallback.contains("_tx = 'tx1'"));
        assert!(fallback.contains("WHERE username = 'alice'"));
        assert_eq!(
            account_select_by_tx_sql("tx1"),
            "SELECT * FROM accounts WHERE _tx = 'tx1'"
        );
        assert_eq!(
            account_relogin_failed_sql(),
            "SELECT username FROM accounts WHERE active = false AND error_msg IS NOT NULL"
        );
        assert_eq!(
            account_reset_locks_sql(),
            "UPDATE accounts SET locks = json_object()"
        );
        assert_eq!(
            account_set_active_sql(),
            "UPDATE accounts SET active = :active WHERE username = :username"
        );
        assert!(account_next_available_sql("SearchTimeline").contains("$.\"SearchTimeline\""));
        assert!(account_mark_inactive_sql().contains("UPDATE accounts SET active = false"));
    }
}
