use serde_json::{Map, Value};

// --- Pure Rust API functions ---

pub fn api_gql_op_rs(kind: &str) -> Result<&'static str, String> {
    match kind {
        "user_about" => Ok("zUnx-DLN9dkwOkNhTLySjg/AboutAccountQuery"),
        "verified_followers" => Ok("crKOXrAHR3W3aPuKEJG8GA/BlueVerifiedFollowers"),
        "bookmarks" => Ok("XD0ViOeSOW4YoeNTGjVaYw/Bookmarks"),
        "community_info" => Ok("uBpODvS60xZ1q2L88d-W2A/CommunityQuery"),
        "community_tweets" => Ok("gabM2RYROuhItXzDYUdjyA/CommunityTweetsTimeline"),
        "followers" => Ok("_orfRBQae57vylFPH0Huhg/Followers"),
        "following" => Ok("F42cDX8PDFxkbjjq6JrM2w/Following"),
        "trends" => Ok("_dGVIf1cY6xFanFNPsAzPQ/GenericTimelineById"),
        "list_timeline" => Ok("7UuJsFvnWuZo0HmxrzU42Q/ListLatestTweetsTimeline"),
        "list_members" => Ok("oIetCo19avgStX4mOnGsPg/ListMembers"),
        "retweeters" => Ok("TZsWuSj7vGmncVnq7KWDUQ/Retweeters"),
        "search" => Ok("Yw6L66Pw54NHKuq4Dp7b4Q/SearchTimeline"),
        "tweet_details" | "tweet_replies" | "tweet_thread" => {
            Ok("oCon7R-cgWRFy6EfZjaKfg/TweetDetail")
        }
        "user_by_login" => Ok("IGgvgiOx4QZndDHuD3x9TQ/UserByScreenName"),
        "subscriptions" => Ok("-9O4xZ8ykY_Hf6kyHJX30A/UserCreatorSubscriptions"),
        "user_media" => Ok("9EovraBTXJYGSEQXZqlLmQ/UserMedia"),
        "user_tweets" => Ok("36rb3Xj3iJ64Q-9wKDjCcQ/UserTweets"),
        "user_tweets_and_replies" => Ok("D5eKzDa5ZoJuC1TCeAXbWA/UserTweetsAndReplies"),
        "community_members" => Ok("WSbJGJjZaVasSj9bnqSZSA/membersSliceTimeline_Query"),
        "community_moderators" => Ok("GBMT3GOWy5dYsYC4XJfvow/moderatorsSliceTimeline_Query"),
        _ => Err(format!("unknown gql op kind: {kind}")),
    }
}

pub fn api_gql_queue_rs(op: &str) -> &str {
    op.rsplit('/').next().unwrap_or(op)
}

pub fn api_gql_url_rs(op: &str) -> String {
    format!("https://x.com/i/api/graphql/{op}")
}

pub fn find_cursor_value_rs(obj: &Value, cursor_type: &str) -> Option<String> {
    if let Some(dict) = obj.as_object() {
        if let Some(ct) = dict.get("cursorType").and_then(Value::as_str) {
            if ct == cursor_type {
                if let Some(value) = dict.get("value").and_then(Value::as_str) {
                    return Some(value.to_owned());
                }
            }
        }
        for value in dict.values() {
            if let Some(found) = find_cursor_value_rs(value, cursor_type) {
                return Some(found);
            }
        }
    } else if let Some(arr) = obj.as_array() {
        for item in arr {
            if let Some(found) = find_cursor_value_rs(item, cursor_type) {
                return Some(found);
            }
        }
    }
    None
}

pub fn api_get_cursor_rs(obj: &Value, cursor_type: &str) -> Option<String> {
    if let Some(value) = find_cursor_value_rs(obj, cursor_type) {
        return Some(value);
    }
    get_by_path_rs(obj, "next_cursor")
}

pub fn get_by_path_rs(obj: &Value, key: &str) -> Option<String> {
    let mut stack = vec![obj];
    while let Some(current) = stack.pop() {
        if let Some(dict) = current.as_object() {
            if let Some(value) = dict.get(key) {
                return value.as_str().map(|s| s.to_owned());
            }
            for value in dict.values() {
                if value.is_object() || value.is_array() {
                    stack.push(value);
                }
            }
        } else if let Some(arr) = current.as_array() {
            for value in arr {
                if value.is_object() || value.is_array() {
                    stack.push(value);
                }
            }
        }
    }
    None
}

pub fn api_gql_entries_rs(obj: &Value) -> Vec<&Value> {
    let entries = get_by_path_rs_value(obj, "entries");
    let items_results = get_by_path_rs_value(obj, "items_results");
    let els = entries.or(items_results);

    let Some(els) = els else {
        return Vec::new();
    };

    let Some(arr) = els.as_array() else {
        return Vec::new();
    };

    if arr.is_empty() {
        return Vec::new();
    }

    let should_filter = arr
        .first()
        .and_then(|item| item.as_object())
        .map(|dict| dict.contains_key("entryId"))
        .unwrap_or(false);

    if !should_filter {
        return arr.iter().collect();
    }

    arr.iter()
        .filter(|item| {
            let entry_id = item
                .as_object()
                .and_then(|dict| dict.get("entryId"))
                .and_then(Value::as_str)
                .unwrap_or("");
            !entry_id.starts_with("cursor-")
                && !entry_id.starts_with("messageprompt-")
                && !entry_id.starts_with("module-")
                && !entry_id.starts_with("who-to-follow-")
        })
        .collect()
}

fn get_by_path_rs_value<'a>(obj: &'a Value, key: &str) -> Option<&'a Value> {
    let mut stack = vec![obj];
    while let Some(current) = stack.pop() {
        if let Some(dict) = current.as_object() {
            if let Some(value) = dict.get(key) {
                return Some(value);
            }
            for value in dict.values() {
                if value.is_object() || value.is_array() {
                    stack.push(value);
                }
            }
        } else if let Some(arr) = current.as_array() {
            for value in arr {
                if value.is_object() || value.is_array() {
                    stack.push(value);
                }
            }
        }
    }
    None
}

pub fn api_is_end_rs(res_len: usize, cur: Option<&str>, cnt: i64, lim: i64) -> (i64, bool) {
    let new_count = res_len as i64;
    let new_total = cnt + new_count;
    let is_cur = cur.is_some();
    let is_lim = lim > 0 && new_total >= lim;
    let active = is_cur && !is_lim;
    (new_total, active)
}

pub fn api_build_kv_rs(
    kind: &str,
    value: Option<&str>,
    kv: Option<&Map<String, Value>>,
) -> Result<Map<String, Value>, String> {
    let mut out = Map::new();

    match kind {
        "search" => {
            let q = value.ok_or("missing search query")?;
            out.insert("rawQuery".to_owned(), Value::String(q.to_owned()));
            out.insert("count".to_owned(), Value::Number(20.into()));
            out.insert("product".to_owned(), Value::String("Latest".to_owned()));
            out.insert(
                "querySource".to_owned(),
                Value::String("typed_query".to_owned()),
            );
        }
        "search_user" => {
            out.insert("product".to_owned(), Value::String("People".to_owned()));
        }
        "search_trend" => {
            out.insert(
                "querySource".to_owned(),
                Value::String("trend_click".to_owned()),
            );
        }
        "trends" => {
            let trend_id = value.ok_or("missing trend id")?;
            let resolved = match trend_id {
                "trending" => "VGltZWxpbmU6DAC2CwABAAAACHRyZW5kaW5nAAA",
                "news" => "VGltZWxpbmU6DAC2CwABAAAABG5ld3MAAA",
                "sport" => "VGltZWxpbmU6DAC2CwABAAAABnNwb3J0cwAA",
                "entertainment" => "VGltZWxpbmU6DAC2CwABAAAADWVudGVydGFpbm1lbnQAAA",
                other => other,
            };
            out.insert("timelineId".to_owned(), Value::String(resolved.to_owned()));
            out.insert("count".to_owned(), Value::Number(20.into()));
            out.insert(
                "withQuickPromoteEligibilityTweetFields".to_owned(),
                Value::Bool(true),
            );
        }
        "user_by_login" => {
            let login = value.ok_or("missing login")?;
            out.insert("screen_name".to_owned(), Value::String(login.to_owned()));
            out.insert("withSafetyModeUserFields".to_owned(), Value::Bool(true));
        }
        "user_about" => {
            let username = value.ok_or("missing username")?;
            out.insert("screenName".to_owned(), Value::String(username.to_owned()));
        }
        "tweet_details" | "tweet_replies" | "tweet_thread" => {
            let twid = value.ok_or("missing tweet id")?;
            out.insert("focalTweetId".to_owned(), Value::String(twid.to_owned()));
            if kind == "tweet_replies" {
                out.insert("referrer".to_owned(), Value::String("tweet".to_owned()));
            } else if kind == "tweet_thread" {
                out.insert("referrer".to_owned(), Value::String("profile".to_owned()));
            }
            out.insert(
                "with_rux_injections".to_owned(),
                Value::Bool(kind != "tweet_thread"),
            );
            out.insert("includePromotedContent".to_owned(), Value::Bool(true));
            out.insert("withCommunity".to_owned(), Value::Bool(true));
            out.insert(
                "withQuickPromoteEligibilityTweetFields".to_owned(),
                Value::Bool(true),
            );
            out.insert("withBirdwatchNotes".to_owned(), Value::Bool(true));
            out.insert("withVoice".to_owned(), Value::Bool(true));
            out.insert("withV2Timeline".to_owned(), Value::Bool(true));
        }
        "followers" | "verified_followers" | "following" | "subscriptions" => {
            let uid = value.ok_or("missing user id")?;
            out.insert("userId".to_owned(), Value::String(uid.to_owned()));
            out.insert("count".to_owned(), Value::Number(20.into()));
            out.insert("includePromotedContent".to_owned(), Value::Bool(false));
        }
        "retweeters" => {
            let twid = value.ok_or("missing tweet id")?;
            out.insert("tweetId".to_owned(), Value::String(twid.to_owned()));
            out.insert("count".to_owned(), Value::Number(20.into()));
            out.insert("includePromotedContent".to_owned(), Value::Bool(true));
        }
        "user_tweets" => {
            let uid = value.ok_or("missing user id")?;
            out.insert("userId".to_owned(), Value::String(uid.to_owned()));
            out.insert("count".to_owned(), Value::Number(40.into()));
            out.insert("includePromotedContent".to_owned(), Value::Bool(true));
            out.insert(
                "withQuickPromoteEligibilityTweetFields".to_owned(),
                Value::Bool(true),
            );
            out.insert("withVoice".to_owned(), Value::Bool(true));
            out.insert("withV2Timeline".to_owned(), Value::Bool(true));
        }
        "user_tweets_and_replies" => {
            let uid = value.ok_or("missing user id")?;
            out.insert("userId".to_owned(), Value::String(uid.to_owned()));
            out.insert("count".to_owned(), Value::Number(40.into()));
            out.insert("includePromotedContent".to_owned(), Value::Bool(true));
            out.insert("withCommunity".to_owned(), Value::Bool(true));
            out.insert("withVoice".to_owned(), Value::Bool(true));
            out.insert("withV2Timeline".to_owned(), Value::Bool(true));
        }
        "user_media" => {
            let uid = value.ok_or("missing user id")?;
            out.insert("userId".to_owned(), Value::String(uid.to_owned()));
            out.insert("count".to_owned(), Value::Number(40.into()));
            out.insert("includePromotedContent".to_owned(), Value::Bool(false));
            out.insert("withClientEventToken".to_owned(), Value::Bool(false));
            out.insert("withBirdwatchNotes".to_owned(), Value::Bool(false));
            out.insert("withVoice".to_owned(), Value::Bool(true));
            out.insert("withV2Timeline".to_owned(), Value::Bool(true));
        }
        "list_timeline" | "list_members" => {
            let list_id = value.ok_or("missing list id")?;
            out.insert("listId".to_owned(), Value::String(list_id.to_owned()));
            out.insert("count".to_owned(), Value::Number(20.into()));
        }
        "community_members" | "community_moderators" => {
            let community_id = value.ok_or("missing community id")?;
            out.insert(
                "communityId".to_owned(),
                Value::String(community_id.to_owned()),
            );
            out.insert("count".to_owned(), Value::Number(20.into()));
            out.insert("includePromotedContent".to_owned(), Value::Bool(false));
        }
        "community_tweets" => {
            let community_id = value.ok_or("missing community id")?;
            out.insert(
                "communityId".to_owned(),
                Value::String(community_id.to_owned()),
            );
            out.insert("count".to_owned(), Value::Number(40.into()));
            out.insert(
                "displayLocation".to_owned(),
                Value::String("Community".to_owned()),
            );
            out.insert(
                "rankingMode".to_owned(),
                Value::String("Relevance".to_owned()),
            );
            out.insert("withCommunity".to_owned(), Value::Bool(true));
            out.insert("includePromotedContent".to_owned(), Value::Bool(false));
            out.insert(
                "withQuickPromoteEligibilityTweetFields".to_owned(),
                Value::Bool(true),
            );
            out.insert("withVoice".to_owned(), Value::Bool(true));
            out.insert("withV2Timeline".to_owned(), Value::Bool(true));
        }
        "community_info" => {
            let community_id = value.ok_or("missing community id")?;
            out.insert(
                "communityId".to_owned(),
                Value::String(community_id.to_owned()),
            );
        }
        "bookmarks" => {
            out.insert("count".to_owned(), Value::Number(20.into()));
            out.insert("includePromotedContent".to_owned(), Value::Bool(false));
            out.insert("withClientEventToken".to_owned(), Value::Bool(false));
            out.insert("withBirdwatchNotes".to_owned(), Value::Bool(false));
            out.insert("withVoice".to_owned(), Value::Bool(true));
            out.insert("withV2Timeline".to_owned(), Value::Bool(true));
        }
        _ => {
            return Err(format!("unknown kv kind: {kind}"));
        }
    }

    if let Some(overrides) = kv {
        for (key, value) in overrides {
            out.insert(key.clone(), value.clone());
        }
    }

    Ok(out)
}

pub fn api_build_features_rs(kind: &str) -> Result<Map<String, Value>, String> {
    let mut out = Map::new();

    match kind {
        "user_by_login" => {
            out.insert(
                "highlights_tweets_tab_ui_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert("hidden_profile_likes_enabled".to_owned(), Value::Bool(true));
            out.insert(
                "creator_subscriptions_tweet_preview_api_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "hidden_profile_subscriptions_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "subscriptions_verification_info_verified_since_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "subscriptions_verification_info_is_identity_verified_enabled".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "responsive_web_twitter_article_notes_tab_enabled".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "subscriptions_feature_can_gift_premium".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "profile_label_improvements_pcf_label_in_post_enabled".to_owned(),
                Value::Bool(false),
            );
        }
        "user_about" => {
            out.insert(
                "responsive_web_graphql_timeline_navigation_enabled".to_owned(),
                Value::Bool(true),
            );
        }
        "tweet_thread" => {
            out.insert("rweb_video_screen_enabled".to_owned(), Value::Bool(false));
            out.insert(
                "profile_label_improvements_pcf_label_in_post_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "rweb_tipjar_consumption_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "verified_phone_label_enabled".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "creator_subscriptions_tweet_preview_api_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "responsive_web_graphql_timeline_navigation_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "responsive_web_graphql_skip_user_profile_image_extensions_enabled".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "premium_content_api_read_enabled".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "communities_web_enable_tweet_community_results_fetch".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "c9s_tweet_anatomy_moderator_badge_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "responsive_web_grok_analyze_button_fetch_trends_enabled".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "responsive_web_grok_analyze_post_followups_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "responsive_web_jetfuel_frame".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "responsive_web_grok_share_attachment_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert("articles_preview_enabled".to_owned(), Value::Bool(true));
            out.insert(
                "responsive_web_edit_tweet_api_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "graphql_is_translatable_rweb_tweet_is_translatable_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "view_counts_everywhere_api_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "longform_notetweets_consumption_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "responsive_web_twitter_article_tweet_consumption_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "tweet_awards_web_tipping_enabled".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "responsive_web_grok_show_grok_translated_post".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "responsive_web_grok_analysis_button_from_backend".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "creator_subscriptions_quote_tweet_preview_enabled".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "freedom_of_speech_not_reach_fetch_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert("standardized_nudges_misinfo".to_owned(), Value::Bool(true));
            out.insert(
                "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled"
                    .to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "longform_notetweets_rich_text_read_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "longform_notetweets_inline_media_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "responsive_web_grok_image_annotation_enabled".to_owned(),
                Value::Bool(true),
            );
            out.insert(
                "responsive_web_enhance_cards_enabled".to_owned(),
                Value::Bool(false),
            );
        }
        "followers" => {
            out.insert(
                "responsive_web_twitter_article_notes_tab_enabled".to_owned(),
                Value::Bool(false),
            );
        }
        "verified_followers" => {
            out.insert(
                "responsive_web_twitter_article_notes_tab_enabled".to_owned(),
                Value::Bool(true),
            );
        }
        "bookmarks" => {
            out.insert(
                "graphql_timeline_v2_bookmark_timeline".to_owned(),
                Value::Bool(true),
            );
        }
        "community_info" => {
            out.insert(
                "c9s_list_members_action_api_enabled".to_owned(),
                Value::Bool(false),
            );
            out.insert(
                "c9s_superc9s_indication_enabled".to_owned(),
                Value::Bool(false),
            );
        }
        "none" => {}
        _ => {
            return Err(format!("unknown feature kind: {kind}"));
        }
    }

    Ok(out)
}

fn set_global_features_rs(out: &mut Map<String, Value>) {
    let features = [
        ("articles_preview_enabled", false),
        ("c9s_tweet_anatomy_moderator_badge_enabled", true),
        ("communities_web_enable_tweet_community_results_fetch", true),
        ("creator_subscriptions_quote_tweet_preview_enabled", false),
        ("creator_subscriptions_tweet_preview_api_enabled", true),
        ("freedom_of_speech_not_reach_fetch_enabled", true),
        (
            "graphql_is_translatable_rweb_tweet_is_translatable_enabled",
            true,
        ),
        ("longform_notetweets_consumption_enabled", true),
        ("longform_notetweets_inline_media_enabled", true),
        ("longform_notetweets_rich_text_read_enabled", true),
        ("responsive_web_edit_tweet_api_enabled", true),
        ("responsive_web_enhance_cards_enabled", false),
        ("responsive_web_graphql_exclude_directive_enabled", true),
        (
            "responsive_web_graphql_skip_user_profile_image_extensions_enabled",
            false,
        ),
        (
            "responsive_web_grok_community_note_auto_translation_is_enabled",
            false,
        ),
        ("responsive_web_graphql_timeline_navigation_enabled", true),
        ("responsive_web_grok_imagine_annotation_enabled", false),
        ("responsive_web_media_download_video_enabled", false),
        ("responsive_web_profile_redirect_enabled", true),
        (
            "responsive_web_twitter_article_tweet_consumption_enabled",
            true,
        ),
        ("rweb_tipjar_consumption_enabled", true),
        ("rweb_video_timestamps_enabled", true),
        ("standardized_nudges_misinfo", true),
        ("tweet_awards_web_tipping_enabled", false),
        (
            "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled",
            true,
        ),
        (
            "tweet_with_visibility_results_prefer_gql_media_interstitial_enabled",
            false,
        ),
        ("tweetypie_unmention_optimization_enabled", true),
        ("verified_phone_label_enabled", false),
        ("view_counts_everywhere_api_enabled", true),
        (
            "responsive_web_grok_analyze_button_fetch_trends_enabled",
            false,
        ),
        ("premium_content_api_read_enabled", false),
        (
            "profile_label_improvements_pcf_label_in_post_enabled",
            false,
        ),
        ("responsive_web_grok_share_attachment_enabled", false),
        ("responsive_web_grok_analyze_post_followups_enabled", false),
        ("responsive_web_grok_image_annotation_enabled", false),
        ("responsive_web_grok_analysis_button_from_backend", false),
        ("responsive_web_jetfuel_frame", false),
        ("rweb_video_screen_enabled", true),
        ("responsive_web_grok_show_grok_translated_post", true),
    ];
    for (key, val) in features {
        out.insert(key.to_owned(), Value::Bool(val));
    }
}

pub fn api_merge_features_rs(overrides: Option<&Map<String, Value>>) -> Map<String, Value> {
    let mut out = Map::new();
    set_global_features_rs(&mut out);
    if let Some(overrides) = overrides {
        for (key, value) in overrides {
            out.insert(key.clone(), value.clone());
        }
    }
    out
}

fn feature_kind_for_endpoint(kind: &str) -> &str {
    match kind {
        "user_by_login" => "user_by_login",
        "user_about" => "user_about",
        "tweet_thread" => "tweet_thread",
        "followers" => "followers",
        "verified_followers" => "verified_followers",
        "bookmarks" => "bookmarks",
        "community_info" => "community_info",
        _ => "none",
    }
}

fn cursor_type_for_endpoint(kind: &str) -> &str {
    match kind {
        "tweet_replies" => "ShowMoreThreads",
        _ => "Bottom",
    }
}

pub fn api_endpoint_config_rs(
    kind: &str,
    value: Option<&str>,
    kv: Option<&Map<String, Value>>,
) -> Result<Map<String, Value>, String> {
    let mut out = Map::new();
    out.insert(
        "op".to_owned(),
        Value::String(api_gql_op_rs(kind)?.to_owned()),
    );
    out.insert(
        "kv".to_owned(),
        Value::Object(api_build_kv_rs(kind, value, kv)?),
    );
    out.insert(
        "features".to_owned(),
        Value::Object(api_build_features_rs(feature_kind_for_endpoint(kind))?),
    );
    out.insert(
        "cursor_type".to_owned(),
        Value::String(cursor_type_for_endpoint(kind).to_owned()),
    );
    Ok(out)
}

pub fn api_build_params_rs(
    queue: &str,
    kv: &Map<String, Value>,
    features: &Map<String, Value>,
    cursor: Option<&str>,
) -> Map<String, Value> {
    let mut variables = Map::new();
    for (key, value) in kv {
        variables.insert(key.clone(), value.clone());
    }
    if let Some(cursor) = cursor {
        variables.insert("cursor".to_owned(), Value::String(cursor.to_owned()));
    }

    let mut params = Map::new();
    params.insert("variables".to_owned(), Value::Object(variables));
    params.insert("features".to_owned(), Value::Object(features.clone()));

    if queue == "SearchTimeline" || queue == "ListLatestTweetsTimeline" {
        let mut toggles = Map::new();
        toggles.insert("withArticleRichContentState".to_owned(), Value::Bool(false));
        params.insert("fieldToggles".to_owned(), Value::Object(toggles));
    }
    if queue == "UserMedia" {
        let mut toggles = Map::new();
        toggles.insert("withArticlePlainText".to_owned(), Value::Bool(false));
        params.insert("fieldToggles".to_owned(), Value::Object(toggles));
    }

    params
}

pub fn api_build_gql_params_rs(
    op: &str,
    kv: &Map<String, Value>,
    features: Option<&Map<String, Value>>,
    cursor: Option<&str>,
) -> Map<String, Value> {
    let merged = api_merge_features_rs(features);
    api_build_params_rs(api_gql_queue_rs(op), kv, &merged, cursor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn gql_op_maps_endpoint_kinds() {
        assert_eq!(
            api_gql_op_rs("search").unwrap(),
            "Yw6L66Pw54NHKuq4Dp7b4Q/SearchTimeline"
        );
        assert_eq!(
            api_gql_op_rs("tweet_replies").unwrap(),
            "oCon7R-cgWRFy6EfZjaKfg/TweetDetail"
        );
        assert_eq!(
            api_gql_op_rs("community_moderators").unwrap(),
            "GBMT3GOWy5dYsYC4XJfvow/moderatorsSliceTimeline_Query"
        );
        assert_eq!(
            api_gql_op_rs("user_by_login").unwrap(),
            "IGgvgiOx4QZndDHuD3x9TQ/UserByScreenName"
        );
        assert_eq!(
            api_gql_op_rs("followers").unwrap(),
            "_orfRBQae57vylFPH0Huhg/Followers"
        );
        assert_eq!(
            api_gql_op_rs("following").unwrap(),
            "F42cDX8PDFxkbjjq6JrM2w/Following"
        );
        assert_eq!(
            api_gql_op_rs("retweeters").unwrap(),
            "TZsWuSj7vGmncVnq7KWDUQ/Retweeters"
        );
        assert_eq!(
            api_gql_op_rs("user_tweets").unwrap(),
            "36rb3Xj3iJ64Q-9wKDjCcQ/UserTweets"
        );
        assert_eq!(
            api_gql_op_rs("trends").unwrap(),
            "_dGVIf1cY6xFanFNPsAzPQ/GenericTimelineById"
        );
        assert!(api_gql_op_rs("missing").is_err());
    }

    #[test]
    fn gql_queue_and_url_are_derived_from_op() {
        assert_eq!(api_gql_queue_rs("abc/SearchTimeline"), "SearchTimeline");
        assert_eq!(api_gql_queue_rs("NoSlash"), "NoSlash");
        assert_eq!(
            api_gql_url_rs("abc/SearchTimeline"),
            "https://x.com/i/api/graphql/abc/SearchTimeline"
        );
    }

    #[test]
    fn gql_entries_filters_non_content_timeline_entries_pure() {
        let obj = json!({
            "entries": [
                {"entryId": "cursor-bottom"},
                {"entryId": "who-to-follow-1"},
                {"entryId": "tweet-1"}
            ]
        });
        let out = api_gql_entries_rs(&obj);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["entryId"], "tweet-1");
    }

    #[test]
    fn gql_entries_filters_messageprompt_and_module() {
        let obj = json!({
            "entries": [
                {"entryId": "messageprompt-1"},
                {"entryId": "module-1"},
                {"entryId": "search-1"}
            ]
        });
        let out = api_gql_entries_rs(&obj);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["entryId"], "search-1");
    }

    #[test]
    fn gql_entries_returns_empty_for_no_entries() {
        let obj = json!({"data": {}});
        let out = api_gql_entries_rs(&obj);
        assert!(out.is_empty());
    }

    #[test]
    fn get_cursor_finds_bottom_cursor() {
        let obj = json!({
            "entries": [
                {
                    "content": {
                        "cursorType": "Bottom",
                        "value": "next_page_token"
                    }
                }
            ]
        });
        let cursor = api_get_cursor_rs(&obj, "Bottom");
        assert_eq!(cursor, Some("next_page_token".to_owned()));
    }

    #[test]
    fn cursor_falls_back_to_next_cursor() {
        let obj = json!({"next_cursor": "abc"});
        let cursor = api_get_cursor_rs(&obj, "Bottom");
        assert_eq!(cursor, Some("abc".to_owned()));
    }

    #[test]
    fn is_end_returns_active_when_cursor_present() {
        let (total, active) = api_is_end_rs(5, Some("next"), 0, 10);
        assert_eq!(total, 5);
        assert!(active);
    }

    #[test]
    fn is_end_returns_inactive_when_limit_reached() {
        let (total, active) = api_is_end_rs(10, Some("next"), 0, 10);
        assert_eq!(total, 10);
        assert!(!active);
    }

    #[test]
    fn is_end_returns_inactive_when_no_cursor() {
        let (total, active) = api_is_end_rs(5, None, 0, 10);
        assert_eq!(total, 5);
        assert!(!active);
    }

    #[test]
    fn build_kv_followers_allows_overrides_and_string_ids() {
        let mut overrides = Map::new();
        overrides.insert("count".to_owned(), Value::Number(100.into()));
        let out = api_build_kv_rs("followers", Some("user1"), Some(&overrides)).unwrap();
        assert_eq!(out["userId"], "user1");
        assert_eq!(out["count"], 100);
        assert_eq!(out["includePromotedContent"], false);
    }

    #[test]
    fn build_kv_search_sets_defaults() {
        let out = api_build_kv_rs("search", Some("elon musk"), None).unwrap();
        assert_eq!(out["rawQuery"], "elon musk");
        assert_eq!(out["count"], 20);
        assert_eq!(out["product"], "Latest");
    }

    #[test]
    fn build_kv_rejects_unknown_kind() {
        assert!(api_build_kv_rs("unknown", None, None).is_err());
    }

    #[test]
    fn build_features_returns_endpoint_overrides_pure() {
        let login = api_build_features_rs("user_by_login").unwrap();
        assert_eq!(login["hidden_profile_likes_enabled"], true);
        assert_eq!(
            login["responsive_web_twitter_article_notes_tab_enabled"],
            false
        );

        let thread = api_build_features_rs("tweet_thread").unwrap();
        assert_eq!(thread["rweb_video_screen_enabled"], false);
        assert_eq!(thread["responsive_web_grok_image_annotation_enabled"], true);

        let empty = api_build_features_rs("none").unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn merge_features_adds_global_defaults_and_overrides_pure() {
        let defaults = api_merge_features_rs(None);
        assert_eq!(defaults["articles_preview_enabled"], false);
        assert_eq!(defaults["responsive_web_edit_tweet_api_enabled"], true);
        assert_eq!(
            defaults["responsive_web_grok_show_grok_translated_post"],
            true
        );

        let mut overrides = Map::new();
        overrides.insert("articles_preview_enabled".to_owned(), Value::Bool(true));
        overrides.insert("custom_feature".to_owned(), Value::String("ok".to_owned()));

        let merged = api_merge_features_rs(Some(&overrides));
        assert_eq!(merged["articles_preview_enabled"], true);
        assert_eq!(merged["custom_feature"], "ok");
    }

    #[test]
    fn endpoint_config_combines_op_kv_features_and_cursor_pure() {
        let replies = api_endpoint_config_rs("tweet_replies", Some("123"), None).unwrap();
        assert_eq!(replies["op"], "oCon7R-cgWRFy6EfZjaKfg/TweetDetail");
        assert_eq!(replies["cursor_type"], "ShowMoreThreads");
        let kv = replies["kv"].as_object().unwrap();
        assert_eq!(kv["referrer"], "tweet");
        assert_eq!(kv["focalTweetId"], "123");

        let followers = api_endpoint_config_rs("followers", Some("1"), None).unwrap();
        let features = followers["features"].as_object().unwrap();
        assert_eq!(
            features["responsive_web_twitter_article_notes_tab_enabled"],
            false
        );
    }

    #[test]
    fn build_params_adds_cursor_and_field_toggles_pure() {
        let mut kv = Map::new();
        kv.insert("count".to_owned(), Value::Number(20.into()));
        let mut features = Map::new();
        features.insert("feature_a".to_owned(), Value::Bool(true));

        let params = api_build_params_rs("SearchTimeline", &kv, &features, Some("next"));
        let variables = params["variables"].as_object().unwrap();
        assert_eq!(variables["cursor"], "next");
        assert_eq!(variables["count"], 20);

        let toggles = params["fieldToggles"].as_object().unwrap();
        assert_eq!(toggles["withArticleRichContentState"], false);
    }

    #[test]
    fn build_gql_params_merges_features_from_op_queue_pure() {
        let mut kv = Map::new();
        kv.insert("rawQuery".to_owned(), Value::String("rust".to_owned()));
        let mut overrides = Map::new();
        overrides.insert("articles_preview_enabled".to_owned(), Value::Bool(true));

        let params = api_build_gql_params_rs(
            "Yw6L66Pw54NHKuq4Dp7b4Q/SearchTimeline",
            &kv,
            Some(&overrides),
            None,
        );
        let features = params["features"].as_object().unwrap();
        assert_eq!(features["articles_preview_enabled"], true);
        assert!(params.get("fieldToggles").is_some());
    }
}
