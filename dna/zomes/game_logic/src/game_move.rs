use crate::{game_session::ResourceAmount, utils::try_get_and_convert};
use hdk::prelude::*;
use std::collections::BTreeMap;

pub const GAME_MOVE_LINK_TAG: &str = "GAME_MOVE";

#[hdk_entry(id = "game_move", visibility = "public")]
#[derive(Clone)]
pub struct GameMove {
    pub owner: AgentPubKey,
    pub round_hash: EntryHash,
    pub resource_amount: ResourceAmount,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameMoveInput {
    pub resource_amount: ResourceAmount,
    pub round_hash: EntryHash,
}

/**
 * Create a new move entry, and link it from its round
 */
pub fn new_move(
    resource_amount: ResourceAmount,
    round_hash: EntryHash,
) -> ExternResult<HeaderHash> {
    // We don't have to pass as parameter the author of the move, because
    // the agent that's executing this code will always be the author of the move
    // So just their public key from the local conductor
    let agent_info = agent_info()?;

    // Construct the contents of the entry
    let game_move = GameMove {
        owner: agent_info.agent_latest_pubkey,
        resource_amount,
        round_hash: round_hash.clone(),
    };

    // Create the entry
    create_entry(game_move.clone())?;

    // Get the hash of the entry, which is what `create_link` needs
    let move_entry_hash = hash_entry(game_move)?;

    // Link from the round entry to the newly created move so that other agents can discover it
    let create_link_header_hash = create_link(
        round_hash,
        move_entry_hash,
        LinkTag::new(String::from(GAME_MOVE_LINK_TAG)),
    )?;

    Ok(create_link_header_hash)
}

/// Get all moves attached to the round that we have so far
pub fn get_moves_for_round(last_round_hash: EntryHash) -> ExternResult<Vec<GameMove>> {
    let links = get_links(
        last_round_hash,
        Some(LinkTag::new(String::from(GAME_MOVE_LINK_TAG))),
    )?;
    let mut moves: Vec<GameMove> = vec![];
    for link in links.into_inner() {
        let game_move: GameMove = try_get_and_convert(link.target, GetOptions::latest())?;
        moves.push(game_move);
    }
    Ok(moves)
}

/// Consumes list of moves passed to it to finalize them.
/// If every player made at least one move, it returns list of moves which is guaranteed
/// to have a single move for every player.
/// If there are missing moves, it returns None, since we can't finalize the moves and
/// have to wait for other players instead.
pub fn finalize_moves(
    moves: Vec<GameMove>,
    number_of_players: usize,
) -> ExternResult<Option<Vec<GameMove>>> {
    // Check that at least we have as many moves
    // as there are players in the game
    if moves.len() < number_of_players {
        info!(
            "Cannot finalize moves: there are {} players total but only {} moves have been made",
            number_of_players,
            moves.len()
        );
        return Ok(None);
    } else {
        // Now that we know we have moves >= num of players, we need
        // to make sure that every player made at least one move, so
        // we're not closing the round without someone's move
        let mut moves_per_player: BTreeMap<AgentPubKey, Vec<GameMove>> = BTreeMap::new();
        for m in moves {
            match moves_per_player.get_mut(&m.owner) {
                Some(moves) => moves.push(m),
                // TODO(e-nastasia): cloning owner value seems like a waste, but I think
                // that alternative would be to use lifetimes. Not sure it's worth the
                // readability penalty that we'll incur.
                None => {
                    moves_per_player.insert(m.owner.clone(), vec![m]);
                }
            }
        }
        if moves_per_player.keys().len() < number_of_players {
            info!("Cannot close the round: only {} players made their moves, waiting for total {} players", moves_per_player.keys().len(), number_of_players);
            return Ok(None);
        }
        let mut new_moves = vec![];
        for (_, move_vec) in moves_per_player {
            // NOTE(e-nastasia): if we add a timestamp to the game move, we'll be able to
            // filter moves here, but for now we'll do with just taking some move
            new_moves.push(move_vec[0].clone());
        }
        Ok(Some(new_moves))
    }
}
