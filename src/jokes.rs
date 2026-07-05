//! Port of `jokes.py`. Loaded but unused by the live bot; reproduced faithfully.

use crate::text_util::read_text;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use serde::Deserialize;

#[derive(Deserialize)]
struct Joke {
    setup: String,
    punchline: String,
}

static JOKES: Lazy<Vec<Joke>> = Lazy::new(|| {
    let text = read_text("data/jokes.json");
    serde_json::from_str(&text).expect("data/jokes.json should be valid JSON")
});

pub fn random_joke() -> String {
    let mut rng = rand::thread_rng();
    let joke = JOKES.choose(&mut rng).expect("jokes list is non-empty");
    format!("{}\n\n ||{}||", joke.setup, joke.punchline)
}

const DECKS: [&str; 3] = [
    "https://giphy.com/clips/betplus-bet-plus-the-ms-pat-show-APfllllIzLozTEBwbM",
    "https://media.giphy.com/media/MGP5hVgOpcbaVzrV38/giphy.gif",
    "https://media.giphy.com/media/MGP5hVgOpcbaVzrV38/giphy.gif",
];

pub fn hit_the_deck() -> String {
    let mut rng = rand::thread_rng();
    format!("Did someone say DECK??\n{}", DECKS.choose(&mut rng).unwrap())
}

const DONKS: [&str; 2] = [
    "https://i.imgur.com/4OmIo0j.png",
    "https://i.imgur.com/4V97LRB.png",
];

pub fn hit_the_donk() -> String {
    let mut rng = rand::thread_rng();
    format!("Did someone say DONK??\n{}", DONKS.choose(&mut rng).unwrap())
}
