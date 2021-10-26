use crate::{
    game_move::GameMove,
    game_session::{GameParams, PlayerStats, ResourceAmount},
    utils::player_stats_from_moves,
};
use hdk::prelude::*;

// Having a separate struct for the round state would come in
// handy later in development
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundState {
    // amount of resources at the beginning of the round
    pub resources_left: ResourceAmount,
    // total amount of resources consumed during the round
    pub resources_taken: ResourceAmount,
    // amount of resources that regrew at the end of the round
    pub resources_grown: ResourceAmount,
    // resource statistics for every player
    pub player_stats: PlayerStats,
}

#[hdk_entry(id = "game_round", visibility = "public")]
#[derive(Clone, PartialEq, Eq)]
pub struct GameRound {
    // number of current round, from 0
    pub round_num: u32,
    // GameSession to which this round belongs
    pub session: EntryHash,
    // state of this round
    pub state: RoundState,
}

// That's a Rust way of providing methods that would be called on specific
// instances of a struct. This block is for the GameRound struct
// Learn more here: https://doc.rust-lang.org/book/ch05-03-method-syntax.html
impl GameRound {
    /// Creates a new GameRound instance with the provided input
    /// We're writing this method to encapsulate creating RoundState instance
    /// to initialize the state field of GameRound
    pub fn new(
        round_num: u32,
        session: EntryHash,
        resources_left: ResourceAmount,
        resources_taken: ResourceAmount,
        resources_grown: ResourceAmount,
        player_stats: PlayerStats,
    ) -> GameRound {
        let state = RoundState {
            resources_left,
            resources_taken,
            resources_grown,
            player_stats,
        };
        GameRound {
            round_num,
            session,
            state,
        }
    }
}

/// Calculate state of the round using provided game params and player moves
/// NOTE: this fn would be used both in validation and when creating game round entries
/// so it doesn't make any DHT queries and only operates with input data
fn calculate_round_state(
    last_round: &GameRound,
    params: &GameParams,
    player_moves: Vec<GameMove>,
) -> RoundState {
    let consumed_resources_in_round: ResourceAmount =
        player_moves.iter().map(|x| x.resource_amount).sum();
    let resources_left = last_round.state.resources_left - consumed_resources_in_round;
    let total_leftover_resource = (resources_left as f32 * params.regeneration_factor) as i32;
    let grown_resources_in_round = total_leftover_resource - resources_left;

    let player_stats = player_stats_from_moves(player_moves);

    RoundState {
        resources_left: total_leftover_resource,
        resources_taken: consumed_resources_in_round,
        resources_grown: grown_resources_in_round,
        player_stats,
    }
}
