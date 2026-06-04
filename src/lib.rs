mod account;
mod api;
pub mod db;
pub mod logger;
pub mod queue_client;
pub mod types;
mod utils;
pub mod xclid;

// Public re-exports of pure Rust API
pub use account::{
    AUTH_TOKEN, account_delete_inactive_sql, account_delete_sql, account_get_and_lock_sql,
    account_get_for_queue_sql, account_lock_until_sql, account_login_select_sql,
    account_mark_inactive_sql, account_next_available_sql, account_relogin_failed_sql,
    account_relogin_sql, account_reset_locks_sql, account_save_sql, account_select_all_sql,
    account_select_by_tx_sql, account_select_by_username_sql, account_set_active_sql,
    account_stats_sql, account_unlock_sql, guess_delim_rs, json_string_map,
    parse_account_file_lines_rs, selected_proxy, stats_map,
};
pub use api::{
    api_build_features_rs, api_build_gql_params_rs, api_build_kv_rs, api_build_params_rs,
    api_endpoint_config_rs, api_get_cursor_rs, api_gql_entries_rs, api_gql_op_rs, api_gql_queue_rs,
    api_gql_url_rs, api_is_end_rs, api_merge_features_rs,
};
pub use db::{
    db_check_sqlite_version, db_latest_migration_version, db_migration_sql, db_min_sqlite_version,
};
pub use logger::{init_log_level, log_filter, set_log_level};
pub use queue_client::{queue_req_id, queue_response_state};
pub use types::{
    Card, Coordinates, GroupedTrend, Media, MediaAnimated, MediaPhoto, MediaVideo,
    MediaVideoVariant, Place, PollOption, TextLink, Trend, TrendMetadata, TrendUrl, Tweet, User,
    UserRef, to_old_rep,
};
pub use utils::{get_env_bool, parse_cookies_rs, parse_proxy};
pub use xclid::{
    cacl_anim_key, calc_anim_key_from_indices, get_scripts_list, parse_anim_arr_from_paths,
    parse_anim_idx_from_script, parse_vk_bytes_from_content, xclid_calc,
};
