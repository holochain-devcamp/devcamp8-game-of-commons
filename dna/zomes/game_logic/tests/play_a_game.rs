use game_logic::{GameMoveInput, GameRoundInfo, GameSession, JoinGameInfo, PlayerProfile};
use hdk::prelude::{EntryHash, HeaderHash};
use holochain::test_utils::consistency_10s;
use holochain::{conductor::config::ConductorConfig, sweettest::*};

#[tokio::test(flavor = "multi_thread")]
async fn sweetest_example() {
    // Use prebuilt DNA file
    let dna_path = std::env::current_dir()
        .unwrap()
        .join("../../workdir/game-of-commons.dna");
    let dna = SweetDnaFile::from_bundle(&dna_path).await.unwrap();

    // Set up conductors
    let mut conductors = SweetConductorBatch::from_config(2, ConductorConfig::default()).await;
    let apps = conductors
        .setup_app("game-of-commons", &[dna])
        .await
        .unwrap();
    conductors.exchange_peer_info().await;

    let ((alice,), (bobbo,)) = apps.into_tuples();

    let alice_zome = alice.zome("game_logic");
    let bob_zome = bobbo.zome("game_logic");

    let game_code = String::from("ABCDE");

    // Alice creates a game code
    let code_hash: EntryHash = conductors[0]
        .call(&alice_zome, "create_game_code_anchor", game_code.clone())
        .await;
    println!("Alice created the game code: {}", code_hash);

    let alice_game_code = JoinGameInfo {
        gamecode: game_code.clone(),
        nickname: String::from("alice"),
    };

    // Alice joins the game with this code
    let alice_profile_hash: EntryHash = conductors[0]
        .call(&alice_zome, "join_game_with_code", alice_game_code.clone())
        .await;
    println!("Alice joined the game: {}", alice_profile_hash);

    let bob_game_code = JoinGameInfo {
        gamecode: game_code.clone(),
        nickname: String::from("bob"),
    };

    // Alice joins the game with this code
    let bob_profile_hash: EntryHash = conductors[1]
        .call(&bob_zome, "join_game_with_code", bob_game_code.clone())
        .await;
    println!("Bob joined the game: {}", bob_profile_hash);

    consistency_10s(&[&alice, &bobbo]).await;

    let list_of_players: Vec<PlayerProfile> = conductors[0]
        .call(&alice_zome, "get_players_for_game_code", game_code.clone())
        .await;
    println!("List of players in the game: {:?}", list_of_players);
    // Verify that there actually 2 players in the game: no more, no less
    assert_eq!(list_of_players.len(), 2);

    //Alice starts a new game (session) with the game code
    let first_round_entry_hash: EntryHash = conductors[0]
        .call(
            &alice_zome,
            "start_game_session_with_code",
            game_code.clone(),
        )
        .await;
    println!(
        "Alice created new game session with first round: {:?}",
        first_round_entry_hash
    );

    let alice_owned_games: Vec<(EntryHash, GameSession)> = conductors[0]
        .call(&alice_zome, "get_my_owned_sessions", ())
        .await;
    println!("Verify that Alice's owned games is 1");

    assert_eq!(alice_owned_games.len(), 1);

    let bob_owned_games: Vec<(EntryHash, GameSession)> = conductors[1]
        .call(&bob_zome, "get_my_owned_sessions", ())
        .await;
    println!("Verify that Bob's owned games is 0");

    assert_eq!(bob_owned_games.len(), 0);

    // ROUND 1
    // Alice makes her move
    let game_move = GameMoveInput {
        resource_amount: 5,
        round_hash: first_round_entry_hash.clone(),
    };
    let game_move_round_1_alice: HeaderHash = conductors[0]
        .call(&alice_zome, "make_new_move", game_move)
        .await;
    println!("ROUND 1: Alice made a move: {}", game_move_round_1_alice);

    // Bob makes her move
    let game_move = GameMoveInput {
        resource_amount: 10,
        round_hash: first_round_entry_hash.clone(),
    };
    let game_move_round_1_bob: HeaderHash = conductors[1]
        .call(&bob_zome, "make_new_move", game_move)
        .await;
    println!("ROUND 1: Bob made a move: {}", game_move_round_1_bob);

    consistency_10s(&[&alice, &bobbo]).await;

    let close_game_round_1_bob: GameRoundInfo = conductors[1]
        .call(&bob_zome, "try_to_close_round", first_round_entry_hash)
        .await;
    println!("Bob tried to close round 1: {:?}", close_game_round_1_bob);
    println!(
        "Verify that first round has ended and next_action == START_NEXT_ROUND: {}",
        close_game_round_1_bob.next_action
    );
    assert_eq!(
        close_game_round_1_bob.next_action,
        String::from("START_NEXT_ROUND")
    );
}
