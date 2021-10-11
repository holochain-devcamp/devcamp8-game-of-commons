use crate::{game_code::get_game_code_anchor, player_profile::get_player_profiles_for_game_code};
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

    // For now, return the game session entry hash
    // Once we implement a GameRound, we'll be doing more in this fn
    Ok(game_session_entry_hash)
}
