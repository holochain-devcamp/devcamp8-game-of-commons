import { Orchestrator, Player, Cell } from "@holochain/tryorama";
import { config, installation, sleep } from "../utils";

export default (orchestrator: Orchestrator<any>) =>
  orchestrator.registerScenario("game_logic tests", async (s, t) => {
    // Declare two players using the previously specified config, nicknaming them "alice" and "bob"
    // note that the first argument to players is just an array conductor configs that that will
    // be used to spin up the conductor processes which are returned in a matching array.
    const [alice_player, bob_player]: Player[] = await s.players([
      config,
      config,
    ]);

    // install your happs into the conductors and destructuring the returned happ data using the same
    // array structure as you created in your installation array.
    const [[alice_happ]] = await alice_player.installAgentsHapps(installation);
    const [[bob_happ]] = await bob_player.installAgentsHapps(installation);

    await s.shareAllNodes([alice_player, bob_player]);

    const alice = alice_happ.cells.find((cell) =>
      cell.cellNick.includes("/game-of-commons.dna")
    ) as Cell;
    const bob = bob_happ.cells.find((cell) =>
      cell.cellNick.includes("/game-of-commons.dna")
    ) as Cell;

    const ZOME_NAME = "game_logic";
    const GAME_CODE = "ABCDE";

    // Alice creates a game code
    const codeHash = await alice.call(
      ZOME_NAME,
      "create_game_code_anchor",
      GAME_CODE
    );
    console.log("Alice created the game code: ", codeHash);
    t.ok(codeHash);

    await sleep(50);

    // Alice joins the game with this code
    const joinHashAlice = await alice.call(ZOME_NAME, "join_game_with_code", {
      gamecode: GAME_CODE,
      nickname: "Alice",
    });
    console.log("Alice joined the game: ", joinHashAlice);
    t.ok(joinHashAlice);

    // Bob joins the game with this code
    const joinHashBob = await alice.call(ZOME_NAME, "join_game_with_code", {
      gamecode: GAME_CODE,
      nickname: "Bob",
    });
    console.log("Bob joined the game: ", joinHashBob);
    t.ok(joinHashBob);

    let list_of_players = await alice.call(
      ZOME_NAME,
      "get_players_for_game_code",
      GAME_CODE
    );
    console.log("List of players in the game: ", list_of_players);
    t.ok(list_of_players);
    // Verify that there actually 2 players in the game: no more, no less
    t.ok(list_of_players.length == 2);

    //Alice starts a new game (session) with the game code
    let first_round_entry_hash = await alice.call(
      ZOME_NAME,
      "start_game_session_with_code",
      GAME_CODE
    );
    console.log(
      "Alice created new game session with first round:",
      first_round_entry_hash
    );
    t.ok(first_round_entry_hash);

    let alice_owned_games = await alice.call(
      ZOME_NAME,
      "get_my_owned_sessions",
      null
    );
    console.log("Verify that Alice's owned games is 1");
    t.ok(alice_owned_games.length == 1);

    let bob_owned_games = await bob.call(
      ZOME_NAME,
      "get_my_owned_sessions",
      null
    );
    console.log("Verify that Bob's owned games is 0");
    t.ok(bob_owned_games.length == 0);

    let new_move = await alice.call(ZOME_NAME, "make_new_move", {
      resource_amount: 10,
      round_hash: first_round_entry_hash,
    });
    console.log("Verify that Alice's has been able to create a new move");
    t.ok(new_move);
  });
