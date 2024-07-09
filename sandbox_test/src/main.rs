use sandbox::{
    bases::Baserunners,
    entities::{World},
    rng::Rng,
    sim::{Event, Sim},
    mods::{Mod, ModLifetime},
    Game, GameTeam,
};

fn main() {
    //let mut rng = Rng::new(69, 420);
    //let mut rng = Rng::new(2200200200200200200, 1234567890987654321);
    //let mut rng = Rng::new(3141592653589793238, 2718281828459045235);
    let mut rng = Rng::new(37, 396396396396);
    //let mut rng = Rng::new(1923746321473263448, 2938897239474837483);

    let mut world = World::new();
    let team_a = world.gen_team(&mut rng, "Team A".to_string(), "A".to_string());
    let team_b = world.gen_team(&mut rng, "Team B".to_string(), "B".to_string());

    world.player_mut(world.team(team_a).rotation[0]).mods.add(Mod::DebtU, ModLifetime::Permanent);

    let mut game = Game {
        weather: sandbox::Weather::Eclipse,
        top: true,
        inning: 1,
        home_team: GameTeam {
            id: team_a,
            pitcher: world.team(team_a).rotation[0],
            batter: None,
            batter_index: 0,
            score: 0.0,
        },
        away_team: GameTeam {
            id: team_b,
            pitcher: world.team(team_b).rotation[0],
            batter: None,
            batter_index: 0,
            score: 0.0,
        },
        balls: 0,
        strikes: 0,
        outs: 0,
        events_inning: 0,
        polarity: false,
        scoring_plays_inning: 0,
        salmon_resets_inning: 0,
        last_salmon_inning: 0,
        runners: Baserunners::new(),
        linescore_home: vec![0.0],
        linescore_away: vec![0.0],
    };

    loop {
        let mut sim = Sim::new(&mut world, &mut rng);
        let evt = sim.next(&game);
        game.events_inning += 1;

        // keeping sim outside the loop means it borrows world and we can't pass it as mut here, which might be fine...?
        evt.apply(&mut game, &mut world);

        if let Event::GameOver = evt {
            println!("game over!");
            break;
        }

        let base = format!(
            "[{}|{}|{}]",
            if game.runners.occupied(2) { "X" } else { " " },
            if game.runners.occupied(1) { "X" } else { " " },
            if game.runners.occupied(0) { "X" } else { " " }
        );

        let away_score = (game.away_team.score * 10.0).round() / 10.0;
        let home_score = (game.home_team.score * 10.0).round() / 10.0; //floats

        println!(
            "{}{} {}@{} ({}b/{}s/{}o) {} {:?}",
            if game.top { "t" } else { "b" },
            game.inning,
            away_score,
            home_score,
            game.balls,
            game.strikes,
            game.outs,
            base,
            evt
        );
    }

    // println!("Hello, world!");
}
