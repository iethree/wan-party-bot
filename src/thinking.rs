//! Port of `thinking.py` — the rotating "loading" messages and the spinner loop.
//! Reachable only via the never-synced `/loading` and `/leaderboards` commands.

use rand::seq::SliceRandom;
use serenity::all::{ChannelId, Context, CreateMessage, EditMessage};
use std::time::Duration;

pub const THINKING_MESSAGES: [&str; 129] = [
    "Fetching data...",
    "I'm thinking...",
    "Still thinking...",
    "Analyzing data...",
    "Getting snacks...",
    "Reticulating splines...",
    "Generating witty dialog...",
    "Swapping time and space...",
    "Spinning violently around the y-axis...",
    "Tokenizing real life...",
    "Bending the spoon...",
    "Filtering morale...",
    "Gathering purple hippos...",
    "We need a new fuse...",
    "640K ought to be enough for anybody",
    "The architects are still drafting...",
    "Breeding some more bits...",
    "Waiting for the elves to clock in...",
    "Testing patience...",
    "Encouraging the penguins...",
    "Sharpening the knives...",
    "Answering questions...",
    "Reconfoobling energymotron...",
    "It's not you. It's me.",
    "Counting backwards from Infinity...",
    "Don't panic...",
    "Embiggening Prototypes...",
    "Creating time-loop inversion field...",
    "Spinning the wheel of fortune...",
    "Loading the enchanted bunny...",
    "Computing chance of success...",
    "Looking for exact change...",
    "Pulling out my checkbook...",
    "I feel like im supposed to be loading something...",
    "Getting back on track...",
    "Should have used a compiled language...",
    "Adjusting flux capacitor...",
    "Shoving sloths...",
    "I swear it's almost done.",
    "Let's take a mindfulness minute...",
    "Listening for the sound of one hand clapping...",
    "Keeping all the 1's and removing all the 0's...",
    "Putting the icing on the cake. The cake is not a lie...",
    "Cleaning off the cobwebs...",
    "Making sure all the i's have dots...",
    "Aligning dilithium crystals...",
    "Connecting Neurotoxin Storage Tank...",
    "Granting wishes...",
    "Spinning the hamster…",
    "99 bottles of beer on the wall...",
    "Stay awhile and listen...",
    "Convincing AI not to turn evil...",
    "What's that smell?",
    "Constructing additional pylons...",
    "Roping some seaturtles...",
    "Locating Jebediah Kerman...",
    "We are not liable for any broken screens as a result of waiting.",
    "turning it off and on again...",
    "Well, this is embarrassing.",
    "Whoops...",
    "Shhh....",
    "Didn't know paint dried so quickly.",
    "Walking several dogs...",
    "Organizing socks...",
    "Dividing by zero...",
    "Consulting chuck norris...",
    "Cracking military-grade encryption...",
    "Simulating traveling salesman...",
    "Proving P=NP...",
    "Entangling superstrings...",
    "Twiddling thumbs...",
    "Searching for plot device...",
    "Trying to sort in O(n)...",
    "Searching for more jokes...",
    "Waiting for the intern to get more coffee...",
    "convert this bug to a feature...",
    "Winter is coming...",
    "Updating dependencies...",
    "Installing dependencies...",
    "Uninstalling dependencies...",
    "Switching to the latest COBOL framework...",
    "Distracted by cat gifs...",
    "Finding someone to hold my beer...",
    "Aw, snap! Not..",
    "Ordering 1s and 0s...",
    "Consulting the manual...",
    "Feel free to spin in your chair",
    "Downloading more RAM...",
    "Mining some bitcoins...",
    "Downloading more RAM..",
    "Updating to Windows Vista...",
    "Deleting something...",
    "Initializing the initializer...",
    "Finishing the finisher...",
    "Optimizing the optimizer...",
    "Compiling the compiler...",
    "Updating Updater...",
    "Downloading Downloader...",
    "Debugging Debugger...",
    "Awaiting the awaiter...",
    "Googling it...",
    "Shovelling coal into the server...",
    "Pushing pixels...",
    "Building a wall...",
    "Reading Terms and Conditions for you...",
    "Running with scissors...",
    "Definitely not a virus...",
    "Work, work...",
    "Discovering new ways of making you wait...",
    "Your time is very important to us...",
    "Catching em' all...",
    "Finding elevator music...",
    "Grabbing extra minions...",
    "Warming up...",
    "Starting over...",
    "Resetting the machine...",
    "Doing the heavy lifting...",
    "We're working very Hard .... Really",
    "Waking up the minions",
    "Serve other customers...",
    "Our premium plan is faster",
    "Feeding unicorns...",
    "Rupturing the subspace barrier...",
    "Creating an anti-time reaction...",
    "Converging tachyon pulses...",
    "Bypassing control of the matter-antimatter integrator...",
    "Adjusting the dilithium crystal converter assembly...",
    "Reversing the shield polarity...",
    "Disrupting warp fields with an inverse graviton burst...",
];

/// Python's `round()` uses banker's rounding (round-half-to-even).
fn python_round(x: f64) -> i64 {
    let floor = x.floor();
    let diff = x - floor;
    if diff < 0.5 {
        floor as i64
    } else if diff > 0.5 {
        floor as i64 + 1
    } else {
        let f = floor as i64;
        if f % 2 == 0 {
            f
        } else {
            f + 1
        }
    }
}

/// `thinking(ctx, max_seconds=120, duration=2)`. Sends "let me think...", then
/// edits it with a random message every `duration` seconds, deleting it at the end.
///
/// Fidelity note: Python catches `asyncio.CancelledError` to delete the message on
/// cancellation. With tokio aborts the future is just dropped; the normal path
/// deletes here exactly like the original.
pub async fn thinking(ctx: &Context, channel_id: ChannelId, max_seconds: u64, duration: u64) {
    let sent = channel_id
        .send_message(&ctx.http, CreateMessage::new().content("let me think..."))
        .await;
    let mut msg = match sent {
        Ok(m) => m,
        Err(_) => return,
    };

    let iterations = python_round(max_seconds as f64 / duration as f64);
    for _ in 0..iterations {
        let new_msg = {
            let mut rng = rand::thread_rng();
            THINKING_MESSAGES.choose(&mut rng).copied().unwrap_or("")
        };
        tokio::time::sleep(Duration::from_secs(duration)).await;
        let _ = msg
            .edit(&ctx.http, EditMessage::new().content(new_msg))
            .await;
    }
    let _ = msg.delete(&ctx.http).await;
}
