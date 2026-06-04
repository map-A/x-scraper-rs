use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;
use std::collections::HashMap;

fn decode_base64_or_original(value: &str) -> String {
    STANDARD
        .decode(value)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_else(|| value.to_owned())
}

fn parse_json_cookies(decoded: &str) -> Option<HashMap<String, String>> {
    let parsed: Value = serde_json::from_str(decoded).ok()?;

    let cookies = match parsed {
        Value::Object(mut obj) if obj.contains_key("cookies") => obj.remove("cookies").unwrap(),
        other => other,
    };

    match cookies {
        Value::Array(items) => {
            let mut map = HashMap::new();
            for item in items {
                let Value::Object(obj) = item else {
                    return None;
                };
                let (Some(Value::String(name)), Some(Value::String(value))) =
                    (obj.get("name"), obj.get("value"))
                else {
                    return None;
                };
                map.insert(name.clone(), value.clone());
            }
            Some(map)
        }
        Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, value) in obj {
                let text = match value {
                    Value::String(text) => text,
                    other => other.to_string(),
                };
                map.insert(key, text);
            }
            Some(map)
        }
        _ => None,
    }
}

fn parse_semicolon_cookies(decoded: &str) -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    for part in decoded.split(';') {
        let item = part.trim();
        let Some((key, val)) = item.split_once('=') else {
            continue;
        };
        map.insert(key.to_owned(), val.to_owned());
    }
    if map.is_empty() { None } else { Some(map) }
}

pub fn parse_cookies_rs(value: &str) -> Result<HashMap<String, String>, String> {
    let decoded = decode_base64_or_original(value);

    if let Some(cookies) = parse_json_cookies(&decoded) {
        return Ok(cookies);
    }

    if let Some(cookies) = parse_semicolon_cookies(&decoded) {
        return Ok(cookies);
    }

    Err(format!("Invalid cookie value: {decoded}"))
}

pub fn parse_proxy(proxy: Option<&str>) -> Option<String> {
    let proxy = proxy?;
    if proxy.is_empty() {
        return None;
    }
    if proxy.contains("://") {
        return Some(proxy.to_owned());
    }
    if proxy.contains('@') {
        return Some(format!("http://{proxy}"));
    }

    let parts = proxy.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        [host, port] => Some(format!("http://{host}:{port}")),
        [host, port, user, password] => Some(format!("http://{user}:{password}@{host}:{port}")),
        _ => Some(proxy.to_owned()),
    }
}

pub fn get_env_bool(key: &str, default_val: bool) -> bool {
    match std::env::var(key) {
        Ok(value) => matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"),
        Err(_) => default_val,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookies_parse_semicolon() {
        let val = "abc=123; def=456; ghi=789";
        let result = parse_cookies_rs(val).unwrap();
        let mut expected = HashMap::new();
        expected.insert("abc".to_owned(), "123".to_owned());
        expected.insert("def".to_owned(), "456".to_owned());
        expected.insert("ghi".to_owned(), "789".to_owned());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cookies_parse_json_object() {
        let val = r#"{"abc": "123", "def": "456", "ghi": "789"}"#;
        let result = parse_cookies_rs(val).unwrap();
        let mut expected = HashMap::new();
        expected.insert("abc".to_owned(), "123".to_owned());
        expected.insert("def".to_owned(), "456".to_owned());
        expected.insert("ghi".to_owned(), "789".to_owned());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cookies_parse_json_array() {
        let val = r#"[{"name": "abc", "value": "123"}, {"name": "def", "value": "456"}, {"name": "ghi", "value": "789"}]"#;
        let result = parse_cookies_rs(val).unwrap();
        let mut expected = HashMap::new();
        expected.insert("abc".to_owned(), "123".to_owned());
        expected.insert("def".to_owned(), "456".to_owned());
        expected.insert("ghi".to_owned(), "789".to_owned());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cookies_parse_base64_object() {
        // base64 of '{"abc": "123", "def": "456", "ghi": "789"}'
        let val = "eyJhYmMiOiAiMTIzIiwgImRlZiI6ICI0NTYiLCAiZ2hpIjogIjc4OSJ9";
        let result = parse_cookies_rs(val).unwrap();
        let mut expected = HashMap::new();
        expected.insert("abc".to_owned(), "123".to_owned());
        expected.insert("def".to_owned(), "456".to_owned());
        expected.insert("ghi".to_owned(), "789".to_owned());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cookies_parse_base64_array() {
        // base64 of '[{"name": "abc", "value": "123"}, ...]'
        let val = "W3sibmFtZSI6ICJhYmMiLCAidmFsdWUiOiAiMTIzIn0sIHsibmFtZSI6ICJkZWYiLCAidmFsdWUiOiAiNDU2In0sIHsibmFtZSI6ICJnaGkiLCAidmFsdWUiOiAiNzg5In1d";
        let result = parse_cookies_rs(val).unwrap();
        let mut expected = HashMap::new();
        expected.insert("abc".to_owned(), "123".to_owned());
        expected.insert("def".to_owned(), "456".to_owned());
        expected.insert("ghi".to_owned(), "789".to_owned());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cookies_parse_json_wrapped_cookies() {
        let val = r#"{"cookies": {"abc": "123", "def": "456", "ghi": "789"}}"#;
        let result = parse_cookies_rs(val).unwrap();
        let mut expected = HashMap::new();
        expected.insert("abc".to_owned(), "123".to_owned());
        expected.insert("def".to_owned(), "456".to_owned());
        expected.insert("ghi".to_owned(), "789".to_owned());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cookies_parse_invalid() {
        let val = "{invalid}";
        let err = parse_cookies_rs(val).unwrap_err();
        assert!(err.contains("Invalid cookie value"));
    }

    #[test]
    fn proxy_parsing_matches_python_cases() {
        assert_eq!(parse_proxy(None), None);
        assert_eq!(parse_proxy(Some("")), None);

        // already has scheme
        assert_eq!(
            parse_proxy(Some("http://1.2.3.4:8080")),
            Some("http://1.2.3.4:8080".to_owned())
        );
        assert_eq!(
            parse_proxy(Some("http://user:pass@1.2.3.4:8080")),
            Some("http://user:pass@1.2.3.4:8080".to_owned())
        );
        assert_eq!(
            parse_proxy(Some("socks5://1.2.3.4:1080")),
            Some("socks5://1.2.3.4:1080".to_owned())
        );
        assert_eq!(
            parse_proxy(Some("socks5://user:pass@1.2.3.4:1080")),
            Some("socks5://user:pass@1.2.3.4:1080".to_owned())
        );

        // host:port
        assert_eq!(
            parse_proxy(Some("1.2.3.4:8080")),
            Some("http://1.2.3.4:8080".to_owned())
        );

        // host:port:user:pass
        assert_eq!(
            parse_proxy(Some("1.2.3.4:8080:user:pass")),
            Some("http://user:pass@1.2.3.4:8080".to_owned())
        );

        // user:pass@host:port (no scheme)
        assert_eq!(
            parse_proxy(Some("user:pass@1.2.3.4:8080")),
            Some("http://user:pass@1.2.3.4:8080".to_owned())
        );
    }

    #[test]
    fn env_bool_matches_python_cases() {
        // Pick a namespaced variable unlikely to be present in the environment.
        unsafe {
            std::env::remove_var("TWSCRAPE_RS_TEST_BOOL");
            assert!(!get_env_bool("TWSCRAPE_RS_TEST_BOOL", false));
            assert!(get_env_bool("TWSCRAPE_RS_TEST_BOOL", true));
            std::env::set_var("TWSCRAPE_RS_TEST_BOOL", "yes");
            assert!(get_env_bool("TWSCRAPE_RS_TEST_BOOL", false));
            std::env::set_var("TWSCRAPE_RS_TEST_BOOL", "0");
            assert!(!get_env_bool("TWSCRAPE_RS_TEST_BOOL", true));
            std::env::remove_var("TWSCRAPE_RS_TEST_BOOL");
        }
    }
}
