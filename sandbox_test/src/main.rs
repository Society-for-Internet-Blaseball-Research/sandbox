use sandbox::{
    bases::Baserunners,
    entities::{LegendaryItem, NameGen, World},
    events::{Event, Events},
    rng::Rng,
    sim::Sim,
    mods::{Mod, ModLifetime},
    Game, GameTeam,
};
use uuid::Uuid;

fn main() {
    //edit seed
    let mut rng = Rng::new(69, 420);
    //let mut rng = Rng::new(2200200200200200200, 1234567890987654321);
    //let mut rng = Rng::new(3141592653589793238, 2718281828459045235);
    //let mut rng = Rng::new(37, 396396396396);
    //let mut rng = Rng::new(1923746321473263448, 2938897239474837483);

    let mut world = World::new();
    let mut name_gen = NameGen::new();
    let mut teams: Vec<Uuid> = Vec::new();
    let team_names: Vec<&str> = include_str!("teams.txt").split(",").collect();
    let emojis: Vec<&str> = include_str!("emojis.txt").split(" ").collect();
    for i in 0..20 {
        teams.push(world.gen_team(&mut rng, team_names[i].to_string(), emojis[i].to_string()));
    }

    //edit mods and legendary items
    //world.team_mut(team_a).mods.add(Mod::FourthStrike, ModLifetime::Season);
    //world.player_mut(world.team(team_a).lineup[0]).add_legendary_item(LegendaryItem::TheIffeyJr);

    let mut game = Game {
        weather: sandbox::Weather::generate(&mut rng),
        top: true,
        inning: 1,
        home_team: GameTeam {
            id: teams[0],
            pitcher: world.team(teams[0]).rotation[0],
            batter: None,
            batter_index: 0,
            score: 0.0,
        },
        away_team: GameTeam {
            id: teams[1],
            pitcher: world.team(teams[1]).rotation[0],
            batter: None,
            batter_index: 0,
            score: 0.0,
        },
        balls: 0,
        strikes: 0,
        outs: 0,
        polarity: false,
        scoring_plays_inning: 0,
        salmon_resets_inning: 0,
        events: Events::new(),
        runners: Baserunners::new(if world.team(teams[1]).mods.has(Mod::FifthBase) { 5 } else { 4 }),
        linescore_home: vec![0.0],
        linescore_away: vec![0.0],
    };

    if world.team(teams[0]).mods.has(Mod::HomeFieldAdvantage) {
        game.home_team.score += 1.0;
    }

    loop {
        let mut sim = Sim::new(&mut world, &mut rng);
        let evt = sim.next(&game);

        // keeping sim outside the loop means it borrows world and we can't pass it as mut here, which might be fine...?
        evt.apply(&mut game, &mut world);

        if let Event::GameOver = evt {
            println!(
                "game over! {}: {}, {}: {}",
                world.team(game.away_team.id).name,
                game.away_team.score,
                world.team(game.home_team.id).name,
                game.home_team.score
            );
            break;
        } 

        let base = if game.runners.base_number == 5 {
            format!(
            "[{}|{}|{}|{}]",
            if game.runners.occupied(3) { "X" } else { " " },
            if game.runners.occupied(2) { "X" } else { " " },
            if game.runners.occupied(1) { "X" } else { " " },
            if game.runners.occupied(0) { "X" } else { " " }
            )
        } else {
            format!(
            "[{}|{}|{}]",
            if game.runners.occupied(2) { "X" } else { " " },
            if game.runners.occupied(1) { "X" } else { " " },
            if game.runners.occupied(0) { "X" } else { " " }
            )
        };

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
