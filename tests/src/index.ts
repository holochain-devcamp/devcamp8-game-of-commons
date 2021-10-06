
import { Orchestrator } from "@holochain/tryorama";

import game_logic from './game-of-commons/game_logic';

let orchestrator: Orchestrator<any>;

orchestrator = new Orchestrator();
game_logic(orchestrator);
orchestrator.run();



