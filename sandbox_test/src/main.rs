use sandbox::{
    bases::Baserunners,
    entities::{World},
    rng::Rng,
    sim::{Event, Sim},
    Game, GameTeam,
};

fn main() {
    let mut rng = Rng::new(69, 420);
    //let mut rng = Rng::new(2200200200200200200, 1234567890987654321);
    //let mut rng = Rng::new(3141592653589793238, 2718281828459045235);
    //let mut rng = Rng::new(37, 396396396396);

    let mut world = World::new();
    let team_a = world.gen_team(&mut rng, "Team A".to_string(), "A".to_string());
    let team_b = world.gen_team(&mut rng, "Team B".to_string(), "B".to_string());

    let mut game = Game {
        weather: sandbox::Weather::Blooddrain,
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
        runners: Baserunners::new(),
    };

    loop {
        let mut sim = Sim::new(&mut world, &mut rng);
        let evt = sim.next(&game);

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

        println!(
            "{}{} {}@{} ({}b/{}s/{}o) {} {:?}",
            if game.top { "t" } else { "b" },
            game.inning,
            game.away_team.score,
            game.home_team.score,
            game.balls,
            game.strikes,
            game.outs,
            base,
            evt
        );
    }

    // println!("Hello, world!");
}
