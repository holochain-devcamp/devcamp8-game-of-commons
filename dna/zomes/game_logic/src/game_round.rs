use crate::{
    game_move::{finalize_moves, get_moves_for_round, GameMove},
    game_session::{GameParams, GameSession, PlayerStats, ResourceAmount},
    utils::{player_stats_from_moves, try_from_element, try_get_element},
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

/// Helper struct to package output for the UI
#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct GameRoundInfo {
    pub round_num: u32,
    pub resources_left: Option<ResourceAmount>,
    pub resources_taken_round: Option<ResourceAmount>,
    pub resources_grown_round: Option<ResourceAmount>,
    pub current_round_entry_hash: Option<EntryHash>,
    pub prev_round_entry_hash: Option<EntryHash>,
    pub game_session_hash: Option<EntryHash>,
    pub next_action: String,
    pub moves: Vec<(ResourceAmount, String, AgentPubKey)>,
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

/// Poll the current DHT state to check if player executing this fn can
/// close the current round
pub fn try_to_close_round(last_round_hash: EntryHash) -> ExternResult<GameRoundInfo> {
    // Retrieve last round element from the DHT and convert it to a Rust struct instance
    // We would need both the element and the struct instance during this fn
    let last_round_element = try_get_element(last_round_hash.clone(), GetOptions::latest())?;
    let last_round: GameRound = try_from_element(last_round_element.clone())?;

    // TODO optimization: check if new round with next round_num already exists
    // in that case we can skip the rest of this fn, because that means that while
    // we're executing it someone else already closed the round at last_round_hash
    // so we can save ourselves the necesity to commit this round update again

    // Retrieve game session element from the DHT and convert it to a Rust struct instance
    // We would need both the element and the struct instance during this fn
    let game_session_element = try_get_element(last_round.session.clone(), GetOptions::latest())?;
    let game_session: GameSession = try_from_element(game_session_element.clone())?;

    // Retrieve game moves from DHT
    let moves = get_moves_for_round(last_round_hash.clone())?;

    // Try to process those moves and see if we have enough to close the round
    match finalize_moves(moves, game_session.players.len())? {
        // we get the moves (which are guaranteed to be unique, hence the name),
        // so we can close the round
        Some(unique_moves) => {
            let mut moves_info: Vec<(ResourceAmount, String, AgentPubKey)> = vec![];
            for game_move in &unique_moves {
                moves_info.push((
                    game_move.resource_amount.clone(),
                    "playername".into(),
                    game_move.owner.clone(),
                ));
            }
            info!("all players made their moves: calculating round state");
            let round_state =
                calculate_round_state(&last_round, &game_session.game_params, unique_moves);
            // Check if we can start the next round
            if can_start_new_round(&game_session, &last_round, &round_state) {
                let round_hash = create_new_round(
                    &game_session,
                    &last_round,
                    last_round_element.header_address(),
                    &round_state,
                )?;
                return Ok(GameRoundInfo {
                    current_round_entry_hash: Some(round_hash),
                    prev_round_entry_hash: Some(last_round_hash),
                    game_session_hash: None,
                    resources_left: Some(round_state.resources_left),
                    resources_taken_round: Some(round_state.resources_taken),
                    resources_grown_round: Some(round_state.resources_grown),
                    round_num: last_round.round_num + 1,
                    next_action: "START_NEXT_ROUND".into(),
                    moves: moves_info,
                });
            } else {
                // NOTE: we'll be closing the game session here, later in the devcamp
                // This return is needed here to ensure all if branches of our fn return
                // the same datatype, otherwise it's quite useless
                return Ok(GameRoundInfo {
                    current_round_entry_hash: None,
                    prev_round_entry_hash: None,
                    game_session_hash: None,
                    resources_left: None,
                    resources_taken_round: None,
                    resources_grown_round: None,
                    round_num: last_round.round_num + 1,
                    next_action: "SHOW_GAME_RESULTS".into(),
                    moves: vec![],
                });
            }
        }
        // There aren't enough moves yet, so we get nothing and wait
        None => {
            return Ok(GameRoundInfo {
                current_round_entry_hash: None,
                prev_round_entry_hash: Some(last_round_hash),
                game_session_hash: Some(last_round.session.clone()),
                resources_left: None,
                resources_taken_round: None,
                resources_grown_round: None,
                round_num: last_round.round_num,
                next_action: "WAITING".into(),
                moves: vec![],
            });
        }
    };
}
