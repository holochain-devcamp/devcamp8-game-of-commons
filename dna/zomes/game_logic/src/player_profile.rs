use crate::game_code::{get_game_code_anchor,create_game_code_anchor};
use hdk::prelude::*;

pub const PLAYER_LINK_TAG: &str = "PLAYER";

/// This is a Rust structure which represents an actual
/// Holochain entry that stores user's profile for the specific game
/// First we derive just a Rust struct, and then we apply hdk_entry
/// macro to it, which generates code to impelement Holochain entry.
/// id defines how this entry would be called, while visibility defines
/// where an entry will be stored. We plan to store it on DHT, so we
/// go with the "public" value
/// `#[derive(Clone)]` is needed to implement a Rust trait to allow
/// deep copies of the Rust struct, which would come in handy when we
/// want to use.
#[hdk_entry(id = "player_profile", visibility = "public")]
#[derive(Clone)]
pub struct PlayerProfile {
    pub player_id: AgentPubKey,
    pub nickname: String,
}

/// Struct to receive user input from the UI when user
/// wants to join the game.
/// Note that there are more traits implemented: we need those
/// to be able to send this struct via our zome API
#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct JoinGameInfo {
    pub gamecode: String,
    pub nickname: String,
}

/// Creates a PlayerProfile instance, commits it as a Holochain entry
/// and returns a hash value of this entry
pub fn create_and_hash_entry_player_profile(nickname: String) -> ExternResult<EntryHash> {
    // Retrieve info about an agent who is currently executing this code
    // For every instance of the app this would produce different results.
    let agent = agent_info()?;
    // Print some debug output into the logs, you'll see it when running
    // integration tests / app in conductor
    // Note the `{:?}` thing: this is what you write when you need to print
    // a Rust struct that implements the Debug trait. For things that implement
    // Display trait (like nickname here of String type) simple `{}` would do.
    debug!(
        "create_and_hash_entry_player_profile | nickname: {}, agent {:?}",
        nickname,
        agent.clone()
    );
    // Instantiate a Rust struct to store this data
    let player_profile = PlayerProfile {
        // Beware: this is bad design for real apps, because:
        // 1/ initial_pubkey is linked to app itself, so no roaming profile
        // 2/ lost if app is reinstalled (= that would be basically a new user)
        player_id: agent.agent_initial_pubkey,
        nickname,
    };
    // Commit the Rust struct instance to DHT
    // This is where actual write to DHT happens.
    // Note: this fn isn't idempotent! If someone would try to commit the
    // same player_profile multiple times, every time a Header about entry creation
    // would be created. Since the data is the same, it wouldn't affect it
    // and since our app logic doesn't look for these headers, it wouldn't
    // break the app.
    create_entry(&player_profile)?;
    debug!("create_and_hash_entry_player_profile | profile created, hashing");
    // Calculate a hash value of the entry we just written to DHT:
    // that would be essentially ID of that piece of information.
    // And since there's no ; in the end, this is what we return from current fn
    hash_entry(&player_profile)
}

/// Creates user's profile for the game and registers this user as one of the game players
/// Notice how we packed all input parameters in a single struct: this is a requirement
/// for our function to be exposed as zome API. And even though this particular fn isn't
/// exposed (there's a wrapper for it in lib.rs that is), it's easier for them to have the
/// same signature. Also it's nice to be able to read about all datatypes that cross the API
/// as those would need to be defined as structs.
/// 
/* 
When you create an anchor with the function return a EntryHash. Once you
know the entry_hash of an anchor it is best to use the get_anchor(entry_hash) fn to retrieve
this anchor, when you need it. In the case of the devcamp game, we have a little problem.
Players share the game_code via chat or voice or video... That means that the player who
initiated the game, the game leader, knows the entry_hash of the game code, but players that
want to join the game do not. Other players need to be able to find the same anchor if they
want to join the game. Of course the game leader could communicate the entry hash, but that
is not as convenient as passing the much shorter game code.
So for other players that do not have the game code the problem exists in finding out the
entry hash of the anchor while they only have game code.

There are 2 approaches you can take to solve this problem, each with it own benefits.
1/ Other players can take the game_code and calculate the hash, without actually creating
    a anchor in the DHT (with the same entry hash, but a different header hash). Like we do
    in player_profile::get_game_code_anchor
Benefits: less DHT operations, no extra header in the DHT
Downside: calculating the entry_hash and fetching the anchor with this hash via 'get_anchor',
            does not guarantee that anchor will be found at the point in time that you start
            searching it. Even if you have a entry_hash of entry that absolutely, 100% exists.
            It does not guarantee it can be found in your part of the DHT, yet. Eventually it
            will be.The downside is you to need poll until you find the anchor. This how you
            could calculate a entry hash:
    let path: Path = (&Anchor {
            anchor_type: GAME_CODES_ANCHOR.into(),
            anchor_text: Some(game_code),
        })
        .into();
    let anchor_hash = path.hash()
2/ The other way is for the other players to create the same anchor. Which we do here by calling
player_profile::create_game_code_anchor. The anchor entry will be
created again. It will add a header and a entry to the DHT. But since the entry has the same
entry_hash it will already be stored.
Benefit: entry is added to your source chain before being sent to the DHT, so it is
immediately available. No polling needed
Downside: More DHT ops, extra header in the DHT
*/
pub fn join_game_with_code(input: JoinGameInfo) -> ExternResult<EntryHash> {
    // Another example of logs output with a different priority level
    info!("join_game_with_code | input: {:?}", input);
    // Create an anchor for the game code provided in input
    let anchor = create_game_code_anchor(input.gamecode)?;
    debug!("join_game_with_code | anchor created {:?}", &anchor);
    // Create player's profile. So far it isn't connected to anything,
    // just a combination of nickname & pub key
    let player_profile_entry_hash = create_and_hash_entry_player_profile(input.nickname)?;
    debug!(
        "join_game_with_code | profile entry hash {:?}",
        &player_profile_entry_hash
    );
    // Create a uni-directional link from the anchor (base) to
    // the player's profile (target) with a tag value of PLAYER_LINK_TAG
    // Having a tag value for the link helps to keep data scheme organized
    create_link(
        anchor.clone().into(),
        player_profile_entry_hash.into(),
        LinkTag::new(String::from(PLAYER_LINK_TAG)),
    )?;
    debug!("join_game_with_code | link created");
    // Return entry hash of the anchor wrapped in ExternResult::Ok variant
    Ok(anchor)
}

/// Retrieves player profiles that are linked to the anchor for the provided
/// short_unique_code.
pub fn get_player_profiles_for_game_code(
    short_unique_code: String,
) -> ExternResult<Vec<PlayerProfile>> {
    // Retrieve entry hash of our game code anchor
    let anchor = get_game_code_anchor(short_unique_code)?;
    debug!("anchor: {:?}", anchor);
    // Retrieve a set of links that have anchor as a base, with the tag PLAYER_LINK_TAG
    let links: Links = get_links(anchor, Some(LinkTag::new(String::from(PLAYER_LINK_TAG))))?;
    debug!("links: {:?}", links);
    // The following code isn't idiomatic Rust and could've been written
    // in a much more elegant & short way. But, that woudln't have been easy
    // to read for people unfamiliar with Rust, so here we go.
    // First, create a buffer vec for our results. Make it mutable so we
    // can add results one-by-one later
    let mut players = vec![];
    // Iterate through all the links contained inside the link instance
    for link in links.into_inner() {
        debug!("link: {:?}", link);
        // Retrieve an element at the hash specified by link.target
        // No fancy retrieve options are applied, so we just go with GetOptions::default()
        let element: Element = get(link.target, GetOptions::default())?
            .ok_or(WasmError::Guest(String::from("Entry not found")))?;
        // Retrieve an Option with our entry inside. Since not all Elements can have
        // entry, their method `entry()` returns an Option which would be None in case
        // the corresponding Element is something different.
        let entry_option = element.entry().to_app_option()?;
        // Now try to unpack the option that we received and write an error to show
        // in case it turns out there's no entry
        let entry: PlayerProfile = entry_option.ok_or(WasmError::Guest(
            "The targeted entry is not agent pubkey".into(),
        ))?;
        // Add this PlayerProfile to our results vector
        players.push(entry);
    }

    // wrap our vector into ExternResult and return
    Ok(players)
}
