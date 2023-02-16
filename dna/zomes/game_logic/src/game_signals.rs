use crate::player_profile::PlayerProfile;
use hdk::prelude::*;

/// Our signals aren't too different from each other, so
/// we'll only need one type to cover most of them
#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct SignalPayload {
    pub game_session_entry_hash: EntryHash,
    pub round_entry_hash_update: EntryHash,
}

// Different kinds of signals available in our hApp
#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
#[serde(tag = "signal_name", content = "signal_payload")]
pub enum GameSignal {
    PlayerJoined(PlayerProfile),
    StartGame(SignalPayload),
    StartNextRound(SignalPayload),
    GameOver(SignalPayload),
}
