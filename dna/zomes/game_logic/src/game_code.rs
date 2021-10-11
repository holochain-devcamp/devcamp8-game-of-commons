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
    // NOTE: This fn always does a write to DHT,
    // so every time it's called a Header will be created for another instance of the anchor.
    // The end result of this write is still an entry hash, which is what we need.
    // Depending on your app this may or may not be the right choice. In the next section you
    // can read why, for this app, we made this choice.

    /*
        When you create an anchor with the anchor() fn this function return a EntryHash. Once you
        know the entry_hash of an anchor it is best to use the get_anchor(entry_hash) fn to retrieve
        this anchor, when you need it. In the case of the devcamp game, we have a little problem.
        Players share the game_code via chat or voice or video     That means that the player who
        initiated the game, the game leader, knows the entry_hash of the game code, but players that
        want to join the game do not. Other players need to be able to find the same anchor if they
        want to join the game. Of course the game leader could communicate the entry hash, but that
        is not as convenient as passing the much shorter game code.
        So for other players that do not have the game code the problem exists in finding out the
        entry hash of the anchor while they only have game code.

    There are 2 approaches you can take to solve this problem, each with it own benefits.
    1/ Other players can take the game_code and calculate the hash, without actually creating
        a anchor in the DHT (with the same entry hash, but a different header hash).
    Benefits: less DHT operations, no extra header in the DHT
    Downside: calculating the entry_hash and fetching the anchor with this hash via 'get_anchor',
                  does not guarantee that anchor will be found at the point in time that you start
                  searching it. Even if you have a entry_hash of entry that absolutely, 100% exists.
                  It does not guarantee it can be found in your part of the DHT, yet. Eventually it
                  will be.The downside is you to need poll until you find the anchor. This how you
                  could calculate a entry hash:
         let path: Path = (&Anchor {
                anchor_type: GAME_CODES_ANCHOR.into(),
                anchor_text: Some(game_code),
            })
                .into();
            let anchor_hash = path.hash()
        2/ The other way it for the other players to create the same anchor. The anchor entry will
        create again. It will add a header and a entry to the DHT. But since the entry has the same
        entry_hash it will not be stored again.
        Benefit: entry is added to in your source chain before being sent to the DHT, so it is
        immediately available. No polling needed
        Downside: More DHT ops, extra header in the DHT

        In the devcamp game we choose option 2: each player creates the anchor.
        */
        let path: Path = (&Anchor {
            anchor_type: GAME_CODES_ANCHOR.into(),
            anchor_text: Some(game_code),
        })
            .into();
        path.hash()
}
