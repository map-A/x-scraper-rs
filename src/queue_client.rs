use serde_json::{Map, Value};

fn pad_left(value: &str, width: usize) -> String {
    format!("{value:>width$}")
}

pub fn queue_req_id(
    rate_limit_remaining: &str,
    rate_limit_limit: &str,
    username: Option<&str>,
) -> String {
    let lr = rate_limit_remaining;
    let ll = rate_limit_limit;
    let width = lr.len().max(ll.len());
    let username = username.unwrap_or("<UNKNOWN>");

    format!(
        "{}/{} - {}",
        pad_left(lr, width),
        pad_left(ll, width),
        username
    )
}

pub fn queue_response_state(
    status_code: i64,
    content_type: &str,
    has_cf_ray: bool,
    limit_remaining: i64,
    limit_reset: i64,
    err_msg: &str,
    has_dependency_user_data: bool,
) -> Map<String, Value> {
    let mut out = Map::new();
    out.insert("status_code".to_owned(), Value::Number(status_code.into()));
    out.insert(
        "limit_remaining".to_owned(),
        Value::Number(limit_remaining.into()),
    );
    out.insert("limit_reset".to_owned(), Value::Number(limit_reset.into()));
    out.insert("err_msg".to_owned(), Value::String(err_msg.to_owned()));

    if content_type.contains("text/html") && status_code >= 400 {
        out.insert("action".to_owned(), Value::String("abort".to_owned()));
        out.insert(
            "reason".to_owned(),
            Value::String("blocked_html".to_owned()),
        );
        out.insert(
            "source".to_owned(),
            Value::String(if has_cf_ray {
                "Cloudflare".to_owned()
            } else {
                "HTML".to_owned()
            }),
        );
        return out;
    }

    if err_msg.starts_with("(336) The following features cannot be null") {
        out.insert(
            "action".to_owned(),
            Value::String("dev_update_required".to_owned()),
        );
        out.insert(
            "reason".to_owned(),
            Value::String("dev_update_required".to_owned()),
        );
        return out;
    }

    if limit_remaining == 0 && limit_reset > 0 {
        out.insert("action".to_owned(), Value::String("rate_limit".to_owned()));
        out.insert("reason".to_owned(), Value::String("rate_limit".to_owned()));
        return out;
    }

    if err_msg.starts_with("(88) Rate limit exceeded") && limit_remaining > 0 {
        out.insert("action".to_owned(), Value::String("inactive".to_owned()));
        out.insert(
            "reason".to_owned(),
            Value::String("ban_rate_limit".to_owned()),
        );
        return out;
    }

    if err_msg.starts_with("(326) Authorization: Denied by access control") {
        out.insert("action".to_owned(), Value::String("inactive".to_owned()));
        out.insert(
            "reason".to_owned(),
            Value::String("access_denied".to_owned()),
        );
        return out;
    }

    if err_msg.starts_with("(32) Could not authenticate you") {
        out.insert("action".to_owned(), Value::String("inactive".to_owned()));
        out.insert("reason".to_owned(), Value::String("auth_failed".to_owned()));
        return out;
    }

    if err_msg == "OK" && status_code == 403 {
        out.insert("action".to_owned(), Value::String("inactive".to_owned()));
        out.insert("reason".to_owned(), Value::String("empty_403".to_owned()));
        out.insert("err_msg".to_owned(), Value::Null);
        return out;
    }

    if err_msg.starts_with("(131) Dependency: Internal error") {
        if status_code == 200 && has_dependency_user_data {
            out.insert("err_msg".to_owned(), Value::String("OK".to_owned()));
        } else {
            out.insert("action".to_owned(), Value::String("abort".to_owned()));
            out.insert(
                "reason".to_owned(),
                Value::String("dependency_error".to_owned()),
            );
            return out;
        }
    }

    let effective_err_msg = if err_msg.starts_with("(131) Dependency: Internal error")
        && status_code == 200
        && has_dependency_user_data
    {
        "OK"
    } else {
        err_msg
    };

    if status_code == 200 && effective_err_msg.contains("_Missing: No status found with that ID") {
        out.insert("action".to_owned(), Value::String("ignore".to_owned()));
        out.insert(
            "reason".to_owned(),
            Value::String("missing_status".to_owned()),
        );
        return out;
    }

    if status_code == 200 && effective_err_msg.contains("Authorization") {
        out.insert("action".to_owned(), Value::String("ignore".to_owned()));
        out.insert(
            "reason".to_owned(),
            Value::String("authorization_unknown".to_owned()),
        );
        return out;
    }

    if effective_err_msg != "OK" {
        out.insert("action".to_owned(), Value::String("ignore".to_owned()));
        out.insert(
            "reason".to_owned(),
            Value::String("unknown_api_error".to_owned()),
        );
        return out;
    }

    if status_code >= 400 {
        out.insert("action".to_owned(), Value::String("http_error".to_owned()));
        out.insert("reason".to_owned(), Value::String("http_error".to_owned()));
        return out;
    }

    out.insert("action".to_owned(), Value::String("ok".to_owned()));
    out.insert("reason".to_owned(), Value::String("ok".to_owned()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_left_matches_python_rjust() {
        assert_eq!(pad_left("7", 3), "  7");
        assert_eq!(pad_left("100", 2), "100");
    }

    #[test]
    fn queue_req_id_formats_limits_and_username() {
        assert_eq!(queue_req_id("7", "100", Some("alice")), "  7/100 - alice");
    }

    #[test]
    fn queue_req_id_uses_defaults() {
        assert_eq!(queue_req_id("-1", "-1", None), "-1/-1 - <UNKNOWN>");
    }

    #[test]
    fn queue_response_state_handles_ok() {
        let out = queue_response_state(200, "application/json", false, 100, 0, "OK", false);
        assert_eq!(out["action"], "ok");
        assert_eq!(out["reason"], "ok");
    }

    #[test]
    fn queue_response_state_handles_rate_limit() {
        let out = queue_response_state(429, "application/json", false, 0, 123, "OK", false);
        assert_eq!(out["action"], "rate_limit");
    }

    #[test]
    fn queue_response_state_handles_html_block() {
        let out = queue_response_state(503, "text/html", true, -1, -1, "OK", false);
        assert_eq!(out["action"], "abort");
        assert_eq!(out["source"], "Cloudflare");
    }
}
