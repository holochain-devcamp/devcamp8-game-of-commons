use hdk::prelude::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod game_code;
mod game_move;
mod game_round;
mod game_session;
mod game_signals;
mod player_profile;
mod utils;

pub use crate::{
    game_move::GameMoveInput,
    game_round::GameRoundInfo,
    game_session::GameSession,
    game_signals::GameSignal,
    player_profile::{JoinGameInfo, PlayerProfile},
};

// This is part of Holochain data model definition, and here we specify
// what kinds of entries are available in our applicaton.
entry_defs![
    // Our implementation of game_code uses `anchor` helper method,
    // which requires us to add the Anchor and Path entry definitions
    Anchor::entry_def(),
    Path::entry_def(),
    // PlayerProfile Holochain entry definition callback. You wouldn't find a fn
    // named entry_def in player_profile.rs: this is one of the functions
    // generated by applying `#[hdk_entry]` macro to PlayerProfile struct
    player_profile::PlayerProfile::entry_def(),
    // GameSession Holochain entry definition callback
    game_session::GameSession::entry_def(),
    // GameRound Holochain entry definition callback
    game_round::GameRound::entry_def(),
    // GameMove Holochain entry definition callback
    game_move::GameMove::entry_def()
];

#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    // ------ first, set up cap grants for signals to work
    // grant unrestricted access to accept_cap_claim so other agents can send us claims
    let mut functions: GrantedFunctions = BTreeSet::new();
    // give unrestricted access to recv_remote_signal, which is needed for sending remote signals
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));

    // Create the simplest capability grant entry
    create_cap_grant(CapGrantEntry {
        tag: "".into(),    // we wouldn't need to use it, so tag can be empty
        access: ().into(), // empty access converts to unrestricted
        functions,         // this grant would allow these functions to be called
    })?;

    // ------ second, set up a tracing config. This isn't related to the signals part in any way:
    // even more so -- this isn't Holochain specific! Just Rust tracing crate.
    // Holochain's init fn is basically an entry point into the hApp (which is the role main function
    // plays in Rust apps), so it's a good place for such setup to be done.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // configures formatter with defaults
        .with_target(false)
        .without_time()
        .compact()
        // completes the builder.
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // init is an important callback for every hApp, and it's return value is
    // critical for what happens next, so there are multiple different values defined.
    // If we get to this line, everything is going fine: we can safely pass the
    // init stage and move on to the next step.
    Ok(InitCallbackResult::Pass)
}

/// Function to handle signals we receive from other backend nodes
/// It's purpose is simple: receive a remote signal and emit a local
/// signal to the UI so it can react on it
#[hdk_extern]
fn recv_remote_signal(signal: ExternIO) -> ExternResult<()> {
    // Try to decode signal data we received into GameSignal
    let game_signal_result: Result<GameSignal, SerializedBytesError> = signal.decode();
    // Handle possible decoding errors
    match game_signal_result {
        // No decoding errors, so we emit local signal to the UI
        Ok(a) => emit_signal(a),
        // Decoding errors, so we just return an error
        Err(_) => Err(WasmError::Guest("Remote signal failed".into())),
    }
}

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

/// Creates a user profile and links it to the game_code
#[hdk_extern]
pub fn join_game_with_code(input: JoinGameInfo) -> ExternResult<EntryHash> {
    player_profile::join_game_with_code(input)
}

/// Lists all players who are linked to the game_code
#[hdk_extern]
pub fn get_players_for_game_code(short_unique_code: String) -> ExternResult<Vec<PlayerProfile>> {
    player_profile::get_player_profiles_for_game_code(short_unique_code)
}

/// Creates a GameSession entry for the corresponding game_code
#[hdk_extern]
pub fn start_game_session_with_code(game_code: String) -> ExternResult<EntryHash> {
    game_session::start_game_session_with_code(game_code)
}

/// Lists all game sessions created by the agent who calls this fn
#[hdk_extern]
pub fn get_my_owned_sessions(_: ()) -> ExternResult<Vec<(EntryHash, GameSession)>> {
    game_session::get_my_own_sessions_via_source_query()
}

/// Creates a new move for the given round
#[hdk_extern]
pub fn make_new_move(input: GameMoveInput) -> ExternResult<HeaderHash> {
    game_move::new_move(input.resource_amount, input.round_hash)
}

/// Function to call from the UI on a regular basis to try and close the currently
/// active GameRound. It will check the currently available GameRound state and then
/// will close it if it's possible. If not, it will return None
#[hdk_extern]
pub fn try_to_close_round(prev_round_hash: EntryHash) -> ExternResult<GameRoundInfo> {
    game_round::try_to_close_round(prev_round_hash.into())
}

#[hdk_extern]
pub fn validate_update_entry_game_round(
    data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    game_round::validate_update_entry_game_round(data)
}

#[hdk_extern]
pub fn validate_create_entry_game_move(
    validate_data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    game_move::validate_create_entry_game_move(validate_data)
}

#[hdk_extern]
pub fn validate_update_entry_game_move(
    validate_data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    game_move::validate_update_entry_game_move(validate_data)
}

#[hdk_extern]
pub fn validate_delete_entry_game_move(
    validate_data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    game_move::validate_delete_entry_game_move(validate_data)
}
