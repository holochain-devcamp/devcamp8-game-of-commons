use crate::game_session::ResourceAmount;
use hdk::prelude::*;

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
