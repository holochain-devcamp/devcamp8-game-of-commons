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
