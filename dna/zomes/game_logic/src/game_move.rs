use hdk::prelude::*;
use crate::game_session::ResourceAmount;

#[hdk_entry(id = "game_move", visibility = "public")]
#[derive(Clone)]
pub struct GameMove {
    pub owner: AgentPubKey,
    pub round: EntryHash,
    pub resources: ResourceAmount,
}