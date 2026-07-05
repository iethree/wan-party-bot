//! Port of `get_error_message.py`.

use crate::text_util::read_lines_keepends;
use once_cell::sync::Lazy;
use rand::Rng;

// Python re-opens the file every call, but the contents are static; cache once.
// `readlines()` keeps the trailing newline on each line, and the original returns
// the line verbatim (newline included), so we preserve that.
static LINES: Lazy<Vec<String>> = Lazy::new(|| read_lines_keepends("data/error_messages.txt"));

#[allow(non_snake_case)]
pub fn get_error_message_for_fun_times_everyone_loves_error_messages() -> String {
    let lines = &*LINES;
    let mut rng = rand::thread_rng();
    // random.randint(0, len(lines) - 1) — inclusive on both ends.
    let idx = rng.gen_range(0..=lines.len() - 1);
    lines[idx].clone()
}
