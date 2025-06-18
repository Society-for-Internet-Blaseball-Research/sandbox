# sandbox
"putting the game in a pachinko machine to see if the crabs really are good or bad" - Astrid

## What is sandbox?
Sandbox is a project dedicated to reverse-engineering Blaseball games. It's written entirely in Rust.

## Interaction
You can download it via `git clone https://github.com/Society-for-Internet-Blaseball-Research/sandbox`.

Run a default season by running `cargo run -- --prefill --seasonmode`.

## CLI Arguments

* s1: (u64) First part of the seed. Default: 69.
* s2: (u64) Second part of the seed. Default: 420.
* prefill: (bool) Whether sandbox should use real players (true) or generate them (false). Default: false.
* season: (u8) Which season's rules should sandbox use (0-indexed). Default: 11.
* teams: (usize) The team number. Default: 20.
* divsize: (usize) How many teams in a division. Default: 5.
* seasonmode: (bool) Whether a sandbox loop should be a season (true) or a game (false). Default: false.
* loops: (usize) How many loops should sandbox go through. Default: 1/

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
