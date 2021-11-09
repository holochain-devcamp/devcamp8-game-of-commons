use crate::{
    game_code::get_game_code_anchor,
    game_round::{GameRound, RoundState},
    game_signals::{GameSignal, SignalPayload},
    player_profile::get_player_profiles_for_game_code,
};
use hdk::prelude::*;
use std::collections::BTreeMap;

// A convenient alias that would help to:
// - separate variables that store resource values from other i32 variables
// - conveniently change the Resource type if we want by making a single edit here
pub type ResourceAmount = i32;
// Alias to avoid writing the generic type specification every time
// At any given moment in time, player's state in the game is just a resource value
pub type PlayerStats = BTreeMap<AgentPubKey, ResourceAmount>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    InProgress,
    // We would want to store the last round's hash for when
    // the game is lost/finished to have an easy way to retrieve
    // the latest round, without having to traverse all the rounds
    // from the beginning
    // A game is lost for everybody when we consumed all the resources
    // and there's nothing left.
    Lost { last_round: EntryHash },
    // A game is finished when we played all rounds without depleting
    // the resources
    Finished { last_round: EntryHash },
}

#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
pub struct GameParams {
    pub regeneration_factor: f32, // how would resources re-grow every round
    pub start_amount: ResourceAmount, // how many resources are there when the game starts
    pub num_rounds: u32,          // how many rounds in the game
}

#[hdk_entry(id = "game_session", visibility = "public")]
#[derive(Clone)]
pub struct GameSession {
    pub owner: AgentPubKey,        // who started the game
    pub status: SessionState,      // how the game is going
    pub game_params: GameParams,   // what specific game are we playing
    pub players: Vec<AgentPubKey>, // who is playing
    pub scores: PlayerStats,       // end scores
    pub anchor: EntryHash,         // game code anchor that identifies this game
}

pub const OWNER_SESSION_TAG: &str = "MY_GAMES";
pub const GAME_CODE_TO_SESSION_TAG: &str = "GAME_SESSION";
pub const SESSION_TO_ROUND_TAG: &str = "GAME_ROUND";

/// Collects input info for the GameSession and calls new_session
pub fn start_game_session_with_code(game_code: String) -> ExternResult<EntryHash> {
    let anchor = get_game_code_anchor(game_code.clone())?;
    let players = get_player_profiles_for_game_code(game_code)?;
    let game_params = GameParams {
        regeneration_factor: 1.1,
        start_amount: 100,
        num_rounds: 3,
    };
    let player_keys: Vec<AgentPubKey> = players.iter().map(|x| x.player_id.clone()).collect();
    new_session(player_keys, game_params, anchor)
}

/// Creates new Holochain entry for GameSession
pub fn new_session(
    players: Vec<AgentPubKey>,
    game_params: GameParams,
    anchor: EntryHash,
) -> ExternResult<EntryHash> {
    // Agent who executes this fn is automatically the owner of the game
    let agent_info_owner = agent_info()?;
    // Create Rust struct instance to hold data of new game
    let game_session = GameSession {
        owner: agent_info_owner.agent_initial_pubkey.clone(),
        status: SessionState::InProgress,
        game_params: game_params,
        players: players.clone(),
        // there's no score yet, so we just create an empty instance of PlayerStats
        scores: PlayerStats::new(),
        anchor: anchor.clone(),
    };
    // Create a Holochain entry on DHT
    create_entry(&game_session)?;
    // Calculate hash of that entry for further usage
    let game_session_entry_hash = hash_entry(&game_session)?;

    // Create link from session owner's address to the game session entry
    // This is to allow owner to query only for their games
    create_link(
        agent_info_owner.agent_initial_pubkey.clone().into(),
        game_session_entry_hash.clone(),
        LinkTag::new(OWNER_SESSION_TAG),
    )?;

    // Create link from game code anchor to the game session entry
    // This is to make game discoverable by everyone who knows the game code anchor
    create_link(
        anchor.into(),
        game_session_entry_hash.clone(),
        LinkTag::new(GAME_CODE_TO_SESSION_TAG),
    )?;

    // Create a round zero: a dummy round we'll need to collect moves
    let round_zero = GameRound::new(
        0,
        game_session_entry_hash.clone(),
        game_session.game_params.start_amount,
        0,
        0,
        PlayerStats::new(),
    );
    // Commit round_zero to DHT
    create_entry(&round_zero)?;
    // Calculate this entry's hash (nothing is written to DHT)
    let entry_hash_round_zero = hash_entry(&round_zero)?;

    // Create a link from the game session to the round zero
    // to make it discoverable by everyone who knows game code
    // (So they'll go game_code -> game_session -> round_zero)
    create_link(
        game_session_entry_hash.clone(),
        entry_hash_round_zero.clone(),
        LinkTag::new(SESSION_TO_ROUND_TAG),
    )?;

    // We're sending signal to notify that a new round has started and that
    // players can make their moves now
    // WARNING: remote_signal is fire and forget, no error if it fails,
    // might be a weak point if this were production hApp
    let signal_payload = SignalPayload {
        game_session_entry_hash: game_session_entry_hash.into(),
        round_entry_hash_update: entry_hash_round_zero.clone().into(),
    };

    let signal = ExternIO::encode(GameSignal::StartGame(signal_payload))?;
    let other_players = others(players)?;
    remote_signal(signal, other_players)?;

    // Return hash of the round zero because players would need it
    // to make their moves, and we're saving them a lookup by doing so
    Ok(entry_hash_round_zero)
}

/// Small helper fn to filter out all players who are not the current agent
/// (the agent who is executing this fn right now)
fn others(players: Vec<AgentPubKey>) -> Result<Vec<AgentPubKey>, WasmError> {
    let me = &agent_info()?.agent_initial_pubkey;
    let others: Vec<AgentPubKey> = players.into_iter().filter(|p| p.ne(me)).collect();
    Ok(others)
}

/// Queries source chain contents of the agent executing this fn
/// Since game owner is the one creating the GameSession, they'll have all their games
/// on the source chain already, so there's no need to go to network for this.
/// This fns returns a tuple of (EntryHash, GameSession) for every game session:
/// this is to make sure that UI would have both the data to display
/// and it's hash to identify the corresponding Holochain entry for any other actions
pub fn get_my_own_sessions_via_source_query() -> ExternResult<Vec<(EntryHash, GameSession)>> {
    // Create a new filter instance that would define query we want to execute
    let filter = ChainQueryFilter::new()
        .include_entries(true)
        .entry_type(EntryType::App(AppEntryType::new(
            entry_def_index!(GameSession)?,
            zome_info()?.zome_id,
            EntryVisibility::Public,
        )));

    // Actually execute our query
    let list_of_elements = query(filter)?;
    // Below we repeat the similar logic we had in the player_profile::get_player_profiles_for_game_code:
    // only there we had to transform link to element and here we're already dealing with elements
    let mut list_of_tuples: Vec<(EntryHash, GameSession)> = vec![];
    for el in list_of_elements {
        // Retrieve an Option with our entry inside. Since not all Elements can have
        // entry, their method `entry()` returns an Option which would be None in case
        // the corresponding Element is something different.
        let entry_option = el.entry().to_app_option()?;
        // Now try to unpack the option that we received and write an error to show
        // in case it turns out there's no entry
        let gs: GameSession = entry_option.ok_or(WasmError::Guest(
            "The targeted entry is not GameSession".into(),
        ))?;
        // Calculate entry hash
        let gs_hash = el.header().entry_hash().ok_or(WasmError::Guest(
            "The targeted entry is not GameSession".into(),
        ))?;
        // Add a tuple with entry hash and actual entry to our results list
        list_of_tuples.push((gs_hash.clone(), gs));
    }
    Ok(list_of_tuples)
}

/// Ends the game session and updates it's state (finished/lost)
/// depending on the results of the last round.
/// NOTE: GameRound param is needed if we want to send a signal that session
/// has ended, but right now we don't have signals so this param is skipped
pub fn end_game(
    game_session: &GameSession,
    game_session_header_hash: &HeaderHash,
    _: &GameRound,
    last_round_entry_hash: &EntryHash,
    round_state: &RoundState,
) -> ExternResult<EntryHash> {
    info!("Ending the game");
    // If there are no resources, then the game is lost,
    // otherwise it's finished
    // NOTE: this is a Rust trick where we define value of the game_status
    // as a result of executing if and it's branches.
    let game_status = if round_state.resources_left <= 0 {
        SessionState::Lost {
            last_round: last_round_entry_hash.clone(),
        }
    } else {
        SessionState::Finished {
            last_round: last_round_entry_hash.clone(),
        }
    };
    // Create a Rust struct instance with new data of our game session
    // Most of the fields come from the original GameSession,
    // but state and scores are different
    let game_session_update = GameSession {
        owner: game_session.owner.clone(),
        status: game_status,
        game_params: game_session.game_params.clone(),
        players: game_session.players.clone(),
        scores: round_state.player_stats.clone(),
        anchor: game_session.anchor.clone(),
    };
    // Update the original game session entry on DHT with the game_session_update
    // contents. We're making an update chain from the game_session_header_hash
    let game_session_header_hash_update =
        update_entry(game_session_header_hash.clone(), &game_session_update)?;
    // Calculate the hash of the entry that we just commited
    // Reminder: update_entry would return us only the header hash,
    // but we need the entry hash,
    let game_session_entry_hash_update = hash_entry(&game_session_update)?;
    debug!(
        "updated game session header hash: {:?}",
        game_session_header_hash_update.clone()
    );
    debug!(
        "updated game session entry hash: {:?}",
        game_session_entry_hash_update.clone()
    );

    // Create a payload for signalling to other players that game has ended
    let signal_payload = SignalPayload {
        game_session_entry_hash: game_session_entry_hash_update.clone(),
        round_entry_hash_update: last_round_entry_hash.clone().into(),
    };
    // Encode our payload into a signal (no signals are sent at this point!)
    let signal = ExternIO::encode(GameSignal::GameOver(signal_payload))?;
    // Actually send other players a signal that game has ended
    remote_signal(signal, game_session.players.clone())?;

    // Return hash of the entry as the ID of the new data we commited to DHT
    Ok(game_session_entry_hash_update.clone())
}
