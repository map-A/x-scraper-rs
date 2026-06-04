use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRef {
    pub id: i64,
    pub id_str: String,
    pub username: String,
    pub displayname: String,
}

impl UserRef {
    pub fn parse(obj: &Value) -> Option<Self> {
        let id_str = obj.get("id_str")?.as_str()?;
        Some(UserRef {
            id: id_str.parse().ok()?,
            id_str: id_str.to_owned(),
            username: obj.get("screen_name")?.as_str()?.to_owned(),
            displayname: obj.get("name")?.as_str()?.to_owned(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaPhoto {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaVideoVariant {
    pub url: String,
    pub bitrate: i64,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaVideo {
    pub thumbnail_url: String,
    pub variants: Vec<MediaVideoVariant>,
    pub duration: i64,
    pub views: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAnimated {
    pub thumbnail_url: String,
    pub video_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Media {
    pub photos: Vec<MediaPhoto>,
    pub videos: Vec<MediaVideo>,
    pub animated: Vec<MediaAnimated>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub full_name: String,
    pub name: String,
    pub place_type: String,
    pub country: String,
    pub country_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLink {
    pub url: String,
    pub text: Option<String>,
    pub tcourl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "_type")]
pub enum Card {
    #[serde(rename = "summary")]
    Summary {
        title: Option<String>,
        description: Option<String>,
        url: Option<String>,
    },
    #[serde(rename = "poll")]
    Poll {
        finished: Option<bool>,
        options: Vec<PollOption>,
    },
    #[serde(rename = "broadcast")]
    Broadcast {
        title: Option<String>,
        url: Option<String>,
        photo: Option<String>,
    },
    #[serde(rename = "audiospace")]
    Audiospace { url: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOption {
    pub label: String,
    pub votes_count: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendUrl {
    pub url: String,
    pub url_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendMetadata {
    pub domain_context: Option<String>,
    pub meta_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedTrend {
    pub name: String,
    pub url: TrendUrl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub id_str: String,
    pub username: String,
    pub displayname: String,
    pub raw_description: String,
    pub followers_count: i64,
    pub friends_count: i64,
    pub statuses_count: i64,
    pub favourites_count: i64,
    pub listed_count: i64,
    pub media_count: i64,
    pub profile_image_url: String,
    pub location: String,
    pub protected: bool,
    pub verified: bool,
    pub blue: bool,
    pub blue_type: Option<String>,
    pub description_links: Vec<TextLink>,
    pub pinned_ids: Vec<i64>,
    pub profile_banner_url: Option<String>,
    pub url: String,
}

impl User {
    pub fn parse(obj: &Value) -> Option<Self> {
        let id_str = obj.get("id_str")?.as_str()?;
        let username = obj.get("screen_name")?.as_str()?;

        let description_links = obj
            .get("entities")
            .and_then(|e| e.get("description"))
            .and_then(|d| d.get("urls"))
            .and_then(|u| u.as_array())
            .map(|arr| arr.iter().filter_map(TextLink::parse).collect())
            .unwrap_or_default();

        let pinned_ids: Vec<i64> = obj
            .get("pinned_tweet_ids_str")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str()?.parse().ok())
                    .collect()
            })
            .unwrap_or_default();

        Some(User {
            id: id_str.parse().ok()?,
            id_str: id_str.to_owned(),
            username: username.to_owned(),
            displayname: obj.get("name")?.as_str()?.to_owned(),
            raw_description: get_str(obj, "description"),
            followers_count: get_i64(obj, "followers_count"),
            friends_count: get_i64(obj, "friends_count"),
            statuses_count: get_i64(obj, "statuses_count"),
            favourites_count: get_i64(obj, "favourites_count"),
            listed_count: get_i64(obj, "listed_count"),
            media_count: get_i64(obj, "media_count"),
            profile_image_url: get_str(obj, "profile_image_url_https"),
            location: get_str(obj, "location"),
            protected: obj
                .get("protected")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            verified: obj
                .get("verified")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            blue: obj
                .get("is_blue_verified")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            blue_type: obj
                .get("verified_type")
                .and_then(Value::as_str)
                .map(|s| s.to_owned()),
            description_links,
            pinned_ids,
            profile_banner_url: obj
                .get("profile_banner_url")
                .and_then(Value::as_str)
                .map(|s| s.to_owned()),
            url: format!("https://x.com/{username}"),
        })
    }
}

impl TextLink {
    pub fn parse(obj: &Value) -> Option<Self> {
        let url = obj.get("expanded_url").and_then(Value::as_str)?;
        let tcourl = obj.get("url").and_then(Value::as_str)?;
        Some(TextLink {
            url: url.to_owned(),
            text: obj
                .get("display_url")
                .and_then(Value::as_str)
                .map(|s| s.to_owned()),
            tcourl: Some(tcourl.to_owned()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    pub id: i64,
    pub id_str: String,
    pub url: String,
    pub raw_content: String,
    pub user: Option<User>,
    pub conversation_id: i64,
    pub conversation_id_str: String,
    pub reply_count: i64,
    pub retweet_count: i64,
    pub like_count: i64,
    pub quote_count: i64,
    pub bookmarked_count: i64,
    pub view_count: Option<i64>,
    pub lang: String,
    pub source: Option<String>,
    pub in_reply_to_tweet_id: Option<i64>,
    pub in_reply_to_tweet_id_str: Option<String>,
    pub in_reply_to_user: Option<UserRef>,
    pub mentioned_users: Vec<UserRef>,
    pub links: Vec<TextLink>,
    pub hashtags: Vec<String>,
    pub cashtags: Vec<String>,
    pub media: Option<Media>,
    pub card: Option<Card>,
    pub retweeted_tweet: Option<Box<Tweet>>,
    pub quoted_tweet: Option<Box<Tweet>>,
    pub place: Option<Place>,
    pub coordinates: Option<Coordinates>,
    pub is_quote_status: bool,
    pub is_translatable: bool,
    pub display_text_range: Option<Vec<i64>>,
    pub in_reply_to_screen_name: Option<String>,
    pub edit_control: Option<Value>,
    pub voice_info: Option<Value>,
}

impl Tweet {
    pub fn parse(obj: &Value, res: &Value) -> Option<Self> {
        let id_str = obj.get("id_str")?.as_str()?;
        let user_id_str = obj.get("user_id_str")?.as_str()?;

        let user_obj = get_nested(res, &format!("users.{user_id_str}"));
        let user = user_obj.and_then(|u| User::parse(u));

        let username = user
            .as_ref()
            .map(|u| u.username.as_str())
            .unwrap_or("unknown");
        let conversation_id_str =
            get_str_opt(obj, "conversation_id_str").unwrap_or_else(|| id_str.to_owned());

        let raw_content = get_nested(obj, "note_tweet.note_tweet_results.result.text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| get_str(obj, "full_text"));

        let hashtags: Vec<String> = get_nested(obj, "entities.hashtags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|h| h.get("text")?.as_str().map(|s| s.to_owned()))
                    .collect()
            })
            .unwrap_or_default();

        let cashtags: Vec<String> = get_nested(obj, "entities.symbols")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|h| h.get("text")?.as_str().map(|s| s.to_owned()))
                    .collect()
            })
            .unwrap_or_default();

        let mentioned_users: Vec<UserRef> = get_nested(obj, "entities.user_mentions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(UserRef::parse).collect())
            .unwrap_or_default();

        let links = parse_links(obj);

        let media = parse_media(obj);

        let card = parse_card(obj, &format!("https://x.com/{username}/status/{id_str}"));

        // Retweeted tweet
        let rt_id = _first_str(
            obj,
            &[
                "retweeted_status_id_str",
                "retweeted_status_result.result.rest_id",
                "retweeted_status_result.result.tweet.rest_id",
            ],
        );
        let rt_obj = rt_id.and_then(|id| get_nested(res, &format!("tweets.{id}")));
        let retweeted_tweet = rt_obj.and_then(|o| Tweet::parse(o, res));

        // View count: check current tweet and retweeted tweet
        let view_count = get_views_from(obj, rt_obj.unwrap_or(&Value::Null));

        // Quoted tweet
        let qt_id = _first_str(
            obj,
            &[
                "quoted_status_id_str",
                "quoted_status_result.result.rest_id",
                "quoted_status_result.result.tweet.rest_id",
            ],
        );
        let qt_obj = qt_id.and_then(|id| get_nested(res, &format!("tweets.{id}")));
        let quoted_tweet = qt_obj.and_then(|o| Tweet::parse(o, res));

        // Issue #42: restore full rt text
        let mut raw_content = raw_content;
        if let Some(ref rt) = retweeted_tweet {
            if let Some(ref rt_user) = rt.user {
                if raw_content.ends_with('…') {
                    let rt_msg = format!("RT @{}: {}", rt_user.username, rt.raw_content);
                    if raw_content != rt_msg {
                        raw_content = rt_msg;
                    }
                }
            }
        }

        let in_reply_to_user_id = obj.get("in_reply_to_user_id_str").and_then(Value::as_str);
        let in_reply_to_user = in_reply_to_user_id
            .and_then(|uid| get_nested(res, &format!("users.{uid}")))
            .and_then(|u| {
                Some(UserRef {
                    id: u.get("id_str")?.as_str()?.parse().ok()?,
                    id_str: u.get("id_str")?.as_str()?.to_owned(),
                    username: u.get("screen_name")?.as_str()?.to_owned(),
                    displayname: u.get("name")?.as_str()?.to_owned(),
                })
            });

        Some(Tweet {
            id: id_str.parse().ok()?,
            id_str: id_str.to_owned(),
            url: format!("https://x.com/{username}/status/{id_str}"),
            raw_content,
            user,
            conversation_id: conversation_id_str.parse().unwrap_or(0),
            conversation_id_str,
            reply_count: get_i64(obj, "reply_count"),
            retweet_count: get_i64(obj, "retweet_count"),
            like_count: get_i64(obj, "favorite_count"),
            quote_count: get_i64(obj, "quote_count"),
            bookmarked_count: obj
                .get("bookmark_count")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            view_count,
            lang: get_str(obj, "lang"),
            source: obj
                .get("source")
                .and_then(Value::as_str)
                .map(|s| s.to_owned()),
            in_reply_to_tweet_id: obj
                .get("in_reply_to_status_id_str")
                .and_then(Value::as_str)
                .and_then(|s| s.parse().ok()),
            in_reply_to_tweet_id_str: obj
                .get("in_reply_to_status_id_str")
                .and_then(Value::as_str)
                .map(|s| s.to_owned()),
            in_reply_to_user,
            mentioned_users,
            links,
            hashtags,
            cashtags,
            media,
            card,
            retweeted_tweet: retweeted_tweet.map(Box::new),
            quoted_tweet: quoted_tweet.map(Box::new),
            place: parse_place(obj),
            coordinates: parse_coordinates(obj),
            is_quote_status: obj
                .get("is_quote_status")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            is_translatable: obj
                .get("is_translatable")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            display_text_range: obj
                .get("display_text_range")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect()),
            in_reply_to_screen_name: obj
                .get("in_reply_to_screen_name")
                .and_then(Value::as_str)
                .map(|s| s.to_owned()),
            edit_control: obj.get("edit_control").cloned(),
            voice_info: obj.get("voice_info").cloned(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    pub id: String,
    pub name: String,
    pub rank: i64,
    pub trend_url: TrendUrl,
    pub trend_metadata: TrendMetadata,
    pub grouped_trends: Option<Vec<GroupedTrend>>,
}

impl Trend {
    pub fn parse(obj: &Value) -> Option<Self> {
        let name = obj.get("name")?.as_str()?;
        let rank = obj
            .get("rank")
            .and_then(|v| match v {
                Value::Number(n) => n.as_i64(),
                Value::String(s) => s.parse().ok(),
                _ => None,
            })
            .unwrap_or(0);

        let trend_url_obj = obj.get("trend_url")?;
        let trend_url = TrendUrl {
            url: get_str(trend_url_obj, "url"),
            url_type: get_str(trend_url_obj, "urlType"),
        };

        let metadata_obj = obj.get("trend_metadata").unwrap_or(&Value::Null);
        let trend_metadata = TrendMetadata {
            domain_context: metadata_obj
                .get("domain_context")
                .and_then(Value::as_str)
                .map(|s| s.to_owned()),
            meta_description: metadata_obj
                .get("meta_description")
                .and_then(Value::as_str)
                .map(|s| s.to_owned()),
        };

        let grouped_trends = obj
            .get("grouped_trends")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?;
                        let url_obj = item.get("url")?;
                        Some(GroupedTrend {
                            name: name.to_owned(),
                            url: TrendUrl {
                                url: get_str(url_obj, "url"),
                                url_type: get_str(url_obj, "urlType"),
                            },
                        })
                    })
                    .collect()
            });

        Some(Trend {
            id: format!("trend-{name}"),
            name: name.to_owned(),
            rank,
            trend_url,
            trend_metadata,
            grouped_trends,
        })
    }
}

// --- Utility functions ---

fn get_str(obj: &Value, key: &str) -> String {
    obj.get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

fn get_str_opt(obj: &Value, key: &str) -> Option<String> {
    obj.get(key).and_then(Value::as_str).map(|s| s.to_owned())
}

fn get_i64(obj: &Value, key: &str) -> i64 {
    obj.get(key).and_then(Value::as_i64).unwrap_or(0)
}

pub fn get_nested<'a>(obj: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = obj;
    for part in path.split('.') {
        current = current.get(part)?;
        if current.is_null() {
            return None;
        }
    }
    Some(current)
}

fn _first_str(obj: &Value, paths: &[&str]) -> Option<String> {
    for path in paths {
        if let Some(v) = get_nested(obj, path) {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    return Some(s.to_owned());
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
fn get_views(obj: &Value) -> Option<i64> {
    get_views_from(obj, &Value::Null)
}

fn get_views_from(obj: &Value, rt_obj: &Value) -> Option<i64> {
    for source in [obj, rt_obj] {
        for path in ["ext_views.count", "views.count"] {
            if let Some(v) = get_nested(source, path) {
                if let Some(n) = v.as_i64() {
                    return Some(n);
                }
                if let Some(s) = v.as_str() {
                    if let Ok(n) = s.parse::<i64>() {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}

fn parse_links(obj: &Value) -> Vec<TextLink> {
    let mut links = Vec::new();

    if let Some(urls) = get_nested(obj, "entities.urls").and_then(|v| v.as_array()) {
        for url in urls {
            if let Some(link) = TextLink::parse(url) {
                links.push(link);
            }
        }
    }

    if let Some(urls) = get_nested(obj, "note_tweet.note_tweet_results.result.entity_set.urls")
        .and_then(|v| v.as_array())
    {
        for url in urls {
            if let Some(link) = TextLink::parse(url) {
                links.push(link);
            }
        }
    }

    links
}

fn parse_media(obj: &Value) -> Option<Media> {
    let media_arr = obj
        .get("extended_entities")
        .and_then(|e| e.get("media"))
        .and_then(|m| m.as_array())?;

    let mut photos = Vec::new();
    let mut videos = Vec::new();
    let mut animated = Vec::new();

    for item in media_arr {
        let media_type = item.get("type").and_then(Value::as_str).unwrap_or("");
        match media_type {
            "photo" => {
                if let Some(url) = item.get("media_url_https").and_then(Value::as_str) {
                    photos.push(MediaPhoto {
                        url: url.to_owned(),
                    });
                }
            }
            "video" => {
                if let Some(video) = parse_video(item) {
                    videos.push(video);
                }
            }
            "animated_gif" => {
                let thumbnail = item
                    .get("media_url_https")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let video_url = get_nested(item, "video_info.variants")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.last())
                    .and_then(|v| v.get("url"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                animated.push(MediaAnimated {
                    thumbnail_url: thumbnail,
                    video_url,
                });
            }
            _ => {}
        }
    }

    Some(Media {
        photos,
        videos,
        animated,
    })
}

fn parse_video(item: &Value) -> Option<MediaVideo> {
    let thumbnail = item.get("media_url_https")?.as_str()?.to_owned();
    let video_info = item.get("video_info")?;
    let duration = video_info.get("duration_millis")?.as_i64()?;
    let variants: Vec<MediaVideoVariant> = video_info
        .get("variants")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|v| v.get("bitrate").is_some())
                .filter_map(|v| {
                    Some(MediaVideoVariant {
                        url: v.get("url")?.as_str()?.to_owned(),
                        bitrate: v.get("bitrate")?.as_i64()?,
                        content_type: v.get("content_type")?.as_str()?.to_owned(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Some(MediaVideo {
        thumbnail_url: thumbnail,
        variants,
        duration,
        views: get_nested(item, "mediaStats.viewCount").and_then(Value::as_i64),
    })
}

fn parse_place(obj: &Value) -> Option<Place> {
    let place = obj.get("place")?;
    Some(Place {
        id: place.get("id")?.as_str()?.to_owned(),
        full_name: place.get("full_name")?.as_str()?.to_owned(),
        name: place.get("name")?.as_str()?.to_owned(),
        place_type: place.get("place_type")?.as_str()?.to_owned(),
        country: place.get("country")?.as_str()?.to_owned(),
        country_code: place.get("country_code")?.as_str()?.to_owned(),
    })
}

fn parse_coordinates(obj: &Value) -> Option<Coordinates> {
    if let Some(coords) = get_nested(obj, "coordinates.coordinates").and_then(|v| v.as_array()) {
        if coords.len() >= 2 {
            return Some(Coordinates {
                longitude: coords[0].as_f64()?,
                latitude: coords[1].as_f64()?,
            });
        }
    }
    if let Some(geo) = get_nested(obj, "geo.coordinates").and_then(|v| v.as_array()) {
        if geo.len() >= 2 {
            return Some(Coordinates {
                longitude: geo[1].as_f64()?,
                latitude: geo[0].as_f64()?,
            });
        }
    }
    None
}

fn parse_card(obj: &Value, _tweet_url: &str) -> Option<Card> {
    let binding = Value::Null;
    let card_obj = obj.get("card").unwrap_or(&binding);
    let legacy = card_obj.get("legacy").unwrap_or(card_obj);

    let name = legacy.get("name").and_then(Value::as_str).unwrap_or("");
    let binding2: Vec<Value> = Vec::new();
    let values = legacy
        .get("binding_values")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or(binding2);

    // Helper: find value by key in binding_values list
    let get_str = |key: &str| -> Option<String> {
        values
            .iter()
            .find(|v| v.get("key").and_then(Value::as_str) == Some(key))
            .and_then(|v| v.get("value"))
            .and_then(|v| v.get("string_value"))
            .and_then(Value::as_str)
            .map(|s| s.to_owned())
    };

    let get_bool = |key: &str| -> Option<bool> {
        values
            .iter()
            .find(|v| v.get("key").and_then(Value::as_str) == Some(key))
            .and_then(|v| v.get("value"))
            .and_then(|v| v.get("boolean_value"))
            .and_then(Value::as_bool)
    };

    // Unified card
    if name == "unified_card" {
        let unified_json = values
            .iter()
            .find(|v| v.get("key").and_then(Value::as_str) == Some("unified_card"))
            .and_then(|v| v.get("value"))
            .and_then(|v| v.get("string_value"))
            .and_then(Value::as_str)?;

        let unified: Value = serde_json::from_str(unified_json).ok()?;
        let co = unified.get("component_objects").unwrap_or(&Value::Null);
        let dobj = unified.get("destination_objects").unwrap_or(&Value::Null);
        let me = unified
            .get("media_entities")
            .and_then(|v| v.as_object())
            .map(|obj| obj.values().collect::<Vec<_>>())
            .unwrap_or_default();
        let _me = me.first().cloned();

        let title = get_nested(co, "details_1.data.title.content")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let description = get_nested(co, "details_1.data.subtitle.content")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let url = get_nested(dobj, "browser_with_docked_media_1.data.url_data.url")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();

        return Some(Card::Summary {
            title: Some(title),
            description: Some(description),
            url: Some(url),
        });
    }

    // Summary / summary_large_image / player cards
    if name.starts_with("summary") || name.starts_with("player") {
        // Extract title: prefer longest among "title" and "*_alt_text" keys
        let mut title_candidates: Vec<String> = values
            .iter()
            .filter(|v| {
                let key = v.get("key").and_then(Value::as_str).unwrap_or("");
                key == "title" || key.ends_with("_alt_text")
            })
            .filter_map(|v| {
                v.get("value")
                    .and_then(|v| v.get("string_value"))
                    .and_then(Value::as_str)
                    .map(|s| s.to_owned())
            })
            .collect();
        title_candidates.sort_by(|a, b| b.len().cmp(&a.len()));
        let title = title_candidates.first().cloned();

        let description = get_str("description");
        let url = get_str("card_url");

        return Some(Card::Summary {
            title,
            description,
            url,
        });
    }

    // Poll card
    if name.starts_with("poll") && name.contains("choice") {
        let finished = get_bool("counts_are_final");
        let mut options = Vec::new();
        for i in 1..=20 {
            let label_key = format!("choice{i}_label");
            let count_key = format!("choice{i}_count");
            if let Some(label) = get_str(&label_key) {
                let count = get_str(&count_key).unwrap_or_else(|| "0".to_owned());
                options.push(PollOption {
                    label,
                    votes_count: count,
                });
            }
        }
        if !options.is_empty() {
            return Some(Card::Poll { finished, options });
        }
    }

    // Broadcast card
    if name.contains("broadcast") {
        // Find the largest image by height
        let photo = values
            .iter()
            .filter(|v| {
                v.get("value")
                    .and_then(|v| v.get("type"))
                    .and_then(Value::as_str)
                    == Some("IMAGE")
            })
            .filter(|v| {
                let key = v.get("key").and_then(Value::as_str).unwrap_or("");
                !key.contains("color")
            })
            .max_by_key(|v| {
                v.get("value")
                    .and_then(|v| v.get("image_value"))
                    .and_then(|v| v.get("height"))
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
            })
            .and_then(|v| {
                v.get("value")
                    .and_then(|v| v.get("image_value"))
                    .and_then(|v| v.get("url"))
                    .and_then(Value::as_str)
                    .map(|s| s.to_owned())
            });

        return Some(Card::Broadcast {
            title: get_str("broadcast_title"),
            url: get_str("broadcast_url"),
            photo,
        });
    }

    // Audiospace / Periscope card
    if name.contains("audiospace") || name.contains("periscope") {
        return Some(Card::Audiospace {
            url: get_str("card_url"),
        });
    }

    None
}

// --- to_old_rep: convert Twitter V2 API response to V1 flat format ---

pub fn to_old_rep(obj: &Value) -> Value {
    use std::collections::HashMap;

    fn get_typed_objects(obj: &Value, res: &mut HashMap<String, Vec<Value>>) {
        if let Some(typename) = obj.get("__typename").and_then(Value::as_str) {
            res.entry(typename.to_owned())
                .or_default()
                .push(obj.clone());
        }
        if let Some(dict) = obj.as_object() {
            for value in dict.values() {
                if value.is_object() || value.is_array() {
                    get_typed_objects(value, res);
                }
            }
        } else if let Some(arr) = obj.as_array() {
            for value in arr {
                if value.is_object() || value.is_array() {
                    get_typed_objects(value, res);
                }
            }
        }
    }

    fn merge_legacy(base: &Value, legacy: &Value) -> Value {
        let mut out = base.clone();
        if let (Some(out_obj), Some(legacy_obj)) = (out.as_object_mut(), legacy.as_object()) {
            for (k, v) in legacy_obj {
                if !out_obj.contains_key(k) {
                    out_obj.insert(k.clone(), v.clone());
                }
            }
        }
        out
    }

    fn flatten_user_v2(obj: &Value) -> Value {
        let legacy = obj.get("legacy").unwrap_or(&Value::Null).clone();
        let mut flat = merge_legacy(obj, &legacy);

        let rest_id = flat
            .get("rest_id")
            .or_else(|| flat.get("id_str"))
            .and_then(Value::as_str)
            .map(|s| s.to_owned())
            .or_else(|| {
                obj.get("rest_id")
                    .or_else(|| obj.get("id_str"))
                    .and_then(Value::as_str)
                    .map(|s| s.to_owned())
            });

        if let Some(ref rid) = rest_id {
            if let Some(obj) = flat.as_object_mut() {
                obj.insert("rest_id".to_owned(), Value::String(rid.clone()));
                obj.insert("id_str".to_owned(), Value::String(rid.clone()));
                obj.insert(
                    "id".to_owned(),
                    Value::Number(rid.parse::<i64>().unwrap_or(0).into()),
                );
                obj.insert("legacy".to_owned(), Value::Null);
            }
        }

        // Fill from core
        let core = obj
            .get("core")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        if let (Some(flat_obj), Some(core_obj)) = (flat.as_object_mut(), core.as_object()) {
            for key in ["screen_name", "name", "created_at"] {
                if !flat_obj.contains_key(key) {
                    if let Some(val) = core_obj.get(key) {
                        flat_obj.insert(key.to_owned(), val.clone());
                    }
                }
            }
        }

        // Fill avatar
        if flat
            .get("profile_image_url_https")
            .and_then(Value::as_str)
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            if let Some(url) = obj
                .get("avatar")
                .and_then(|a| a.get("image_url"))
                .and_then(Value::as_str)
            {
                if let Some(flat_obj) = flat.as_object_mut() {
                    flat_obj.insert(
                        "profile_image_url_https".to_owned(),
                        Value::String(url.to_owned()),
                    );
                }
            }
        }

        // Fill location
        if flat
            .get("location")
            .and_then(Value::as_str)
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            if let Some(loc) = obj
                .get("location")
                .and_then(|l| l.get("location"))
                .and_then(Value::as_str)
            {
                if let Some(flat_obj) = flat.as_object_mut() {
                    flat_obj.insert("location".to_owned(), Value::String(loc.to_owned()));
                }
            }
        }

        // Fill protected
        if !flat
            .as_object()
            .map(|o| o.contains_key("protected"))
            .unwrap_or(false)
        {
            if let Some(prot) = obj
                .get("privacy")
                .and_then(|p| p.get("protected"))
                .and_then(Value::as_bool)
            {
                if let Some(flat_obj) = flat.as_object_mut() {
                    flat_obj.insert("protected".to_owned(), Value::Bool(prot));
                }
            }
        }

        // Fill verified
        if !flat
            .as_object()
            .map(|o| o.contains_key("verified"))
            .unwrap_or(false)
        {
            if let Some(ver) = obj
                .get("verification")
                .and_then(|v| v.get("verified"))
                .and_then(Value::as_bool)
            {
                if let Some(flat_obj) = flat.as_object_mut() {
                    flat_obj.insert("verified".to_owned(), Value::Bool(ver));
                }
            }
        }

        // Fill is_blue_verified
        if !flat
            .as_object()
            .map(|o| o.contains_key("is_blue_verified"))
            .unwrap_or(false)
        {
            if let Some(blue) = obj.get("is_blue_verified").and_then(Value::as_bool) {
                if let Some(flat_obj) = flat.as_object_mut() {
                    flat_obj.insert("is_blue_verified".to_owned(), Value::Bool(blue));
                }
            }
        }

        // Fill description
        if flat
            .get("description")
            .and_then(Value::as_str)
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            if let Some(desc) = obj
                .get("profile_bio")
                .and_then(|b| b.get("description"))
                .and_then(Value::as_str)
            {
                if let Some(flat_obj) = flat.as_object_mut() {
                    flat_obj.insert("description".to_owned(), Value::String(desc.to_owned()));
                }
            }
        }

        // Set defaults
        if let Some(flat_obj) = flat.as_object_mut() {
            flat_obj
                .entry("description".to_owned())
                .or_insert_with(|| Value::String(String::new()));
            flat_obj
                .entry("location".to_owned())
                .or_insert_with(|| Value::String(String::new()));
            flat_obj
                .entry("followers_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("friends_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("statuses_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("favourites_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("listed_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("media_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("profile_image_url_https".to_owned())
                .or_insert_with(|| Value::String(String::new()));
            flat_obj
                .entry("entities".to_owned())
                .or_insert_with(|| Value::Object(Default::default()));
            flat_obj
                .entry("pinned_tweet_ids_str".to_owned())
                .or_insert_with(|| Value::Array(vec![]));
        }

        flat
    }

    fn flatten_tweet_v2(obj: &Value) -> Value {
        let legacy = obj.get("legacy").unwrap_or(&Value::Null).clone();
        let mut flat = merge_legacy(obj, &legacy);

        let rest_id = flat
            .get("rest_id")
            .or_else(|| flat.get("id_str"))
            .and_then(Value::as_str)
            .map(|s| s.to_owned())
            .or_else(|| {
                obj.get("rest_id")
                    .or_else(|| obj.get("id_str"))
                    .and_then(Value::as_str)
                    .map(|s| s.to_owned())
            });

        if let Some(ref rid) = rest_id {
            if let Some(obj) = flat.as_object_mut() {
                obj.insert("rest_id".to_owned(), Value::String(rid.clone()));
                obj.insert("id_str".to_owned(), Value::String(rid.clone()));
                obj.insert(
                    "id".to_owned(),
                    Value::Number(rid.parse::<i64>().unwrap_or(0).into()),
                );
                obj.insert("legacy".to_owned(), Value::Null);
            }
        }

        // Fill source
        if !flat
            .as_object()
            .map(|o| o.contains_key("source"))
            .unwrap_or(false)
        {
            if let Some(source) = obj.get("source").and_then(Value::as_str) {
                if let Some(flat_obj) = flat.as_object_mut() {
                    flat_obj.insert("source".to_owned(), Value::String(source.to_owned()));
                }
            }
        }

        // Set defaults
        if let Some(flat_obj) = flat.as_object_mut() {
            flat_obj
                .entry("full_text".to_owned())
                .or_insert_with(|| Value::String(String::new()));
            flat_obj
                .entry("lang".to_owned())
                .or_insert_with(|| Value::String(String::new()));
            flat_obj
                .entry("reply_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("retweet_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("favorite_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("quote_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("bookmark_count".to_owned())
                .or_insert_with(|| Value::Number(0.into()));
            flat_obj
                .entry("entities".to_owned())
                .or_insert_with(|| Value::Object(Default::default()));
            let conv_id = flat_obj
                .get("id_str")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            flat_obj
                .entry("conversation_id_str".to_owned())
                .or_insert_with(|| Value::String(conv_id));
        }

        flat
    }

    fn to_old_obj(obj: &Value) -> Value {
        if !obj.is_object() {
            return obj.clone();
        }
        if obj.get("__typename").and_then(Value::as_str) == Some("User") {
            return flatten_user_v2(obj);
        }
        flatten_tweet_v2(obj)
    }

    fn to_old_user(obj: &Value) -> Option<Value> {
        let legacy = obj
            .get("legacy")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        let core = obj
            .get("core")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));

        let mut legacy = legacy;
        if let (Some(leg_obj), Some(core_obj)) = (legacy.as_object_mut(), core.as_object()) {
            for key in ["screen_name", "name", "created_at"] {
                if !leg_obj.contains_key(key) {
                    if let Some(val) = core_obj.get(key) {
                        leg_obj.insert(key.to_owned(), val.clone());
                    }
                }
            }
        }

        if legacy
            .get("screen_name")
            .and_then(Value::as_str)
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            return None;
        }

        // Set defaults
        if let Some(leg_obj) = legacy.as_object_mut() {
            let defaults = [
                ("description", ""),
                ("followers_count", "0"),
                ("friends_count", "0"),
                ("statuses_count", "0"),
                ("favourites_count", "0"),
                ("listed_count", "0"),
                ("media_count", "0"),
                ("location", ""),
                ("profile_image_url_https", ""),
            ];
            for (key, default) in defaults {
                leg_obj
                    .entry(key.to_owned())
                    .or_insert_with(|| match default {
                        "0" => Value::Number(0.into()),
                        _ => Value::String(default.to_owned()),
                    });
            }
        }

        let rest_id = obj
            .get("rest_id")
            .or_else(|| obj.get("id_str"))
            .cloned()
            .unwrap_or(Value::String(String::new()));

        let mut merged = obj.clone();
        if let Some(obj) = merged.as_object_mut() {
            obj.insert("legacy".to_owned(), legacy);
            obj.insert("rest_id".to_owned(), rest_id);
        }

        Some(to_old_obj(&merged))
    }

    // Main logic
    let mut typed = HashMap::new();
    get_typed_objects(obj, &mut typed);

    let mut tweets = serde_json::Map::new();

    // Tweet objects
    for tw in typed.get("Tweet").unwrap_or(&vec![]) {
        if tw.get("legacy").is_some() && !tw.get("legacy").unwrap_or(&Value::Null).is_null() {
            if let Some(rest_id) = tw.get("rest_id").and_then(Value::as_str) {
                tweets.insert(rest_id.to_owned(), to_old_obj(tw));
            }
        }
    }

    // TweetWithVisibilityResults
    for tw in typed.get("TweetWithVisibilityResults").unwrap_or(&vec![]) {
        if let Some(tweet) = tw.get("tweet") {
            if tweet.get("legacy").is_some()
                && !tweet.get("legacy").unwrap_or(&Value::Null).is_null()
            {
                if let Some(rest_id) = tweet.get("rest_id").and_then(Value::as_str) {
                    tweets.insert(rest_id.to_owned(), to_old_obj(tweet));
                }
            }
        }
    }

    let mut users = serde_json::Map::new();
    for u in typed.get("User").unwrap_or(&vec![]) {
        if let Some(user) = to_old_user(u) {
            if let Some(id_str) = user.get("id_str").and_then(Value::as_str) {
                users.insert(id_str.to_owned(), user);
            }
        }
    }

    let mut trends = serde_json::Map::new();
    for t in typed.get("TimelineTrend").unwrap_or(&vec![]) {
        if let Some(name) = t.get("name").and_then(Value::as_str) {
            trends.insert(name.to_owned(), t.clone());
        }
    }

    let mut result = serde_json::Map::new();
    result.insert("tweets".to_owned(), Value::Object(tweets));
    result.insert("users".to_owned(), Value::Object(users));
    result.insert("trends".to_owned(), Value::Object(trends));

    Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/mocked-data")
    }

    fn load_json(filename: &str) -> Value {
        let path = data_dir().join(filename);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
        serde_json::from_str(&text).unwrap()
    }

    fn load_and_parse_tweets(filename: &str) -> Vec<Tweet> {
        let raw = load_json(filename);
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if seen.insert(tweet.id) {
                        result.push(tweet);
                    }
                }
            }
        }
        result
    }

    fn check_tweet(doc: &Tweet) {
        assert!(doc.id != 0, "id should not be 0");
        assert!(!doc.id_str.is_empty(), "id_str should not be empty");
        assert_eq!(doc.id.to_string(), doc.id_str, "id and id_str should match");
        assert!(doc.url.contains(&doc.id_str), "url should contain id_str");
        assert!(doc.user.is_some(), "user should be Some");
        assert!(doc.conversation_id != 0, "conversationId should not be 0");
        assert_eq!(doc.conversation_id.to_string(), doc.conversation_id_str);

        if let Some(reply_id) = doc.in_reply_to_tweet_id {
            assert!(doc.in_reply_to_tweet_id_str.is_some());
            assert_eq!(
                reply_id.to_string(),
                *doc.in_reply_to_tweet_id_str.as_ref().unwrap()
            );
        }

        if let Some(ref user) = doc.user {
            check_user(user);
        }

        assert!(doc.bookmarked_count >= 0);
    }

    fn check_user(doc: &User) {
        assert!(doc.id != 0, "user id should not be 0");
        assert!(!doc.id_str.is_empty());
        assert_eq!(doc.id.to_string(), doc.id_str);
        assert!(!doc.username.is_empty());
        assert!(!doc.profile_image_url.is_empty());
    }

    #[allow(dead_code)]
    fn check_user_ref(doc: &UserRef) {
        assert!(doc.id != 0);
        assert_eq!(doc.id.to_string(), doc.id_str);
        assert!(!doc.username.is_empty());
        assert!(!doc.displayname.is_empty());
    }

    fn check_trend(doc: &Trend) {
        assert!(!doc.name.is_empty());
        assert!(!doc.trend_url.url.is_empty());
        assert!(!doc.trend_url.url_type.is_empty());
    }

    // --- test_parser.py equivalent tests ---

    #[test]
    fn test_search() {
        let raw = load_json("raw_search.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let mut items = Vec::new();
        let mut seen = std::collections::HashSet::new();
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if seen.insert(tweet.id) {
                        items.push(tweet);
                    }
                }
            }
        }

        assert!(
            !items.is_empty(),
            "should parse at least one tweet from search"
        );

        let mut bookmarks_count = 0i64;
        let mut all_users = Vec::new();
        for doc in &items {
            check_tweet(doc);
            bookmarks_count += doc.bookmarked_count;
            if let Some(ref user) = doc.user {
                all_users.push(user.clone());
            }
            collect_tweet_users(doc, &mut all_users);
        }

        assert!(
            bookmarks_count > 0,
            "bookmark_fields key is changed or unlucky search data"
        );

        // Check user field coverage
        assert!(!all_users.is_empty());
        assert!(all_users.iter().any(|u| !u.location.is_empty()));
        assert!(all_users.iter().any(|u| u.profile_banner_url.is_some()));
    }

    fn collect_tweet_users(doc: &Tweet, users: &mut Vec<User>) {
        if let Some(ref rt) = doc.retweeted_tweet {
            if let Some(ref user) = rt.user {
                users.push(user.clone());
            }
            collect_tweet_users(rt, users);
        }
        if let Some(ref qt) = doc.quoted_tweet {
            if let Some(ref user) = qt.user {
                users.push(user.clone());
            }
            collect_tweet_users(qt, users);
        }
    }

    #[test]
    fn test_user_by_login() {
        let raw = load_json("raw_user_by_login.json");
        let rep = to_old_rep(&raw);
        let users = rep.get("users").unwrap();

        let mut found_user = None;
        if let Some(users_obj) = users.as_object() {
            for (_key, user_obj) in users_obj {
                if let Some(user) = User::parse(user_obj) {
                    found_user = Some(user);
                    break;
                }
            }
        }

        let doc = found_user.expect("should parse a user");
        assert_eq!(doc.id, 2244994945);
        assert_eq!(doc.username, "XDevelopers");
        assert_eq!(doc.id.to_string(), doc.id_str);
    }

    #[test]
    fn test_tweet_details() {
        let raw = load_json("raw_tweet_details.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let target_id: i64 = 1649191520250245121;
        let mut found = None;
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if tweet.id == target_id {
                        found = Some(tweet);
                        break;
                    }
                }
            }
        }

        let doc = found.expect("should find target tweet");
        check_tweet(&doc);
        assert_eq!(doc.id, target_id);
        assert!(doc.user.is_some());
    }

    #[test]
    fn test_trends() {
        let raw = load_json("raw_trends.json");
        let rep = to_old_rep(&raw);
        let trends = rep.get("trends").unwrap();

        let mut items = Vec::new();
        if let Some(trends_obj) = trends.as_object() {
            for (_key, trend_obj) in trends_obj {
                if let Some(trend) = Trend::parse(trend_obj) {
                    items.push(trend);
                }
            }
        }

        assert!(!items.is_empty(), "should parse at least one trend");
        for doc in &items {
            check_trend(doc);
        }
    }

    #[test]
    fn test_followers() {
        let raw = load_json("raw_followers.json");
        let rep = to_old_rep(&raw);
        let users = rep.get("users").unwrap();

        let mut items = Vec::new();
        if let Some(users_obj) = users.as_object() {
            for (_key, user_obj) in users_obj {
                if let Some(user) = User::parse(user_obj) {
                    items.push(user);
                }
            }
        }

        assert!(!items.is_empty(), "should parse at least one follower");
        for doc in &items {
            check_user(doc);
        }
    }

    #[test]
    fn test_following() {
        let raw = load_json("raw_following.json");
        let rep = to_old_rep(&raw);
        let users = rep.get("users").unwrap();

        let mut items = Vec::new();
        if let Some(users_obj) = users.as_object() {
            for (_key, user_obj) in users_obj {
                if let Some(user) = User::parse(user_obj) {
                    items.push(user);
                }
            }
        }

        assert!(!items.is_empty(), "should parse at least one following");
        for doc in &items {
            check_user(doc);
        }
    }

    #[test]
    fn test_user_tweets() {
        let items = load_and_parse_tweets("raw_user_tweets.json");
        assert!(!items.is_empty(), "should parse at least one tweet");
        for doc in &items {
            check_tweet(doc);
        }
    }

    #[test]
    fn test_user_tweets_and_replies() {
        let items = load_and_parse_tweets("raw_user_tweets_and_replies.json");
        assert!(!items.is_empty(), "should parse at least one tweet");
        for doc in &items {
            check_tweet(doc);
        }
    }

    #[test]
    fn test_tweet_replies() {
        let items = load_and_parse_tweets("raw_tweet_replies.json");
        assert!(!items.is_empty(), "should parse at least one reply");
        let twid = 1649191520250245121_i64;
        // Filter to only actual replies (same as Python api.tweet_replies which filters by inReplyToTweetId)
        let replies: Vec<_> = items
            .iter()
            .filter(|t| t.in_reply_to_tweet_id == Some(twid))
            .collect();
        assert!(
            !replies.is_empty(),
            "should have at least one reply to {}",
            twid
        );
        for doc in &replies {
            check_tweet(doc);
            assert_eq!(doc.in_reply_to_tweet_id, Some(twid));
        }
    }

    #[test]
    fn test_tweet_thread() {
        let items = load_and_parse_tweets("raw_tweet_thread.json");
        assert!(
            !items.is_empty(),
            "should parse at least one tweet in thread"
        );
        let twid = 1649191520250245121_i64;
        for doc in &items {
            check_tweet(doc);
            assert_eq!(doc.conversation_id, twid);
        }
    }

    #[test]
    fn test_user_media() {
        let items = load_and_parse_tweets("raw_user_media.json");
        assert!(!items.is_empty(), "should parse at least one media tweet");
        for doc in &items {
            check_tweet(doc);
            assert!(doc.media.is_some(), "media should be Some");
            let media = doc.media.as_ref().unwrap();
            let media_count = media.photos.len() + media.videos.len() + media.animated.len();
            assert!(media_count > 0, "{} should have media", doc.url);
        }
    }

    #[test]
    fn test_list_timeline() {
        let items = load_and_parse_tweets("raw_list_timeline.json");
        assert!(
            !items.is_empty(),
            "should parse at least one tweet from list"
        );
        for doc in &items {
            check_tweet(doc);
        }
    }

    #[test]
    fn test_community_info() {
        let raw = load_json("raw_community_info.json");
        let community = raw
            .pointer("/data/communityResults/result")
            .expect("should find community data");

        let id_str = community
            .get("rest_id")
            .or_else(|| community.get("id_str"))
            .and_then(Value::as_str)
            .unwrap_or("");
        assert!(!id_str.is_empty(), "community should have rest_id");
        assert!(
            community.get("name").is_some(),
            "community should have name"
        );
        let member_count = community
            .get("member_count")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        assert!(member_count > 0, "memberCount should be > 0");
    }

    #[test]
    fn test_community_members() {
        let raw = load_json("raw_community_members.json");
        let rep = to_old_rep(&raw);
        let users = rep.get("users").unwrap();

        let mut items = Vec::new();
        if let Some(users_obj) = users.as_object() {
            for (_key, user_obj) in users_obj {
                if let Some(user) = User::parse(user_obj) {
                    items.push(user);
                }
            }
        }

        assert!(
            !items.is_empty(),
            "should parse at least one community member"
        );
        for doc in &items {
            check_user(doc);
        }
    }

    #[test]
    fn test_community_tweets() {
        let items = load_and_parse_tweets("raw_community_tweets.json");
        assert!(
            !items.is_empty(),
            "should parse at least one community tweet"
        );
        for doc in &items {
            check_tweet(doc);
        }
    }

    #[test]
    fn test_issue_28_retweet_views() {
        let raw = load_json("_issue_28_1.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let target_id: i64 = 1658409412799737856;
        let mut found = None;
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if tweet.id == target_id {
                        found = Some(tweet);
                        break;
                    }
                }
            }
        }

        let doc = found.expect("should find issue 28 tweet");
        check_tweet(&doc);
        assert_eq!(doc.id, target_id);
        assert!(doc.user.is_some());
        assert!(doc.retweeted_tweet.is_some(), "should have retweeted tweet");
        let rt = doc.retweeted_tweet.as_ref().unwrap();
        assert!(
            rt.view_count.is_some(),
            "retweeted tweet should have view count"
        );
        assert_eq!(
            doc.view_count, rt.view_count,
            "views should come from retweeted tweet"
        );
        check_tweet(rt);
    }

    #[test]
    fn test_issue_28_quoted_views() {
        let raw = load_json("_issue_28_2.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let target_id: i64 = 1658421690001502208;
        let mut found = None;
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if tweet.id == target_id {
                        found = Some(tweet);
                        break;
                    }
                }
            }
        }

        let doc = found.expect("should find issue 28 quoted tweet");
        check_tweet(&doc);
        assert_eq!(doc.id, target_id);
        assert!(doc.view_count.is_some());
        assert!(doc.quoted_tweet.is_some(), "should have quoted tweet");
        let qt = doc.quoted_tweet.as_ref().unwrap();
        assert_ne!(qt.id, doc.id);
        check_tweet(qt);
        assert!(qt.view_count.is_some());
    }

    #[test]
    fn test_issue_42_retweet_content() {
        let raw = load_json("_issue_42.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let target_id: i64 = 1665951747842641921;
        let mut found = None;
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if tweet.id == target_id {
                        found = Some(tweet);
                        break;
                    }
                }
            }
        }

        let doc = found.expect("should find issue 42 tweet");
        assert!(doc.retweeted_tweet.is_some());
        let rt = doc.retweeted_tweet.as_ref().unwrap();
        assert!(!doc.raw_content.is_empty());
        assert!(!rt.raw_content.is_empty());
        assert!(
            doc.raw_content.ends_with(&rt.raw_content),
            "content should be full: '{}' should end with '{}'",
            doc.raw_content,
            rt.raw_content
        );
    }

    #[test]
    fn test_issue_56_unique_links() {
        let raw = load_json("_issue_56.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let target_id: i64 = 1682072224013099008;
        let mut found = None;
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if tweet.id == target_id {
                        found = Some(tweet);
                        break;
                    }
                }
            }
        }

        let doc = found.expect("should find issue 56 tweet");
        let tcourls: Vec<_> = doc.links.iter().filter_map(|l| l.tcourl.as_ref()).collect();
        let unique: std::collections::HashSet<_> = tcourls.iter().collect();
        assert_eq!(tcourls.len(), unique.len(), "all tcourl should be unique");
        assert_eq!(doc.links.len(), 5, "should have 5 links");
    }

    #[test]
    fn test_cards_summary() {
        let raw = load_json("card_summary.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let target_id: i64 = 1696922210588410217;
        let mut found = None;
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if tweet.id == target_id {
                        found = Some(tweet);
                        break;
                    }
                }
            }
        }

        let doc = found.expect("should find summary card tweet");
        assert!(doc.card.is_some(), "card should be Some");
        match doc.card.as_ref().unwrap() {
            Card::Summary {
                title,
                description,
                url,
            } => {
                assert!(title.is_some());
                assert!(description.is_some());
                assert!(url.is_some());
            }
            other => panic!("expected Summary card, got {:?}", other),
        }
    }

    #[test]
    fn test_cards_poll() {
        let raw = load_json("card_poll.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let target_id: i64 = 1780666831310877100;
        let mut found = None;
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if tweet.id == target_id {
                        found = Some(tweet);
                        break;
                    }
                }
            }
        }

        let doc = found.expect("should find poll card tweet");
        assert!(doc.card.is_some());
        match doc.card.as_ref().unwrap() {
            Card::Poll { finished, options } => {
                assert!(finished.is_some());
                assert!(!options.is_empty());
                for opt in options {
                    assert!(!opt.label.is_empty());
                    assert!(!opt.votes_count.is_empty());
                }
            }
            other => panic!("expected Poll card, got {:?}", other),
        }
    }

    #[test]
    fn test_cards_broadcast() {
        let raw = load_json("card_broadcast.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let target_id: i64 = 1790441814857826439;
        let mut found = None;
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if tweet.id == target_id {
                        found = Some(tweet);
                        break;
                    }
                }
            }
        }

        let doc = found.expect("should find broadcast card tweet");
        assert!(doc.card.is_some());
        match doc.card.as_ref().unwrap() {
            Card::Broadcast { title, url, photo } => {
                assert!(title.is_some());
                assert!(url.is_some());
                assert!(photo.is_some());
            }
            other => panic!("expected Broadcast card, got {:?}", other),
        }
    }

    #[test]
    fn test_cards_audiospace() {
        let raw = load_json("card_audiospace.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let target_id: i64 = 1789054061729173804;
        let mut found = None;
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if tweet.id == target_id {
                        found = Some(tweet);
                        break;
                    }
                }
            }
        }

        let doc = found.expect("should find audiospace card tweet");
        assert!(doc.card.is_some());
        match doc.card.as_ref().unwrap() {
            Card::Audiospace { url } => {
                assert!(url.is_some());
            }
            other => panic!("expected Audiospace card, got {:?}", other),
        }
    }

    #[test]
    fn test_tweet_new_fields() {
        let raw = load_json("raw_search.json");
        let rep = to_old_rep(&raw);
        let tweets = rep.get("tweets").unwrap();
        let users = rep.get("users").unwrap();
        let res = serde_json::json!({"tweets": tweets, "users": users});

        let mut items = Vec::new();
        let mut seen = std::collections::HashSet::new();
        if let Some(tweets_obj) = tweets.as_object() {
            for (_key, tweet_obj) in tweets_obj {
                if let Some(tweet) = Tweet::parse(tweet_obj, &res) {
                    if seen.insert(tweet.id) {
                        items.push(tweet);
                    }
                }
            }
        }

        assert!(!items.is_empty());

        for doc in &items {
            // isQuoteStatus is always a bool
            assert!(doc.is_quote_status == true || doc.is_quote_status == false);

            // isTranslatable is always a bool
            assert!(doc.is_translatable == true || doc.is_translatable == false);

            if let Some(ref range) = doc.display_text_range {
                assert_eq!(range.len(), 2);
            }

            if let Some(ref screen_name) = doc.in_reply_to_screen_name {
                assert!(!screen_name.is_empty());
            }
        }

        assert!(
            items.iter().any(|d| d.is_quote_status),
            "expected at least one quote tweet"
        );
        assert!(
            items.iter().any(|d| d.in_reply_to_screen_name.is_some()),
            "expected at least one reply tweet"
        );
        assert!(
            items.iter().any(|d| d.edit_control.is_some()),
            "expected editControl in at least one tweet"
        );
        assert!(
            items.iter().any(|d| d.display_text_range.is_some()),
            "expected displayTextRange in at least one tweet"
        );
    }
}
