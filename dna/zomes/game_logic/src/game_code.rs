// Import most commonly used tools for Holochain development
use hdk::prelude::*;

// Since we'll be using a hardcoded string value to access all game code,
// we'd better declare it as a constant to be re-used
// Note: we're using &str instead of String type here because size of this string
// is known at compile time, so there's no need to allocate memory dynamically
// by using String.
// More about &str and String difference here:
// https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#the-string-type
pub const GAME_CODES_ANCHOR: &str = "GAME_CODES";

/// Creates anchor for a new game identified by the short_unique_code
/// and registers it under GAME_CODES_ANCHOR to be discoverable
/// ExternResult here is just a Holochain version of the standard enum Result in Rust
/// which is used for handling errors. ExternResult has pretty much the same dev experience.
/// For more details about Result see:
/// https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#recoverable-errors-with-result
pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHash> {
    // anchor is a helper function which does the following boilerplate work for us:
    // 1) create entry with contents of GAME_CODES_ANCHOR
    // 2) create entry with contents of short_unique_code
    // 3) create a link from entry1 to entry2
    // 4) return hash of entry2
    let anchor = anchor(GAME_CODES_ANCHOR.into(), short_unique_code)?;
    // Note the lack of ; in the end of the next code line: this is the value we return here
    // More on that syntax here:
    // https://doc.rust-lang.org/stable/book/ch03-03-how-functions-work.html#functions-with-return-values
    // Since the return value of our fn is an ExternResult, we're wrapping our
    // anchor (which is an entry hash) into the Ok() variant of ExternResult
    Ok(anchor)
}

/// Retrieves entry hash of the game code anchor that corresponds
/// to the game_code provided
pub fn get_game_code_anchor(game_code: String) -> ExternResult<EntryHash> {
    // NOTE: That's actually a very hacky way to do things. This fn always does a write to DHT,
    // so every time it's called a Header will be created for another instance of the anchor.
    // The end result of this write is still an entry hash, which is what we need.
    // This doesn't break a small game app, but isn't a good idea for a production usage.
    // That should be refactored into actual retrieval using `get_anchor` fn
    anchor(GAME_CODES_ANCHOR.into(), game_code.clone())
}
