use serde_json::Value;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "/tmp/test_twscrape.db";

    // 1. Read account cookies from DB
    let conn = rusqlite::Connection::open(db_path)?;
    let cookies_json: String = conn.query_row(
        "SELECT cookies FROM accounts WHERE username = 'mapp'",
        [],
        |row| row.get(0),
    )?;
    let cookies: HashMap<String, String> = serde_json::from_str(&cookies_json)?;
    let auth_token = cookies.get("auth_token").cloned().unwrap_or_default();
    let ct0 = cookies.get("ct0").cloned().unwrap_or_default();
    println!(
        "Cookies loaded: auth_token={:.20}..., ct0={:.20}...",
        auth_token, ct0
    );
    let cookie_header = format!("auth_token={auth_token}; ct0={ct0}");

    let client = reqwest::Client::new();

    // 2. Test user_by_login
    println!("\n=== user_by_login ===");
    let op = twscrape_rs::api_gql_op_rs("user_by_login").unwrap();
    let kv = twscrape_rs::api_build_kv_rs("user_by_login", Some("elonmusk"), None)?;
    let features = twscrape_rs::api_build_features_rs("user_by_login")?;
    let mut params = serde_json::Map::new();
    params.insert("variables".to_owned(), Value::Object(kv));
    params.insert("features".to_owned(), Value::Object(features));
    let variables_str = serde_json::to_string(params.get("variables").unwrap())?;
    let features_str = serde_json::to_string(params.get("features").unwrap())?;
    let url = format!("https://x.com/i/api/graphql/{op}");
    let mut url_parsed = reqwest::Url::parse(&url)?;
    url_parsed
        .query_pairs_mut()
        .append_pair("variables", &variables_str)
        .append_pair("features", &features_str);

    let resp = client.get(url_parsed)
        .header("Authorization", "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA")
        .header("x-twitter-active-user", "yes").header("x-twitter-client-language", "en")
        .header("cookie", &cookie_header).header("x-csrf-token", &ct0)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .send().await?;
    let status = resp.status();
    let body_text = resp.text().await?;
    println!("Status: {status}, length: {}", body_text.len());
    if status == 200 {
        let body: Value = serde_json::from_str(&body_text)?;
        let rep = twscrape_rs::types::to_old_rep(&body);
        if let Some(users) = rep.get("users").and_then(|u| u.as_object()) {
            for (_, user_obj) in users {
                if let Some(user) = twscrape_rs::types::User::parse(user_obj) {
                    println!(
                        "  Found: @{0} (id={1}, followers={2})",
                        user.username, user.id, user.followers_count
                    );
                    break;
                }
            }
        }
    }

    // 3. Test tweet_details
    println!("\n=== tweet_details ===");
    let op = twscrape_rs::api_gql_op_rs("tweet_details").unwrap();
    let kv = twscrape_rs::api_build_kv_rs("tweet_details", Some("1920384454767374343"), None)?;
    let features = twscrape_rs::api_merge_features_rs(None);
    let mut params = serde_json::Map::new();
    params.insert("variables".to_owned(), Value::Object(kv));
    params.insert("features".to_owned(), Value::Object(features));
    let variables_str = serde_json::to_string(params.get("variables").unwrap())?;
    let features_str = serde_json::to_string(params.get("features").unwrap())?;
    let url = format!("https://x.com/i/api/graphql/{op}");
    let mut url_parsed = reqwest::Url::parse(&url)?;
    url_parsed
        .query_pairs_mut()
        .append_pair("variables", &variables_str)
        .append_pair("features", &features_str);

    let resp = client.get(url_parsed)
        .header("Authorization", "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA")
        .header("x-twitter-active-user", "yes").header("x-twitter-client-language", "en")
        .header("cookie", &cookie_header).header("x-csrf-token", &ct0)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .send().await?;
    let status = resp.status();
    let body_text = resp.text().await?;
    println!("Status: {status}, length: {}", body_text.len());

    if status == 200 {
        let body: Value = serde_json::from_str(&body_text)?;
        let rep = twscrape_rs::types::to_old_rep(&body);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let mut items = Vec::new();
        let mut seen = std::collections::HashSet::new();
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = twscrape_rs::types::Tweet::parse(tweet_obj, &res) {
                    if seen.insert(tweet.id) {
                        items.push(tweet);
                    }
                }
            }
        }

        println!("Parsed {} tweets:", items.len());
        for (i, tweet) in items.iter().take(5).enumerate() {
            let username = tweet
                .user
                .as_ref()
                .map(|u| u.username.as_str())
                .unwrap_or("?");
            let content_len = tweet.raw_content.len().min(100);
            println!(
                "  {}. @{username}: {}",
                i + 1,
                &tweet.raw_content[..content_len]
            );
        }
    } else {
        println!("Error: {}", &body_text[..body_text.len().min(300)]);
    }

    Ok(())
}
