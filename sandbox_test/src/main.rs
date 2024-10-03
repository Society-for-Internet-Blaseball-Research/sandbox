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
    //let name_gen = NameGen::new();
    let mut teams: Vec<Uuid> = Vec::new();
    let team_names: Vec<&str> = include_str!("teams.txt").split(",").collect();
    let emojis: Vec<&str> = include_str!("emojis.txt").split(" ").collect();
    //todo: dehardcode team number
    for i in 0..20 {
        teams.push(world.gen_team(&mut rng, team_names[i].to_string(), emojis[i].to_string()));
    }

    //edit mods and legendary items
    //world.team_mut(team_a).mods.add(Mod::FourthStrike, ModLifetime::Season);
    //world.player_mut(world.team(team_a).lineup[0]).add_legendary_item(LegendaryItem::TheIffeyJr);

    let mut games: Vec<Game> = Vec::new();
    for i in 0..10 {
        games.push(generate_game(teams[2 * i], teams[2 * i + 1], &mut rng, &world));
    }
    let mut sim = Sim::new(&mut world, &mut rng);
    let mut games_active = games.clone();
    let mut games_deactivated: Vec<Uuid> = vec![];
    loop {
        //todo: store games through world
        for game in games_active.iter_mut() {
            //todo: make games a field of Sim
            let evt = sim.next(game);
            // keeping sim outside the loop means it borrows world and we can't pass it as mut here, which might be fine...?
            evt.apply(game, sim.world);
            if let Event::InningSwitch {inning, top} = evt {
                println!("{} {}", if top { "top" } else { "bottom" }, inning);
            }

            if let Event::GameOver = evt {
                println!(
                    "game over! {}: {}, {}: {}",
                    sim.world.team(game.away_team.id).name,
                    game.away_team.score,
                    sim.world.team(game.home_team.id).name,
                    game.home_team.score
                );
                games_deactivated.push(game.id);
            }
        }
        games_active.retain(|g| !games_deactivated.contains(&g.id));
        if games_active.len() == 0 {
            break;
        }

        /*let base = if game.runners.base_number == 5 {
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
        );*/
    }

    // println!("Hello, world!");
}

fn generate_game(team_a: Uuid, team_b: Uuid, rng: &mut Rng, world: &World) -> Game {
    Game {
        id: Uuid::new_v4(),
        weather: sandbox::Weather::generate(rng),
        top: true,
        inning: 1,
        home_team: GameTeam {
            id: team_a,
            //todo: days
            pitcher: world.team(team_a).rotation[0],
            batter: None,
            batter_index: 0,
            score: if world.team(team_a).mods.has(Mod::HomeFieldAdvantage) { 1.0 } else { 0.0 },
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
        polarity: false,
        scoring_plays_inning: 0,
        salmon_resets_inning: 0,
        events: Events::new(),
        runners: Baserunners::new(if world.team(team_b).mods.has(Mod::FifthBase) { 5 } else { 4 }),
        linescore_home: vec![if world.team(team_a).mods.has(Mod::HomeFieldAdvantage) { 1.0 } else { 0.0 }],
        linescore_away: vec![0.0],
    }
}

/*fn generate_schedule(teams: Vec<Uuid>, days: u8) -> BTree<i16, Game> {
    
}*/
