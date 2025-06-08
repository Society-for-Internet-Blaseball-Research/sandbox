# sandbox
"putting the game in a pachinko machine to see if the crabs really are good or bad" - Astrid

## What is sandbox?
Sandbox is a project dedicated to reverse-engineering Blaseball games. It's written entirely in Rust.

## Interaction
Right now you can download it via `git clone https://github.com/Society-for-Internet-Blaseball-Research/sandbox`. A CLI (command line interface) is planned.

## Components
* sandbox: crate containing components related to simulating a single game
    * `lib.rs`: The main component of the sandbox crate, containing code that interacts with and updates the game state.
    * `entities.rs`: Storing data that persists between games.
    * `sim.rs`: Generating events based on rng.
    * `events.rs`: Applying generated events to game and world.
    * `rng.rs`: The core module for generating random numbers accurate to Blaseball PRNG.
    * `formulas.rs`: Functions determining the thresholds for base events.
    * `mods.rs`: Modification logic.
    * `bases.rs`: Baserunner logic.
* sandbox\_test: crate containing components related to interacting with the sandbox crate to simulate multiple games.
    * `main.rs`: The method that runs the simulation. Edit various sections in the code to get different results.
    * `schedule.rs`: Generating a schedule.
    * `postseason.rs`: Postseason logic.
    * `get.rs`: Getting players from Chronicler for "real" seasons.

sandbox is the natural consequence of [resim](https://github.com/xSke/resim).
