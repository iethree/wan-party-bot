//! Port of `blacklist.py`.

/// The blacklisted channel names (compared case-insensitively).
pub const BLACKLIST: [&str; 5] = [
    "sigh-politics",
    "bible",
    "wanglicanism",
    "formative movie crushes of the youthful era",
    "dads",
];

/// `is_blacklisted_channel(channel_name)`.
///
/// Python: `if not channel_name: return False` (covers both `None` and the empty
/// string), then `return channel_name.lower() in blacklist`.
pub fn is_blacklisted_channel(channel_name: Option<&str>) -> bool {
    match channel_name {
        None | Some("") => false,
        Some(name) => BLACKLIST.contains(&name.to_lowercase().as_str()),
    }
}
