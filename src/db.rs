const MIN_SQLITE_VERSION: f64 = 3.24;

pub fn db_min_sqlite_version() -> &'static str {
    "3.24"
}

pub fn db_check_sqlite_version(version: &str) -> Result<(), String> {
    let minor = version.split('.').take(2).collect::<Vec<_>>().join(".");
    if let Ok(value) = minor.parse::<f64>() {
        if value < MIN_SQLITE_VERSION {
            return Err(format!(
                "SQLite version '{minor}' is too old, please upgrade to {}+",
                db_min_sqlite_version()
            ));
        }
    }
    Ok(())
}

pub fn db_migration_sql(version: i64) -> Result<Vec<String>, String> {
    let statements = match version {
        1 => vec![
            "\n        CREATE TABLE IF NOT EXISTS accounts (\n            username TEXT PRIMARY KEY NOT NULL COLLATE NOCASE,\n            password TEXT NOT NULL,\n            email TEXT NOT NULL COLLATE NOCASE,\n            email_password TEXT NOT NULL,\n            user_agent TEXT NOT NULL,\n            active BOOLEAN DEFAULT FALSE NOT NULL,\n            locks TEXT DEFAULT '{}' NOT NULL,\n            headers TEXT DEFAULT '{}' NOT NULL,\n            cookies TEXT DEFAULT '{}' NOT NULL,\n            proxy TEXT DEFAULT NULL,\n            error_msg TEXT DEFAULT NULL\n        );",
        ],
        2 => vec![
            "ALTER TABLE accounts ADD COLUMN stats TEXT DEFAULT '{}' NOT NULL",
            "ALTER TABLE accounts ADD COLUMN last_used TEXT DEFAULT NULL",
        ],
        3 => vec!["ALTER TABLE accounts ADD COLUMN _tx TEXT DEFAULT NULL"],
        4 => vec!["ALTER TABLE accounts ADD COLUMN mfa_code TEXT DEFAULT NULL"],
        _ => {
            return Err(format!("unknown migration version: {version}"));
        }
    };
    Ok(statements.into_iter().map(|s| s.to_owned()).collect())
}

pub fn db_latest_migration_version() -> i64 {
    4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_version_check_matches_existing_float_comparison() {
        assert!(db_check_sqlite_version("3.24.0").is_ok());
        assert!(db_check_sqlite_version("3.40.1").is_ok());
        assert!(db_check_sqlite_version("3.8.0").is_ok());
        assert!(db_check_sqlite_version("3.23.0").is_err());
        assert!(db_check_sqlite_version("not-a-version").is_ok());
    }

    #[test]
    fn migration_sql_contains_expected_schema() {
        let v1 = db_migration_sql(1).unwrap();
        assert_eq!(v1.len(), 1);
        assert!(v1[0].contains("CREATE TABLE IF NOT EXISTS accounts"));
        assert!(v1[0].contains("username TEXT PRIMARY KEY"));

        let v2 = db_migration_sql(2).unwrap();
        assert_eq!(v2.len(), 2);
        assert!(v2[0].contains("ADD COLUMN stats"));
    }
}
