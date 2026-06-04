use std::collections::HashMap;

use clap::{CommandFactory, Parser, Subcommand};
use serde_json::Value;

use twscrape_rs as lib;

// ─── CLI definition ───

#[derive(Parser)]
#[command(name = "twscrape", version, about = "Twitter/X GraphQL API scraper")]
struct Cli {
    /// Accounts database file
    #[arg(long, default_value = "accounts.db")]
    db: String,

    /// Enable debug mode
    #[arg(long)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version info
    Version,

    /// List all accounts
    Accounts,

    /// Get current usage stats
    Stats,

    /// Add accounts from file
    AddAccounts {
        /// File with accounts
        file_path: String,
        /// Account fields separated by delimiter
        line_format: String,
    },

    /// Add one account from cookies
    AddCookie {
        /// Twitter/X username
        username: String,
        /// Cookie string (omit to be prompted)
        cookies: Option<String>,
    },

    /// Delete accounts by username
    DelAccounts {
        /// Usernames to delete
        usernames: Vec<String>,
    },

    /// Log in inactive accounts
    LoginAccounts,

    /// Re-log in selected accounts
    Relogin {
        /// Usernames to re-login
        usernames: Vec<String>,
    },

    /// Retry failed account logins
    ReloginFailed,

    /// Reset all locks
    ResetLocks,

    /// Delete inactive accounts
    DeleteInactive,

    /// Search for tweets
    Search {
        /// Search query
        query: String,
        /// Max tweets to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get tweet details
    TweetDetails {
        /// Tweet ID
        tweet_id: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get replies of a tweet
    TweetReplies {
        /// Tweet ID
        tweet_id: i64,
        /// Max tweets to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get thread tweets
    TweetThread {
        /// Tweet ID
        tweet_id: i64,
        /// Max tweets to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get retweeters of a tweet
    Retweeters {
        /// Tweet ID
        tweet_id: i64,
        /// Max users to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get user data by username
    UserByLogin {
        /// Username
        username: String,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get about info for username
    UserAbout {
        /// Username
        username: String,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get user following
    Following {
        /// User ID
        user_id: i64,
        /// Max users to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get user followers
    Followers {
        /// User ID
        user_id: i64,
        /// Max users to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get user verified followers
    VerifiedFollowers {
        /// User ID
        user_id: i64,
        /// Max users to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get user subscriptions
    Subscriptions {
        /// User ID
        user_id: i64,
        /// Max users to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get user tweets
    UserTweets {
        /// User ID
        user_id: i64,
        /// Max tweets to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get user tweets and replies
    UserTweetsAndReplies {
        /// User ID
        user_id: i64,
        /// Max tweets to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get user media
    UserMedia {
        /// User ID
        user_id: i64,
        /// Max tweets to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get tweets from list
    ListTimeline {
        /// List ID
        list_id: i64,
        /// Max tweets to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get list members
    ListMembers {
        /// List ID
        list_id: i64,
        /// Max users to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get community info
    CommunityInfo {
        /// Community ID
        community_id: String,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get community members
    CommunityMembers {
        /// Community ID
        community_id: String,
        /// Max users to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get community moderators
    CommunityModerators {
        /// Community ID
        community_id: String,
        /// Max users to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get community tweets
    CommunityTweets {
        /// Community ID
        community_id: String,
        /// Max tweets to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },

    /// Get trends
    Trends {
        /// Trend ID or name (e.g. sport, news, trending)
        trend_id: String,
        /// Max trends to retrieve
        #[arg(long, default_value = "-1")]
        limit: i64,
        /// Print raw response
        #[arg(long)]
        raw: bool,
    },
}

// ─── Database helpers ───

fn open_db(path: &str) -> Result<rusqlite::Connection, String> {
    let conn = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA journal_mode=WAL")
        .map_err(|e| e.to_string())?;
    migrate_db(&conn)?;
    Ok(conn)
}

fn migrate_db(conn: &rusqlite::Connection) -> Result<(), String> {
    // Check if accounts table already exists (from Python version or previous runs)
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='accounts'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0i64)
        > 0;

    // Create _meta table if it doesn't exist
    conn.execute_batch("CREATE TABLE IF NOT EXISTS _meta (key TEXT PRIMARY KEY, value TEXT);")
        .map_err(|e| e.to_string())?;

    // Check current version in _meta
    let current: i64 = conn
        .query_row("SELECT value FROM _meta WHERE key = 'version'", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    let latest = lib::db_latest_migration_version();

    if table_exists && current == 0 {
        // Existing DB from Python version — assume fully migrated
        conn.execute_batch(&format!(
            "INSERT OR REPLACE INTO _meta (key, value) VALUES ('version', '{latest}')"
        ))
        .map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Fresh DB or partially migrated — run migrations
    conn.execute_batch(&format!(
        "INSERT OR IGNORE INTO _meta (key, value) VALUES ('version', '0')"
    ))
    .map_err(|e| e.to_string())?;

    for v in (current + 1)..=latest {
        let stmts = lib::db_migration_sql(v)?;
        for stmt in stmts {
            // Ignore "duplicate column" errors from partial migrations
            match conn.execute_batch(&stmt) {
                Ok(_) => {}
                Err(e) if e.to_string().contains("duplicate column") => {}
                Err(e) => return Err(format!("Migration {v} failed: {e}")),
            }
        }
        conn.execute(
            "UPDATE _meta SET value = ?1 WHERE key = 'version'",
            rusqlite::params![v.to_string()],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn add_account(
    conn: &rusqlite::Connection,
    username: &str,
    password: &str,
    email: &str,
    email_password: &str,
    cookies: Option<&str>,
    proxy: Option<&str>,
) -> Result<(), String> {
    let parsed_cookies = cookies
        .map(lib::parse_cookies_rs)
        .transpose()?
        .unwrap_or_default();

    let has_ct0 = parsed_cookies.contains_key("ct0");
    let cookies_json = serde_json::to_string(&parsed_cookies).map_err(|e| e.to_string())?;
    let active = has_ct0;

    let user_agent = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
    );

    let cols = vec![
        "username",
        "password",
        "email",
        "email_password",
        "user_agent",
        "active",
        "locks",
        "stats",
        "headers",
        "cookies",
        "proxy",
    ];
    let sql = lib::account_save_sql(cols.iter().map(|s| s.to_string()).collect());

    conn.execute(
        &sql,
        rusqlite::named_params! {
            ":username": username,
            ":password": password,
            ":email": email,
            ":email_password": email_password,
            ":user_agent": user_agent,
            ":active": active as i64,
            ":locks": "{}",
            ":stats": "{}",
            ":headers": "{}",
            ":cookies": cookies_json,
            ":proxy": proxy,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ─── Print table helper ───

fn print_table(rows: &[HashMap<String, Value>]) {
    if rows.is_empty() {
        return;
    }
    let keys: Vec<String> = rows[0].keys().cloned().collect();

    // Convert all values to strings
    let mut all_rows: Vec<Vec<String>> = Vec::new();
    all_rows.push(keys.clone());

    for row in rows {
        let mut str_row = Vec::new();
        for key in &keys {
            let val = row.get(key).map(value_to_string).unwrap_or_default();
            str_row.push(val);
        }
        all_rows.push(str_row);
    }

    // Calculate column widths
    let mut col_widths = vec![0usize; keys.len()];
    for row in &all_rows {
        for (i, val) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(val.len() + 1);
        }
    }

    // Print rows
    for row in &all_rows {
        let line: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, val)| format!("{val:<width$}", width = col_widths[i]))
            .collect();
        println!("{}", line.join(" "));
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            let s = n.to_string();
            if s.len() > 3 && !s.contains('.') && s.chars().all(|c| c.is_ascii_digit() || c == '-')
            {
                let neg = s.starts_with('-');
                let digits: String = if neg { s[1..].to_string() } else { s.clone() };
                let mut result = String::new();
                for (idx, c) in digits.chars().rev().enumerate() {
                    if idx > 0 && idx % 3 == 0 {
                        result.push(',');
                    }
                    result.push(c);
                }
                if neg {
                    format!("-{}", result.chars().rev().collect::<String>())
                } else {
                    result.chars().rev().collect()
                }
            } else {
                s
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        _ => v.to_string(),
    }
}

// ─── Main ───

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.debug {
        lib::set_log_level("DEBUG");
    }

    let Some(command) = cli.command else {
        // Print help
        let mut cmd = Cli::command();
        cmd.print_help().ok();
        return;
    };

    if let Err(e) = run(cli.db, command, cli.debug).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run(db_path: String, command: Commands, _debug: bool) -> Result<(), String> {
    match command {
        Commands::Version => {
            println!("twscrape: {}", env!("CARGO_PKG_VERSION"));
            println!("SQLite: {}", rusqlite::version());
            Ok(())
        }

        Commands::Accounts => {
            let conn = open_db(&db_path)?;
            let sql = lib::account_select_all_sql();
            let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
            let mut rows: Vec<HashMap<String, Value>> = Vec::new();

            let column_names: Vec<String> =
                stmt.column_names().iter().map(|s| s.to_string()).collect();

            let db_rows = stmt
                .query_map([], |row| {
                    let mut map = HashMap::new();
                    for (i, name) in column_names.iter().enumerate() {
                        let val: rusqlite::types::Value = row.get(i)?;
                        map.insert(name.clone(), rusqlite_value_to_json(val));
                    }
                    Ok(map)
                })
                .map_err(|e| e.to_string())?;

            for row in db_rows {
                if let Ok(r) = row {
                    rows.push(r);
                }
            }

            print_table(&rows);
            Ok(())
        }

        Commands::Stats => {
            let conn = open_db(&db_path)?;
            let queues = get_active_queues(&conn);
            let sql = lib::account_stats_sql(queues);
            let result = conn
                .query_row(&sql, [], |row| {
                    let total: i64 = row.get("total")?;
                    let active: i64 = row.get("active")?;
                    let inactive: i64 = row.get("inactive")?;
                    Ok((total, active, inactive))
                })
                .map_err(|e| e.to_string())?;

            println!(
                "Total: {} - Active: {} - Inactive: {}",
                result.0, result.1, result.2
            );
            Ok(())
        }

        Commands::AddAccounts {
            file_path,
            line_format,
        } => {
            let text = std::fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read {file_path}: {e}"))?;
            let accounts = lib::parse_account_file_lines_rs(&text, &line_format)?;
            let conn = open_db(&db_path)?;

            for acc in &accounts {
                add_account(
                    &conn,
                    acc.get("username").map(|s| s.as_str()).unwrap_or(""),
                    acc.get("password").map(|s| s.as_str()).unwrap_or(""),
                    acc.get("email").map(|s| s.as_str()).unwrap_or(""),
                    acc.get("email_password").map(|s| s.as_str()).unwrap_or(""),
                    acc.get("cookies").map(|s| s.as_str()),
                    acc.get("proxy").map(|s| s.as_str()),
                )?;
            }

            println!("Added {} account(s)", accounts.len());
            println!("\nNow run:\ntwscrape login_accounts");
            Ok(())
        }

        Commands::AddCookie { username, cookies } => {
            let cookies = match cookies {
                Some(c) => c,
                None => {
                    eprint!("cookies (e.g. auth_token=xxx; ct0=yyy): ");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .map_err(|e| e.to_string())?;
                    input.trim().to_string()
                }
            };

            let conn = open_db(&db_path)?;
            add_account(&conn, &username, "", "", "", Some(&cookies), None)?;
            println!("Account '{username}' added with cookies");
            Ok(())
        }

        Commands::DelAccounts { usernames } => {
            let conn = open_db(&db_path)?;
            let sql = lib::account_delete_sql(usernames.clone());
            conn.execute_batch(&sql).map_err(|e| e.to_string())?;
            println!("Deleted {} account(s)", usernames.len());
            Ok(())
        }

        Commands::LoginAccounts => {
            println!("Login flow requires async HTTP client - not yet implemented");
            println!("Use add_cookie to add accounts with cookies instead");
            Ok(())
        }

        Commands::Relogin { usernames } => {
            let conn = open_db(&db_path)?;
            let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";
            let sql = lib::account_relogin_sql(usernames.clone(), ua);
            conn.execute_batch(&sql).map_err(|e| e.to_string())?;
            println!("Reset {} account(s) for re-login", usernames.len());
            Ok(())
        }

        Commands::ReloginFailed => {
            let conn = open_db(&db_path)?;
            let sql = lib::account_relogin_failed_sql();
            let usernames: Vec<String> = {
                let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
                stmt.query_map([], |row| row.get(0))
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok())
                    .collect()
            };

            if usernames.is_empty() {
                println!("No failed accounts found");
            } else {
                let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";
                let sql = lib::account_relogin_sql(usernames.clone(), ua);
                conn.execute_batch(&sql).map_err(|e| e.to_string())?;
                println!("Reset {} failed account(s) for re-login", usernames.len());
            }
            Ok(())
        }

        Commands::ResetLocks => {
            let conn = open_db(&db_path)?;
            let sql = lib::account_reset_locks_sql();
            conn.execute_batch(sql).map_err(|e| e.to_string())?;
            println!("All locks reset");
            Ok(())
        }

        Commands::DeleteInactive => {
            let conn = open_db(&db_path)?;
            let sql = lib::account_delete_inactive_sql();
            let count = conn.execute(sql, []).map_err(|e| e.to_string())?;
            println!("Deleted {count} inactive account(s)");
            Ok(())
        }

        // ─── Search commands (require HTTP client) ───
        Commands::Search { query, limit, raw } => {
            let _ = (query, limit, raw);
            search_not_implemented("search")
        }
        Commands::TweetDetails { tweet_id, raw } => {
            let _ = (tweet_id, raw);
            search_not_implemented("tweet_details")
        }
        Commands::TweetReplies {
            tweet_id,
            limit,
            raw,
        } => {
            let _ = (tweet_id, limit, raw);
            search_not_implemented("tweet_replies")
        }
        Commands::TweetThread {
            tweet_id,
            limit,
            raw,
        } => {
            let _ = (tweet_id, limit, raw);
            search_not_implemented("tweet_thread")
        }
        Commands::Retweeters {
            tweet_id,
            limit,
            raw,
        } => {
            let _ = (tweet_id, limit, raw);
            search_not_implemented("retweeters")
        }
        Commands::UserByLogin { username, raw } => {
            let _ = (username, raw);
            search_not_implemented("user_by_login")
        }
        Commands::UserAbout { username, raw } => {
            let _ = (username, raw);
            search_not_implemented("user_about")
        }
        Commands::Following {
            user_id,
            limit,
            raw,
        } => {
            let _ = (user_id, limit, raw);
            search_not_implemented("following")
        }
        Commands::Followers {
            user_id,
            limit,
            raw,
        } => {
            let _ = (user_id, limit, raw);
            search_not_implemented("followers")
        }
        Commands::VerifiedFollowers {
            user_id,
            limit,
            raw,
        } => {
            let _ = (user_id, limit, raw);
            search_not_implemented("verified_followers")
        }
        Commands::Subscriptions {
            user_id,
            limit,
            raw,
        } => {
            let _ = (user_id, limit, raw);
            search_not_implemented("subscriptions")
        }
        Commands::UserTweets {
            user_id,
            limit,
            raw,
        } => {
            let _ = (user_id, limit, raw);
            search_not_implemented("user_tweets")
        }
        Commands::UserTweetsAndReplies {
            user_id,
            limit,
            raw,
        } => {
            let _ = (user_id, limit, raw);
            search_not_implemented("user_tweets_and_replies")
        }
        Commands::UserMedia {
            user_id,
            limit,
            raw,
        } => {
            let _ = (user_id, limit, raw);
            search_not_implemented("user_media")
        }
        Commands::ListTimeline {
            list_id,
            limit,
            raw,
        } => {
            let _ = (list_id, limit, raw);
            search_not_implemented("list_timeline")
        }
        Commands::ListMembers {
            list_id,
            limit,
            raw,
        } => {
            let _ = (list_id, limit, raw);
            search_not_implemented("list_members")
        }
        Commands::CommunityInfo { community_id, raw } => {
            let _ = (community_id, raw);
            search_not_implemented("community_info")
        }
        Commands::CommunityMembers {
            community_id,
            limit,
            raw,
        } => {
            let _ = (community_id, limit, raw);
            search_not_implemented("community_members")
        }
        Commands::CommunityModerators {
            community_id,
            limit,
            raw,
        } => {
            let _ = (community_id, limit, raw);
            search_not_implemented("community_moderators")
        }
        Commands::CommunityTweets {
            community_id,
            limit,
            raw,
        } => {
            let _ = (community_id, limit, raw);
            search_not_implemented("community_tweets")
        }
        Commands::Trends {
            trend_id,
            limit,
            raw,
        } => {
            let _ = (trend_id, limit, raw);
            search_not_implemented("trends")
        }
    }
}

fn search_not_implemented(cmd: &str) -> Result<(), String> {
    Err(format!(
        "'{cmd}' requires async HTTP client (not yet implemented). \
         Use the library API directly with tokio/reqwest."
    ))
}

fn get_active_queues(conn: &rusqlite::Connection) -> Vec<String> {
    let sql = "SELECT DISTINCT key FROM accounts, json_each(accounts.locks) WHERE active = 1";
    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(_) => return vec!["SearchTimeline".to_string()],
    };
    let queues: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|r| r.ok())
        .collect();

    if queues.is_empty() {
        vec!["SearchTimeline".to_string()]
    } else {
        queues
    }
}

fn rusqlite_value_to_json(val: rusqlite::types::Value) -> Value {
    match val {
        rusqlite::types::Value::Null => Value::Null,
        rusqlite::types::Value::Integer(i) => Value::Number(i.into()),
        rusqlite::types::Value::Real(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        rusqlite::types::Value::Text(s) => Value::String(s),
        rusqlite::types::Value::Blob(b) => {
            Value::Array(b.into_iter().map(|x| Value::Number(x.into())).collect())
        }
    }
}
