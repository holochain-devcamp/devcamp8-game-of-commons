use hdk::prelude::*;

// We apply macro to the the line of code that follows, allowing to compile code
// from the game_code module even if some of this code is unused.
// (Default compiler's behavior is to treat unused code as errors)
// We'll remove this line later once we start using all code from the game_code module.
// "unused" here is name of the lint group, and there are actually a lot of those!
// Check out this link for more details:
// https://doc.rust-lang.org/rustc/lints/groups.html
#[allow(unused)]
mod game_code;

// This is part of Holochain data model definition, and here we specify
// what kinds of entries are available in our applicaton.
entry_defs![
    // Our implementation of game_code uses `anchor` helper method,
    // which requires us to add the Anchor and Path entry definitions
    Anchor::entry_def(),
    Path::entry_def()
];
/// This is another macro applied to the function that follows, and we need it to
/// expose this function as part of our backend API
/// Note that this macro requires fn to accept input parameters, so if your fn
/// doesn't accept anything, write it's signature like this:
/// ```
/// #[hdk_extern]
/// fn foo(_: ()) -> ExternResult<EntryHash>
/// ```
/// This function is part of our publicly exposed API and it simply wraps
/// the corresponding function in game_code module.
#[hdk_extern]
pub fn create_game_code_anchor(short_unique_code: String) -> ExternResult<EntryHash> {
    game_code::create_game_code_anchor(short_unique_code)
}