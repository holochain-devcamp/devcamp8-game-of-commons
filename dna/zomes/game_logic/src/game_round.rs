use crate::{
    game_move::GameMove,
    game_session::{GameParams, PlayerStats, ResourceAmount, GameSession},
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

/// Checks if we can start a new round given the game session and
/// it's latest round (which would be previous round in regard to the one
/// we want to start)
fn can_start_new_round(
    game_session: &GameSession,
    prev_round: &GameRound,
    round_state: &RoundState,
) -> bool {
    // do we have rounds left to play?
    prev_round.round_num + 1 < game_session.game_params.num_rounds
    // are resources not depleted?
        && round_state.resources_left > 0
}

/// Creates a new game round by actually creating the next entry in the update
/// chain that starts at the round zero we created in game_sessio::new_session
/// NOTE: we'll use the first parameter of GameSession type later, but we define
/// it from the beginning to avoid changing fn signature
fn create_new_round(
    _: &GameSession,
    last_round: &GameRound,
    last_round_header_hash: &HeaderHash,
    round_state: &RoundState,
) -> ExternResult<EntryHash> {
    info!(
        "create_new_round: updating game round entry at {:?}. Last round num {:?}",
        last_round, last_round.round_num
    );
    // create a Rust struct instance with all the data we need
    let next_round = GameRound::new(
        last_round.round_num + 1,
        last_round.session.clone().into(),
        round_state.resources_left,
        round_state.resources_taken,
        round_state.resources_grown,
        // making a clone here because GameRound::new would consume player_stats
        // but we have a shared reference to it which doesn't belong to the current fn
        round_state.player_stats.clone(),
    );
    // commit an update to the DHT
    update_entry(last_round_header_hash.clone(), &next_round)?;
    // calculate the hash of the entry (no DHT writes here)
    let round_entry_hash_update = hash_entry(&next_round)?;
    Ok(round_entry_hash_update)
}
